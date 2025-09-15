use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{debug, error};

use crate::server::{operation::NfsOperation, request::NfsRequest, response::NfsOpResponse};

use bold_proto::nfs4_proto::{
    DirList4, Entry4, Fattr4, NfsResOp4, NfsStat4, ReadDir4res, ReadDir4resok, Readdir4args,
};

#[async_trait]
impl NfsOperation for Readdir4args {
    async fn execute<'a>(&self, request: NfsRequest<'a>) -> NfsOpResponse<'a> {
        debug!(
            "Operation 26: READDIR - Read Directory {:?}, with request {:?}",
            self, request
        );
        let current_fh = request.current_filehandle();
        let dir_fh = match current_fh {
            Some(filehandle) => filehandle,
            None => {
                error!("None filehandle");
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errFhexpired,
                };
            }
        };

        // Read directory entries and collect them
        let dir = match dir_fh.file.read_dir() {
            Ok(dir) => dir,
            Err(e) => {
                error!("Failed to read directory: {:?}", e);
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errIo,
                };
            }
        };

        // Collect all directory entries and sort them by name for stable ordering
        let mut entries: Vec<(String, String)> = Vec::new(); // (name, path)
        for entry in dir {
            let name = entry.filename();
            // Skip "." and ".." entries as per NFS specification
            if name != "." && name != ".." {
                entries.push((name.clone(), entry.as_str().to_string()));
            }
        }
        
        // Sort entries by name to ensure stable ordering
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Generate a stable cookie verifier based on directory metadata
        let dir_metadata = match dir_fh.file.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errIo,
                };
            }
        };
        
        // Create a stable hash based on directory change time and file IDs
        let mut hasher = DefaultHasher::new();
        if let Some(modified) = dir_metadata.modified {
            modified.hash(&mut hasher);
        }
        for (name, _) in &entries {
            name.hash(&mut hasher);
        }
        let dir_hash = hasher.finish().to_be_bytes();
        let cookieverf = [
            dir_hash[0], dir_hash[1], dir_hash[2], dir_hash[3],
            dir_hash[4], dir_hash[5], dir_hash[6], dir_hash[7]
        ];

        // Check cookie verifier if this is not the first request
        if self.cookie != 0 && cookieverf != self.cookieverf {
            error!("Nfs4errNotSame - cookie verifier mismatch");
            return NfsOpResponse {
                request,
                result: None,
                status: NfsStat4::Nfs4errNotSame,
            };
        }

        // Find the starting position based on the cookie
        let start_index = if self.cookie == 0 {
            0 // Start from the beginning
        } else {
            // Cookie values 0, 1, 2 are reserved, actual cookies start from 3
            // So cookie 3 corresponds to index 0, cookie 4 to index 1, etc.
            if self.cookie < 3 {
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errBadCookie,
                };
            }
            (self.cookie - 3) as usize
        };

        // Check if the start index is valid
        if start_index > entries.len() {
            return NfsOpResponse {
                request,
                result: None,
                status: NfsStat4::Nfs4errBadCookie,
            };
        }

        // Build the response entries respecting dircount and maxcount limits
        let mut response_entries: Vec<Entry4> = Vec::new();
        let mut current_dircount: usize = 0;
        let mut current_maxcount: usize = 100; // Base overhead for the response structure
        let dircount_limit = self.dircount as usize;
        let maxcount_limit = self.maxcount as usize;

        let mut i = start_index;
        let mut eof = true; // Assume we've reached the end

        while i < entries.len() {
            let (name, path) = &entries[i];
            
            // Calculate XDR size for this entry
            // Entry4 XDR size estimation:
            // - cookie: 8 bytes
            // - name length: 4 bytes
            // - name: variable length (UTF-8 string)
            // - attrs bitmask: 4 bytes + variable
            // - attrs values: variable
            // - nextentry pointer: 4 bytes (optional)
            let name_xdr_size = 8 + 4 + name.len() + 4 + 100 + 4; // Approximate size
            
            // Check if we can fit this entry
            if dircount_limit > 0 && (current_dircount + name.len() + 8) > dircount_limit {
                // We've reached the dircount limit
                eof = false;
                break;
            }
            
            if maxcount_limit > 0 && (current_maxcount + name_xdr_size) > maxcount_limit {
                // Check if this is the first entry and it doesn't fit
                if response_entries.is_empty() {
                    // One entry doesn't fit in maxcount, return NFS4ERR_TOOSMALL
                    return NfsOpResponse {
                        request,
                        result: None,
                        status: NfsStat4::Nfs4errToosmall,
                    };
                }
                // We've reached the maxcount limit
                eof = false;
                break;
            }

            // Update counters
            current_dircount += name.len() + 8; // name + cookie
            current_maxcount += name_xdr_size;

            // Get file handle for this entry
            let filehandle = match request
                .file_manager()
                .get_filehandle_for_path(path.clone())
                .await {
                Ok(fh) => fh,
                Err(e) => {
                    error!("Failed to get filehandle for {}: {:?}", path, e);
                    // Use fattr4_rdattr_error instead of failing the whole operation
                    let attrs = Fattr4 {
                        attrmask: bold_proto::nfs4_proto::Attrlist4::<bold_proto::nfs4_proto::FileAttr>::new(None),
                        attr_vals: bold_proto::nfs4_proto::Attrlist4::<bold_proto::nfs4_proto::FileAttrValue>::new(None),
                    };
                    
                    let entry = Entry4 {
                        name: name.clone(),
                        cookie: (i + 3) as u64, // Cookie values start from 3
                        attrs,
                        nextentry: None,
                    };
                    response_entries.push(entry);
                    i += 1;
                    continue;
                }
            };

            // Get attributes for this entry
            let resp = request
                .file_manager()
                .filehandle_attrs(&self.attr_request, &filehandle);
                
            let (answer_attrs, attrs) = match resp {
                Some(inner) => inner,
                None => {
                    // Use fattr4_rdattr_error instead of failing the whole operation
                    let attrs = Fattr4 {
                        attrmask: bold_proto::nfs4_proto::Attrlist4::<bold_proto::nfs4_proto::FileAttr>::new(None),
                        attr_vals: bold_proto::nfs4_proto::Attrlist4::<bold_proto::nfs4_proto::FileAttrValue>::new(None),
                    };
                    
                    let entry = Entry4 {
                        name: name.clone(),
                        cookie: (i + 3) as u64, // Cookie values start from 3
                        attrs,
                        nextentry: None,
                    };
                    response_entries.push(entry);
                    i += 1;
                    continue;
                }
            };

            let attrs = Fattr4 {
                attrmask: answer_attrs,
                attr_vals: attrs,
            };

            let entry = Entry4 {
                name: name.clone(),
                cookie: (i + 3) as u64, // Cookie values start from 3
                attrs,
                nextentry: None,
            };
            response_entries.push(entry);
            i += 1;
        }

        // If we processed all entries, eof should be true
        if i >= entries.len() {
            eof = true;
        }

        // Build the linked list of entries in reverse order
        let mut tnextentry: Option<Entry4> = None;
        for entry in response_entries.into_iter().rev() {
            let mut entry = entry;
            entry.nextentry = if tnextentry.is_some() {
                Some(Box::new(tnextentry.unwrap()))
            } else {
                None
            };
            tnextentry = Some(entry);
        }

        NfsOpResponse {
            request,
            result: Some(NfsResOp4::Opreaddir(ReadDir4res::Resok4(ReadDir4resok {
                reply: DirList4 {
                    entries: tnextentry,
                    eof,
                },
                cookieverf,
            }))),
            status: NfsStat4::Nfs4Ok,
        }
    }
}