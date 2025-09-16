use async_trait::async_trait;
use tracing::debug;

use crate::server::{operation::NfsOperation, request::NfsRequest, response::NfsOpResponse};

use bold_proto::nfs4_proto::{Close4args, Close4res, NfsResOp4, NfsStat4};

#[async_trait]
impl NfsOperation for Close4args {
    async fn execute<'a>(&self, mut request: NfsRequest<'a>) -> NfsOpResponse<'a> {
        debug!(
            "Operation 4: CLOSE - Close File {:?}, with request {:?}",
            self, request
        );

        let fmanager = request.file_manager();
        let result = fmanager.close_file(self.open_stateid.other).await;

        let current_filehandle = request.current_filehandle().unwrap();
        request.drop_filehandle_from_cache(current_filehandle.id);

        match result {
            Ok(_) => NfsOpResponse {
                request,
                result: Some(NfsResOp4::Opclose(Close4res::OpenStateid(
                    self.open_stateid.clone(),
                ))),
                status: NfsStat4::Nfs4Ok,
            },
            Err(e) => NfsOpResponse {
                request,
                result: None,
                status: e.nfs_error,
            },
        }
    }
}
