use std::io::SeekFrom;

use async_trait::async_trait;
use tracing::{debug, error};

use crate::server::{operation::NfsOperation, request::NfsRequest, response::NfsOpResponse};
use bold_proto::nfs4_proto::{NfsResOp4, NfsStat4, Read4args, Read4res, Read4resok};

#[async_trait]
impl NfsOperation for Read4args {
    async fn execute<'a>(&self, request: NfsRequest<'a>) -> NfsOpResponse<'a> {
        debug!(
            "Operation 25: READ - Read from File {:?}, with request {:?}",
            self, request
        );
        let current_filehandle = request.current_filehandle();
        let filehandle = match current_filehandle {
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

        // Проверим, существует ли файл
        if !filehandle.file.exists().unwrap() {
            return NfsOpResponse {
                request,
                result: Some(NfsResOp4::Opread(Read4res::Resfail)),
                status: NfsStat4::Nfs4errNoent, // Файл не существует
            };
        }
        
        // Проверим, является ли объект файлом
        if filehandle.file.is_dir().unwrap() {
            return NfsOpResponse {
                request,
                result: Some(NfsResOp4::Opread(Read4res::Resfail)),
                status: NfsStat4::Nfs4errIsdir, // Это каталог, а не файл
            };
        }

        let mut buffer: Vec<u8> = vec![0; self.count as usize];
        let mut rfile = match filehandle.file.open_file() {
            Ok(file) => file,
            Err(_) => {
                return NfsOpResponse {
                    request,
                    result: Some(NfsResOp4::Opread(Read4res::Resfail)),
                    status: NfsStat4::Nfs4errAccess, // Нет доступа к файлу
                };
            }
        };
        
        // Проверим, что смещение не превышает размер файла
        let file_len = filehandle.file.metadata().unwrap().len;
        if self.offset > file_len {
            return NfsOpResponse {
                request,
                result: Some(NfsResOp4::Opread(Read4res::Resok4(Read4resok {
                    eof: true,
                    data: vec![],
                }))),
                status: NfsStat4::Nfs4Ok,
            };
        }
        
        rfile.seek(SeekFrom::Start(self.offset)).unwrap();
        let read_result = rfile.read_exact(&mut buffer);
        
        // Если не удалось прочитать данные, возможно, мы достигли конца файла
        let (data, eof) = match read_result {
            Ok(_) => {
                (buffer, true)
            },
            Err(_) => {
                // Попробуем прочитать доступные данные
                let available = (file_len - self.offset) as usize;
                if available > 0 {
                    buffer.resize(available, 0);
                    rfile.seek(SeekFrom::Start(self.offset)).unwrap();
                    match rfile.read_exact(&mut buffer).await {
                        Ok(_) => (buffer, true),
                        Err(_) => (vec![], true)
                    }
                } else {
                    (vec![], true)
                }
            }
        };

        NfsOpResponse {
            request,
            result: Some(NfsResOp4::Opread(Read4res::Resok4(Read4resok {
                eof,
                data,
            }))),
            status: NfsStat4::Nfs4Ok,
        }
    }
}
