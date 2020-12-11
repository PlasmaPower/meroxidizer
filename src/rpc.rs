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

#[derive(Serialize)]
struct FullRequest<'a, P> {
    #[serde(rename = "jsonrpc")]
    json_rpc: &'static str,
    id: i64,
    method: &'a str,
    params: P,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FullResponse<R> {
    Error { error: RpcError },
    Result { id: i64, result: R },
}

#[derive(Debug)]
pub struct RpcMiningTarget {
    pub id: i64,
    pub key: [u8; 32],
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

    pub fn get_mining_target(&mut self, miner_pubkey: &str) -> RpcMiningTarget {
        let req1 = FullRequest {
            json_rpc: "2.0",
            id: 0,
            method: "merit_getDifficulty",
            params: [(); 0],
        };
        let req2 = FullRequest {
            json_rpc: "2.0",
            id: 1,
            method: "merit_getBlockTemplate",
            params: [miner_pubkey],
        };
        #[derive(Deserialize)]
        struct RpcBlockTemplate {
            id: i64,
            key: String,
            header: String,
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Response {
            Difficulty(String),
            BlockTemplate(RpcBlockTemplate),
        }
        self.with_retry(|rpc| {
            serde_json::to_writer(&mut rpc.writer, &req1)?;
            serde_json::to_writer(&mut rpc.writer, &req2)?;
            let mut difficulty = None;
            let mut template = None;
            for _ in 0..2 {
                match FullResponse::<Response>::deserialize(&mut rpc.reader)? {
                    FullResponse::Error { error, .. } => {
                        eyre::bail!("Got error from RPC: {:?}", error)
                    }
                    FullResponse::Result { result, id } => match result {
                        Response::Difficulty(s) => {
                            eyre::ensure!(
                                id == 0,
                                "Got difficulty RPC response with wrong ID {}",
                                id
                            );
                            difficulty = Some(u64::from_str_radix(&s, 16)?);
                        }
                        Response::BlockTemplate(t) => {
                            eyre::ensure!(
                                id == 1,
                                "Got template RPC response with wrong ID {}",
                                id
                            );
                            template = Some(t);
                        }
                    },
                }
            }
            match (difficulty, template) {
                (Some(difficulty), Some(template)) => {
                    let mut key = [0u8; 32];
                    hex::decode_to_slice(&template.key, &mut key)?;
                    Ok(RpcMiningTarget {
                        id: template.id,
                        key,
                        header: hex::decode(template.header)?,
                        difficulty,
                    })
                }
                _ => eyre::bail!("Got wrong combo of RPC responses for difficulty and template"),
            }
        })
    }
}
