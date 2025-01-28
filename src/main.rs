use std::{fmt::Display, str::FromStr};

use ajj::Router;
use alloy_primitives::{FixedBytes, TxHash};
use alloy_rpc_types_mev::{
    CancelBundleRequest, CancelPrivateTransactionRequest, EthBundleHash,
    EthCallBundle, EthCallBundleResponse, EthSendBundle,
    PrivateTransactionRequest,
};
use clap::Parser;
use log::{info, warn};
use reqwest::{Client, Url};
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
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[{}]}}",
                opts.method.unwrap_or_default(),
                serde_json::to_string(&bundle).unwrap(),
            )
            }
            Method::CancelBundle => {
                let cancel: CancelBundleRequest =
                    serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[{}]}}",
                opts.method.unwrap(),
                serde_json::to_string(&cancel).unwrap(),
            )
            }
            Method::CallBundle => {
                let bundle: EthCallBundle = serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[{}]}}",
                opts.method.unwrap(),
                serde_json::to_string(&bundle).unwrap(),
            )
            }
            Method::SendPrivateTransaction => {
                let tx: PrivateTransactionRequest =
                    serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[{}]}}",
                opts.method.unwrap(),
                serde_json::to_string(&tx).unwrap(),
            )
            }
            Method::CancelPrivateTransaction => {
                let cancel: CancelPrivateTransactionRequest =
                    serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[{}]}}",
                opts.method.unwrap(),
                serde_json::to_string(&cancel).unwrap(),
            )
            }
            Method::SendPrivateRawTransaction => {
                let tx: PrivateTransactionRequest =
                    serde_json::from_str(buf.as_str())?;
                format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[{}]}}",
                opts.method.unwrap(),
                serde_json::to_string(&tx).unwrap(),
            )
            }
        }
    };

    let url: Url = match opts.port {
        Some(p) => {
            let mut x = opts.endpoint.unwrap();
            x.set_port(Some(p)).unwrap();
            x
        }
        None => opts.endpoint.unwrap(),
    };

    info!("Sending {} to {}...", &req, &url);
    let resp: String = Client::new()
        .post(url)
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
    Router::new()
        .route("eth_sendBundle", |params: Vec<EthSendBundle>| async move {
            if let Some(bundle) = params.first() {
                info!("Received bundle: {:?}", bundle);
                Ok::<EthBundleHash, &'static str>(EthBundleHash {
                    bundle_hash: FixedBytes::ZERO,
                })
            } else {
                warn!("Received eth_sendBundle with no bundles");
                Err("Must specify exactly one bundle")
            }
        })
        .route(
            "eth_cancelBundle",
            |cancel: CancelBundleRequest| async move {
                info!(
                    "Received cancellation for bundle: {}",
                    cancel.bundle_hash
                );
                Ok::<(), &'static str>(())
            },
        )
        .route("eth_callBundle", |bundle: EthCallBundle| async move {
            info!("Received bundle for simulation: {:?}", bundle);
            let sim_resp: EthCallBundleResponse =
                EthCallBundleResponse::default();
            Ok::<EthCallBundleResponse, &'static str>(sim_resp)
        })
        .route(
            "eth_sendPrivateTransaction",
            |params: Vec<PrivateTransactionRequest>| async move {
                if let Some(bundle) = params.first() {
                    info!("Received private transaction request: {:?}", bundle);
                    Ok::<TxHash, &'static str>(TxHash::ZERO)
                } else {
                    warn!("Received eth_sendBundle with no bundles");
                    Err("Must specify exactly one bundle")
                }
            },
        )
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
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
    };

    use serde_json::Value;

    const BEAVERBUILD_URL: &str = "https://rpc.beaverbuild.org";
    const LOCAL_SOCKET: &str = "0.0.0.0:3000";
    const BEAVER_BUNDLE_EXAMPLE: &str = r#"{"txs":["0x02f8b20181948449bdee618501dcd6500083016b93942dabcea55a12d73191aece59f508b191fb68adac80b844095ea7b300000000000000000000000054e44dbb92dba848ace27f44c0cb4268981ef1cc00000000000000000000000000000000000000000000000052616e065f6915ebc080a0c497b6e53d7cb78e68c37f6186c8bb9e1b8a55c3e22462163495979b25c2caafa052769811779f438b73159c4cc6a05a889da8c1a16e432c2e37e3415c9a0b9887"],"blockNumber":"0x1361bd3"}"#;

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

    #[test]
    fn test_send_bundle() {
        let mut child = Command::new("target/debug/mevcat")
            .arg(BEAVERBUILD_URL)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start the program");

        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        assert!(writeln!(stdin, "{}", BEAVER_BUNDLE_EXAMPLE).is_ok());
        let _ = stdin;

        let output = child.wait().expect("Failed to wait on child");
        dbg!(&output);

        assert!(output.success(), "Program should not error");

        let mut output = String::new();
        assert!(child.stdout.unwrap().read_to_string(&mut output).is_ok());
        let perhaps_val = serde_json::from_str(&output)
            .ok()
            .and_then(|v: Value| v.get("bundleHash").cloned());
        assert!(
            perhaps_val.is_some(),
            "Remote host should return a valid bundle hash object"
        );
    }
}
