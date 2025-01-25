use std::{fmt::Display, str::FromStr};

use ajj::Router;
use alloy_primitives::FixedBytes;
use alloy_rpc_types_mev::{
    CancelBundleRequest, EthBundleHash, EthCallBundle, EthSendBundle,
    PrivateTransactionRequest,
};

use clap::Parser;
use log::info;
use reqwest::Client;

use crate::cli::Opts;

pub mod cli;

#[derive(Copy, Clone, Debug)]
pub enum Method {
    SendBundle,
    CancelBundle,
    CallBundle,
    SendPrivateTransaction,
    CancelPrivateTransaction,
    SendPrivateRawTransaction,
}

impl Default for Method {
    fn default() -> Self {
        Self::SendBundle
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::SendBundle => write!(f, "eth_sendBundle"),
            Method::CancelBundle => write!(f, "eth_cancelBundle"),
            Method::CallBundle => write!(f, "eth_callBundle"),
            Method::SendPrivateTransaction => {
                write!(f, "eth_sendPrivateTransaction")
            }
            Method::CancelPrivateTransaction => {
                write!(f, "eth_cancelPrivateTransaction")
            }
            Method::SendPrivateRawTransaction => {
                write!(f, "eth_cancelPrivateTransaction")
            }
        }
    }
}

impl FromStr for Method {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eth_sendBundle" => Ok(Self::SendBundle),
            "eth_cancelBundle" => Ok(Self::CancelBundle),
            "eth_callBundle" => Ok(Self::CallBundle),
            "eth_sendPrivateTransaction" => Ok(Self::SendPrivateTransaction),
            "eth_cancelPrivateTransaction" => {
                Ok(Self::CancelPrivateTransaction)
            }
            "eth_sendPrivateRawTransaction" => {
                Ok(Self::SendPrivateRawTransaction)
            }
            _ => Err("unsupported RPC"),
        }
    }
}

pub async fn send_rpc(opts: Opts) -> eyre::Result<()> {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let req = match opts.method.unwrap_or_default() {
        Method::SendBundle => {
            let bundle: EthSendBundle = serde_json::from_str(buf.as_str())?;
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_sendBundle\",\"params\":[{}]}}",
                serde_json::to_string(&bundle).unwrap(),
            )
        }
        Method::CancelBundle => {
            let cancel: CancelBundleRequest =
                serde_json::from_str(buf.as_str())?;
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_cancelBundle\",\"params\":[{}]}}",
                serde_json::to_string(&cancel).unwrap(),
            )
        }
        Method::CallBundle => {
            let bundle: EthCallBundle = serde_json::from_str(buf.as_str())?;
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_callBundle\",\"params\":[{}]}}",
                serde_json::to_string(&bundle).unwrap(),
            )
        }
        Method::SendPrivateTransaction => {
            let tx: PrivateTransactionRequest =
                serde_json::from_str(buf.as_str())?;
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_sendPrivateTransaction\",\"params\":[{}]}}",
                serde_json::to_string(&tx).unwrap(),
            )
        }
        _ => "".to_string(),
    };

    info!("Sending {}...", &req);
    let resp: String = Client::new()
        .post(opts.endpoint.unwrap())
        .header("Content-Type", "application/json")
        .body(req)
        .send()
        .await?
        .text()
        .await?;
    println!("{resp}");
    Ok(())
}

pub fn router() -> Router<()> {
    Router::new().route("eth_sendBundle", |bundle: EthSendBundle| async move {
        info!("Received bundle: {:?}", bundle);
        Ok::<EthBundleHash, ()>(EthBundleHash {
            bundle_hash: FixedBytes::ZERO,
        })
    })
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init_timed();
    let opts = Opts::parse();

    if let Some(sock) = opts.listen {
        let listener = tokio::net::TcpListener::bind(sock).await?;
        info!("Bound to {}", sock);
        axum::serve(
            listener,
            router().into_axum(&opts.suffix.unwrap_or("/".to_string())),
        )
        .await?;
    } else {
        send_rpc(opts).await?;
    }

    Ok(())
}
