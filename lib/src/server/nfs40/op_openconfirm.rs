use async_trait::async_trait;
use tracing::debug;

use crate::server::{operation::NfsOperation, request::NfsRequest, response::NfsOpResponse};

use bold_proto::nfs4_proto::{
    NfsResOp4, NfsStat4, OpenConfirm4args, OpenConfirm4res, OpenConfirm4resok,
};

#[async_trait]
impl NfsOperation for OpenConfirm4args {
    async fn execute<'a>(&self, request: NfsRequest<'a>) -> NfsOpResponse<'a> {
        debug!(
            "Operation 20: OPEN_CONFIRM - Confirm Open {:?}, with request {:?}",
            self, request
        );

        let fmanager = request.file_manager();
        let result = fmanager.confirm_lock(self.open_stateid.other).await;

        match result {
            Ok(_) => NfsOpResponse {
                request,
                result: Some(NfsResOp4::OpopenConfirm(OpenConfirm4res::Resok4(
                    OpenConfirm4resok {
                        open_stateid: self.open_stateid.clone(),
                    },
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
