use std::hash::Hasher;

use async_trait::async_trait;
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
        request.file_manager().touch_file(dir_fh.id).await;
        let dir = dir_fh.file.read_dir().unwrap();
        let dir_count = dir.count();

        let mut entries = Vec::new();
        let dir = dir_fh.file.read_dir().unwrap();
        for (i, entry) in dir.enumerate() {
            let filehandle = request
                .file_manager()
                .get_filehandle_for_path(entry.as_str().to_string())
                .await;
            if let Ok(filehandle) = filehandle {
                request.file_manager().touch_file(filehandle.id).await;
                let updated_fh = request
                    .file_manager()
                    .get_filehandle_for_id(filehandle.id)
                    .await
                    .unwrap();
                entries.push((i + 3, updated_fh));
            }
        }

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for (_, fh) in entries.iter() {
            hasher.write(fh.file.filename().as_bytes());
        }
        let mut cookieverf = hasher.finish().to_be_bytes().to_vec();
        if self.cookie != 0 && cookieverf != self.cookieverf {
            error!("Nfs4errNotSame");
            return NfsOpResponse {
                request,
                result: None,
                status: NfsStat4::Nfs4errNotSame,
            };
        }

        // if this directory is empty, we can't create a cookie verifier based on the dir contents
        // setting it to a default value
        if cookieverf.is_empty() {
            cookieverf = [0u8; 8].to_vec();
        } else if cookieverf.len() < 8 {
            let mut diff = 8 - cookieverf.len();
            while diff > 0 {
                cookieverf.push(0);
                diff -= 1;
            }
        }

        let mut tnextentry = None;
        let mut added_entries = 0;
        for (cookie, fh) in entries.into_iter().rev() {
            let resp = request
                .file_manager()
                .filehandle_attrs(&self.attr_request, &fh);
            let (answer_attrs, attrs) = match resp {
                Some(inner) => inner,
                None => {
                    return NfsOpResponse {
                        request,
                        result: None,
                        status: NfsStat4::Nfs4errServerfault,
                    };
                }
            };

            let entry = Entry4 {
                name: fh.file.filename(),
                cookie: cookie as u64,
                attrs: Fattr4 {
                    attrmask: answer_attrs,
                    attr_vals: attrs,
                },
                nextentry: if tnextentry.is_some() {
                    Some(Box::new(tnextentry.unwrap()))
                } else {
                    None
                },
            };
            added_entries += 1;
            tnextentry = Some(entry);
        }
        let eof = {
            if tnextentry.is_some()
                && (tnextentry.clone().unwrap().cookie as usize + added_entries) >= dir_count
            {
                true
            } else {
                tnextentry.is_none()
            }
        };

        NfsOpResponse {
            request,
            result: Some(NfsResOp4::Opreaddir(ReadDir4res::Resok4(ReadDir4resok {
                reply: DirList4 {
                    // len: if tnextentry.is_some() { 1 } else { 0 },
                    entries: tnextentry.clone(),
                    eof,
                },
                cookieverf: cookieverf.as_slice().try_into().unwrap(),
            }))),
            status: NfsStat4::Nfs4Ok,
        }
    }
}

#[cfg(test)]
mod integration_tests {

    use bold_proto::nfs4_proto::Attrlist4;
    use tracing_test::traced_test;

    use crate::{
        server::{
            nfs40::{
                DirList4, FileAttr, FileAttrValue, NfsFtype4, NfsResOp4, NfsStat4, PutFh4args,
                ReadDir4res, ReadDir4resok, Readdir4args,
            },
            operation::NfsOperation,
        },
        test_utils::{create_fake_fs, create_nfs40_server},
    };

    #[tokio::test]
    #[traced_test]
    async fn test_read_directory() {
        // dummy fs, empty
        let request = create_nfs40_server(None).await;
        let fh = request.file_manager().get_root_filehandle().await;

        let putfh_args = PutFh4args {
            object: fh.unwrap().id,
        };
        let putfh_request = putfh_args.execute(request).await;

        let readdir_args = Readdir4args {
            cookie: 0,
            cookieverf: [0u8; 8],
            dircount: 262122,
            maxcount: 1048488,
            attr_request: Attrlist4::<FileAttr>::new(Some(vec![
                FileAttr::Type,
                FileAttr::Change,
                FileAttr::Size,
                FileAttr::Fsid,
                FileAttr::RdattrError,
                FileAttr::Filehandle,
                FileAttr::Fileid,
                FileAttr::Mode,
                FileAttr::Numlinks,
                FileAttr::Owner,
                FileAttr::OwnerGroup,
                FileAttr::Rawdev,
                FileAttr::SpaceUsed,
                FileAttr::TimeAccess,
                FileAttr::TimeMetadata,
                FileAttr::TimeModify,
                FileAttr::MountedOnFileid,
            ])),
        };

        let readdir_response = readdir_args.execute(putfh_request.request).await;
        assert_eq!(readdir_response.status, NfsStat4::Nfs4Ok);
        assert_eq!(
            readdir_response.result,
            Some(NfsResOp4::Opreaddir(ReadDir4res::Resok4(ReadDir4resok {
                cookieverf: [0, 0, 0, 0, 0, 0, 0, 0],
                reply: DirList4 {
                    entries: None,
                    eof: true
                }
            })))
        );

        // a more filled directory, still eof = true

        let request = create_nfs40_server(Some(create_fake_fs())).await;
        let fh = request.file_manager().get_root_filehandle().await;

        let putfh_args = PutFh4args {
            object: fh.unwrap().id,
        };
        let putfh_request = putfh_args.execute(request).await;

        let readdir_args = Readdir4args {
            cookie: 0,
            cookieverf: [0u8; 8],
            dircount: 262122,
            maxcount: 1048488,
            attr_request: Attrlist4::<FileAttr>::new(Some(vec![
                FileAttr::Type,
                FileAttr::Change,
                FileAttr::Size,
                FileAttr::Fsid,
                FileAttr::RdattrError,
                FileAttr::Filehandle,
                FileAttr::Fileid,
                FileAttr::Mode,
                FileAttr::Numlinks,
                FileAttr::Owner,
                FileAttr::OwnerGroup,
                FileAttr::Rawdev,
                FileAttr::SpaceUsed,
                FileAttr::TimeAccess,
                FileAttr::TimeMetadata,
                FileAttr::TimeModify,
                FileAttr::MountedOnFileid,
            ])),
        };

        let readdir_response = readdir_args.execute(putfh_request.request).await;
        assert_eq!(readdir_response.status, NfsStat4::Nfs4Ok);
        let result = readdir_response.result.unwrap();
        match result {
            NfsResOp4::Opreaddir(ReadDir4res::Resok4(res)) => {
                assert_eq!(res.cookieverf.len(), 8);
                let entries = res.reply.entries.unwrap();
                assert_eq!(entries.cookie, 3);
                if entries.name == "file1.txt" {
                    assert_eq!(entries.attrs.attrmask.len(), 14);
                    assert_eq!(entries.attrs.attr_vals.len(), 14);
                    assert_eq!(
                        entries.attrs.attr_vals[0],
                        FileAttrValue::Type(NfsFtype4::Nf4reg)
                    );
                } else if entries.name == "dir1" {
                    assert_eq!(entries.attrs.attrmask.len(), 14);
                    assert_eq!(entries.attrs.attr_vals.len(), 14);
                    assert_eq!(
                        entries.attrs.attr_vals[0],
                        FileAttrValue::Type(NfsFtype4::Nf4dir)
                    );
                } else {
                    panic!("Unexpected entry");
                }
                let next = entries.nextentry.unwrap();
                assert_eq!(next.cookie, 4);
                if next.name == "file1.txt" {
                    assert_eq!(next.attrs.attrmask.len(), 14);
                    assert_eq!(next.attrs.attr_vals.len(), 14);
                    assert_eq!(
                        next.attrs.attr_vals[0],
                        FileAttrValue::Type(NfsFtype4::Nf4reg)
                    );
                } else if next.name == "dir1" {
                    assert_eq!(next.attrs.attrmask.len(), 14);
                    assert_eq!(next.attrs.attr_vals.len(), 14);
                    assert_eq!(
                        next.attrs.attr_vals[0],
                        FileAttrValue::Type(NfsFtype4::Nf4dir)
                    );
                } else {
                    panic!("Unexpected entry");
                }
                assert_eq!(next.nextentry, None);
                assert!(res.reply.eof);
            }
            _ => panic!("Expected Resok4"),
        }
    }
}
