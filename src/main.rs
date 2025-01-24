use ajj::Router;
use alloy_primitives::FixedBytes;
use alloy_rpc_types_mev::{EthBundleHash, EthSendBundle};

use clap::Parser;
use log::info;

use crate::cli::Opts;

pub mod cli;

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

    let listener = tokio::net::TcpListener::bind(opts.address).await?;
    info!("Bound to {}", opts.address);
    axum::serve(listener, router().into_axum("/rpc")).await?;

    Ok(())
}
