use std::{fmt::Display, str::FromStr};

use ajj::Router;
use alloy_primitives::FixedBytes;
use alloy_rpc_types_mev::{
    CancelBundleRequest, CancelPrivateTransactionRequest, EthBundleHash,
    EthCallBundle, EthSendBundle, PrivateTransactionRequest,
};

use clap::Parser;
use log::info;
use reqwest::Client;
use serde_json::Value;

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

    if buf.is_empty() {
        return Ok(());
    }

    let req = if opts.raw {
        buf
    } else {
        match opts.method.unwrap_or_default() {
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
            Method::CancelPrivateTransaction => {
                let cancel: CancelPrivateTransactionRequest =
                    serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_cancelPrivateTransaction\",\"params\":[{}]}}",
                serde_json::to_string(&cancel).unwrap(),
            )
            }
            Method::SendPrivateRawTransaction => {
                let tx: PrivateTransactionRequest =
                    serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_sendPrivateRawTransaction\",\"params\":[{}]}}",
                serde_json::to_string(&tx).unwrap(),
            )
            }
        }
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
    if opts.raw {
        println!("{resp}");
    } else if let Some(res) = serde_json::from_str(&resp)
        .ok()
        .and_then(|v: Value| v.get("result").cloned())
    {
        println!("{res}");
    }
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

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};

    const BEAVERBUILD_URL: &str = "https://rpc.beaverbuild.org";
    const LOCAL_SOCKET: &str = "0.0.0.0:3000";

    #[test]
    fn test_eof() {
        let mut child = Command::new("target/debug/mevcat")
            .arg(BEAVERBUILD_URL)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start the program");

        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        let _ = stdin;

        let output = child.wait().expect("Failed to wait on child");

        assert!(output.success(), "Program did not exit with code 0");
    }

    #[test]
    fn test_port_with_listen() {
        let some_port: u16 = 8080;
        let mut child = Command::new("target/debug/mevcat")
            .arg("-l")
            .arg(LOCAL_SOCKET)
            .arg("-p")
            .arg(some_port.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start the program");

        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        let _ = stdin;

        let output = child.wait().expect("Failed to wait on child");

        assert!(!output.success(), "Program should error");
        assert!(
            output.code().is_some(),
            "Program should return an exit code"
        );
        assert_eq!(
            output.code().unwrap(),
            2,
            "Program should return exit code 2"
        );
    }
}
