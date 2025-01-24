use std::net::SocketAddr;

use clap::Parser;

const DEFAULT_BIND_SOCK: &str = "0.0.0.0:3000";

#[derive(Clone, Debug, Parser)]
pub struct Opts {
    #[clap(long, short, action, default_value=DEFAULT_BIND_SOCK)]
    pub listen: SocketAddr,
}
