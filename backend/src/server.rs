use bytes::Bytes;
use quinn::Endpoint;
use verita_protocol_lib::{
    auth::LoginResponse,
    service::{VeritaError, VeritaErrorCode, VeritaRequest, VeritaResponse, VeritaRpcServer},
    sync::Pipe,
    timestamp,
};

pub struct VeritaServer {
    pipe: Pipe<Bytes>,
}

impl VeritaServer {
    async fn run(endpoint: Endpoint, pipe: Pipe<Bytes>) {
        while let Some(conn) = endpoint.accept().await {
            let Ok(conn) = conn.await else {
                break;
            };
            loop {
                let stream = conn.accept_bi().await;
                let (mut send_stream, mut recv_stream) = match stream {
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                        break;
                    }
                    Err(e) => {
                        println!("Err {e:?}");
                        break;
                    }
                    Ok(s) => s,
                };
                if let Ok(data) = recv_stream.read_to_end(usize::MAX).await
                    && let Ok(response) = pipe.send(Bytes::from_owner(data)).await
                    && let Ok(_) = send_stream.write_all(&response).await
                    && let Ok(_) = send_stream.finish()
                {
                } else {
                    continue;
                }
            }
        }
    }

    pub fn new(endpoint: Endpoint) -> Self {
        let (server_pipe, quinn_pipe) = Pipe::new();
        tokio::spawn(async move { Self::run(endpoint, quinn_pipe).await });
        Self { pipe: server_pipe }
    }

    pub async fn init(&mut self) {
        while let Ok(data) = self.pipe.recv().await {
            let Ok(req) = rkyv::from_bytes::<VeritaRequest, rkyv::rancor::Error>(&data) else {
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&VeritaResponse::Err(VeritaError {
                    code: VeritaErrorCode::CouldNotRespond,
                    timestamp: timestamp(),
                    details: "Could not serialize the request properly. Check if the sent content really is serializable".to_string()
                })).map(|v| v.into_vec()).unwrap_or(b"Could not send data".to_vec());
                let _ = self.pipe.send(Bytes::from_owner(bytes)).await;
                continue;
            };
            let res = match req {
                VeritaRequest::REGISTER(req) => self.handle_register(req),
            }
            .await;
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&res)
                .map(|v| v.into_vec())
                .unwrap_or(b"Could not send data".to_vec());
            let _ = self.pipe.send(Bytes::from_owner(bytes)).await;
        }
    }
}

#[async_trait::async_trait]
impl VeritaRpcServer for VeritaServer {
    type Err = VeritaError;
    async fn handle_register(
        &mut self,
        request: verita_protocol_lib::auth::UserRegistrationRequest,
    ) -> Result<LoginResponse, Self::Err> {
        println!("Recv request {request:?}");
        Ok(LoginResponse {})
    }
}
