use async_trait::async_trait;
use tracing::{debug, error};

use crate::server::{operation::NfsOperation, request::NfsRequest, response::NfsOpResponse};

use bold_proto::nfs4_proto::{
    Attrlist4, ChangeInfo4, CreateHow4, FileAttr, NfsResOp4, NfsStat4, Open4args, Open4res,
    Open4resok, OpenClaim4, OpenDelegation4, OpenFlag4, Stateid4, OPEN4_RESULT_CONFIRM,
};

use crate::server::filemanager::Filehandle;

async fn open_existing_file<'a>(
    _args: &Open4args,
    filehandle: Filehandle,
    file: String,
    mut request: NfsRequest<'a>,
) -> NfsOpResponse<'a> {
    let path = &filehandle.path;

    let fh_path = {
        if path == "/" {
            format!("{}{}", path, file)
        } else {
            format!("{}/{}", path, file)
        }
    };

    debug!("open_existing_file {:?}", fh_path);

    let filehandle = match request
        .file_manager()
        .get_filehandle_for_path(fh_path)
        .await
    {
        Ok(filehandle) => filehandle,
        Err(e) => {
            error!("Err {:?}", e);
            return NfsOpResponse {
                request,
                result: None,
                status: e.nfs_error,
            };
        }
    };

    request.set_filehandle(filehandle.clone());

    // For now, we'll create a simple response without actual locking
    // In a full implementation, we would need to implement proper file locking
    
    // Create a mock lock state for demonstration
    let mock_stateid = [0u8; 12];
    let mock_seqid = 0u32;

    NfsOpResponse {
        request,
        result: Some(NfsResOp4::Opopen(Open4res::Resok4(Open4resok {
            stateid: Stateid4 {
                seqid: mock_seqid,
                other: mock_stateid,
            },
            cinfo: ChangeInfo4 {
                atomic: false,
                before: 0,
                after: 0,
            },
            // OPEN4_RESULT_CONFIRM indicates that the client MUST execute an
            // OPEN_CONFIRM operation before using the open file.
            rflags: OPEN4_RESULT_CONFIRM,
            attrset: Attrlist4::<FileAttr>::new(None),
            delegation: OpenDelegation4::None,
        }))),
        status: NfsStat4::Nfs4Ok,
    }
}

async fn open_for_writing<'a>(
    args: &Open4args,
    filehandle: Filehandle,
    file: String,
    how: CreateHow4,
    mut request: NfsRequest<'a>,
) -> NfsOpResponse<'a> {
    let path = &filehandle.path;

    let fh_path = {
        if path == "/" {
            format!("{}{}", path, file)
        } else {
            format!("{}/{}", path, file)
        }
    };

    debug!("open_for_writing {:?}", fh_path);

    let newfile_op = filehandle.file.join(&file);

    let filehandle = match how {
        CreateHow4::UNCHECKED4(_fattr) => {
            match request
                .file_manager()
                .create_file(
                    newfile_op.unwrap(),
                    args.owner.clientid,
                    args.owner.owner.clone(),
                    args.share_access,
                    args.share_deny,
                    None,
                )
                .await
            {
                Ok(filehandle) => filehandle,
                Err(e) => {
                    error!("Err {:?}", e);
                    return NfsOpResponse {
                        request,
                        result: None,
                        status: NfsStat4::Nfs4errServerfault,
                    };
                }
            }
        }
        CreateHow4::GUARDED4(_fattr) => {
            match request
                .file_manager()
                .create_file(
                    newfile_op.unwrap(),
                    args.owner.clientid,
                    args.owner.owner.clone(),
                    args.share_access,
                    args.share_deny,
                    None,
                )
                .await
            {
                Ok(filehandle) => filehandle,
                Err(e) => {
                    error!("Err {:?}", e);
                    return NfsOpResponse {
                        request,
                        result: None,
                        status: NfsStat4::Nfs4errServerfault,
                    };
                }
            }
        }
        CreateHow4::EXCLUSIVE4(verifier) => {
            match request
                .file_manager()
                .create_file(
                    newfile_op.unwrap(),
                    args.owner.clientid,
                    args.owner.owner.clone(),
                    args.share_access,
                    args.share_deny,
                    Some(verifier),
                )
                .await
            {
                Ok(filehandle) => filehandle,
                Err(e) => {
                    error!("Err {:?}", e);
                    return NfsOpResponse {
                        request,
                        result: None,
                        status: NfsStat4::Nfs4errServerfault,
                    };
                }
            }
        }
    };

    request.set_filehandle(filehandle.clone());
    
    // Check if there are any locks on the filehandle
    
    // we expect this filehandle to have one lock (for the shared reservation)
    // For now, we'll just use the first lock if it exists, or create a mock one
    let mut mock_stateid = [0u8; 12];
    let mut mock_seqid = 0u32;
    
    if !filehandle.locks.is_empty() {
        let lock = &filehandle.locks[0];
        // Copy only the first 12 bytes, or pad with zeros if less than 12
        let len = std::cmp::min(lock.stateid.len(), 12);
        mock_stateid[..len].copy_from_slice(&lock.stateid[..len]);
        mock_seqid = lock.seqid;
    }

    NfsOpResponse {
        request,
        result: Some(NfsResOp4::Opopen(Open4res::Resok4(Open4resok {
            stateid: Stateid4 {
                seqid: mock_seqid,
                other: mock_stateid,
            },
            cinfo: ChangeInfo4 {
                atomic: false,
                before: 0,
                after: 0,
            },
            // OPEN4_RESULT_CONFIRM indicates that the client MUST execute an
            // OPEN_CONFIRM operation before using the open file.
            rflags: OPEN4_RESULT_CONFIRM,
            attrset: Attrlist4::<FileAttr>::new(None),
            delegation: OpenDelegation4::None,
        }))),
        status: NfsStat4::Nfs4Ok,
    }
}

#[async_trait]
impl NfsOperation for Open4args {
    async fn execute<'a>(&self, mut request: NfsRequest<'a>) -> NfsOpResponse<'a> {
        // Description: https://datatracker.ietf.org/doc/html/rfc7530#section-16.16.5
        debug!(
            "Operation 18: OPEN - Open a Regular File {:?}, with request {:?}",
            self, request
        );
        // open sets the current filehandle to the looked up filehandle
        let current_filehandle = request.current_filehandle();
        let filehandle = match current_filehandle {
            Some(filehandle) => filehandle.clone(),
            None => {
                error!("None filehandle");
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errFhexpired,
                };
            }
        };

        // If the current filehandle is not a directory, the error
        // NFS4ERR_NOTDIR will be returned.
        if !filehandle.file.is_dir().unwrap() {
            error!("Not a directory");
            return NfsOpResponse {
                request,
                result: None,
                status: NfsStat4::Nfs4errNotdir,
            };
        }

        let file = match &self.claim {
            // CLAIM_NULL:  For the client, this is a new OPEN request, and there is
            // no previous state associated with the file for the client.
            OpenClaim4::ClaimNull(file) => file.clone(),
            // NFS4ERR_NOTSUPP is returned if the server does not support this
            // claim type.
            _ => {
                error!("Unsupported OpenClaim4 {:?}", self.claim);
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errNotsupp,
                };
            }
        };

        // If the component is of zero length, NFS4ERR_INVAL will be returned.
        if file.is_empty() {
            error!("Zero length file name");
            return NfsOpResponse {
                request,
                result: None,
                status: NfsStat4::Nfs4errInval,
            };
        }

        let path = &filehandle.path;

        let fh_path = {
            if path == "/" {
                format!("{}{}", path, file)
            } else {
                format!("{}/{}", path, file)
            }
        };

        debug!("OPEN: looking for file {:?}", fh_path);

        // Check if the file already exists
        let exists = filehandle.file.join(&file).unwrap().exists().unwrap();

        if exists {
            // If GUARDED4 is specified, and the file already exists, return NFS4ERR_EXIST
            if let OpenFlag4::How(CreateHow4::GUARDED4(_)) = &self.openhow {
                error!("File already exists with GUARDED4");
                return NfsOpResponse {
                    request,
                    result: None,
                    status: NfsStat4::Nfs4errExist,
                };
            }
            open_existing_file(self, filehandle, file, request).await
        } else {
            // If the file doesn't exist, we need to create it
            // But only if the openhow allows creation
            match &self.openhow {
                OpenFlag4::How(create_how) => {
                    open_for_writing(self, filehandle, file, create_how.clone(), request).await
                }
                OpenFlag4::Open4Nocreate => {
                    error!("File does not exist and creation not allowed");
                    NfsOpResponse {
                        request,
                        result: None,
                        status: NfsStat4::Nfs4errNoent,
                    }
                }
            }
        }
    }
}