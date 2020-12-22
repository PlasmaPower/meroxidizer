use crate::cli::Opts;
use eyre::Report;
use log::error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{de::IoRead, Deserializer};
use std::{fmt, io, net::TcpStream, time::Duration};

const CONNECT_BACKOFF: Duration = Duration::from_secs(1);
const RETRY_BACKOFF: Duration = Duration::from_secs(1);

#[derive(Deserialize, Debug)]
pub struct RpcError {
    pub code: i64,
    #[serde(default)]
    pub message: Option<String>,
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code {}", self.code)?;
        if let Some(message) = &self.message {
            write!(f, " message {}", message)?;
        }
        Ok(())
    }
}

impl std::error::Error for RpcError {}

#[derive(Serialize)]
struct FullRequest<'a, P> {
    #[serde(rename = "jsonrpc")]
    json_rpc: &'static str,
    id: i64,
    method: &'a str,
    params: P,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(untagged)]
enum FullResponse<R> {
    Error { error: RpcError },
    Result { id: i64, result: R },
}

#[derive(Debug, Deserialize)]
pub struct RpcMiningTarget {
    pub id: i64,
    #[serde(with = "hex")]
    pub key: [u8; 32],
    #[serde(with = "hex")]
    pub header: Vec<u8>,
    pub difficulty: u64,
}

pub struct Rpc {
    opts: Opts,
    writer: TcpStream,
    reader: Deserializer<IoRead<TcpStream>>,
}

impl Rpc {
    fn try_connect(opts: Opts) -> Result<Rpc, io::Error> {
        let stream = TcpStream::connect(&opts.rpc)?;
        let reader = stream.try_clone()?;
        let reader = Deserializer::from_reader(reader);
        Ok(Rpc {
            writer: stream,
            reader,
            opts,
        })
    }

    pub fn connect(opts: Opts) -> Rpc {
        loop {
            match Self::try_connect(opts.clone()) {
                Ok(r) => return r,
                Err(err) => {
                    error!("error connecting to RPC: {}", err);
                }
            }
            std::thread::sleep(CONNECT_BACKOFF);
        }
    }

    fn with_retry<F, O>(&mut self, mut f: F) -> O
    where
        F: FnMut(&mut Self) -> Result<O, Report>,
    {
        loop {
            match f(self) {
                Ok(x) => return x,
                Err(err) => {
                    error!("error making RPC call: {:#}", err);
                }
            }
            *self = Rpc::connect(self.opts.clone());
            std::thread::sleep(RETRY_BACKOFF);
        }
    }

    pub fn single_request<P: Serialize, R: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, RpcError> {
        let req = FullRequest {
            json_rpc: "2.0",
            method: method,
            id: 0,
            params,
        };
        self.with_retry(|rpc| {
            serde_json::to_writer(&mut rpc.writer, &req)?;
            match FullResponse::<R>::deserialize(&mut rpc.reader)? {
                FullResponse::Error { error, .. } => Ok(Err(error)),
                FullResponse::Result { result, .. } => Ok(Ok(result)),
            }
        })
    }

    pub fn get_height(&mut self) -> usize {
        let req = FullRequest {
            json_rpc: "2.0",
            method: "merit_getHeight",
            id: 0,
            params: [(); 0],
        };
        self.with_retry(|rpc| {
            serde_json::to_writer(&mut rpc.writer, &req)?;
            match FullResponse::deserialize(&mut rpc.reader)? {
                FullResponse::Error { error, .. } => Err(error.into()),
                FullResponse::Result { result, .. } => Ok(result),
            }
        })
    }

    pub fn get_mining_target(&mut self, miner_pubkey: &str) -> RpcMiningTarget {
        let req = FullRequest {
            json_rpc: "2.0",
            id: 1,
            method: "merit_getBlockTemplate",
            params: [miner_pubkey],
        };
        self.with_retry(|rpc| {
            serde_json::to_writer(&mut rpc.writer, &req)?;
            match FullResponse::deserialize(&mut rpc.reader)? {
                FullResponse::Error { error, .. } => Err(error.into()),
                FullResponse::Result { result, .. } => Ok(result)
            }
        })
    }
}
