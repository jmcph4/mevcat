use std::net::SocketAddr;

use clap::Parser;
use reqwest::Url;

use crate::Method;

/// Lightweight tool for interacting with MEV APIs
#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
pub struct Opts {
    /// Start an MEV server instance
    #[clap(long, short, action)]
    pub listen: Option<SocketAddr>,
    ///Port number to use
    #[clap(long, short)]
    pub port: Option<u16>,
    /// URL suffix to bind to
    #[clap(long, short)]
    pub suffix: Option<String>,
    #[clap(required_unless_present = "listen")]
    /// Remote host to submit RPCs to
    pub endpoint: Option<Url>,
    /// RPC verb to use (client-mode only)
    #[clap(long, short, action)]
    pub method: Option<Method>,
    /// Send standard input to the remote host
    #[clap(long, short, action)]
    pub raw: bool,
}
