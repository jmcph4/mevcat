use std::net::SocketAddr;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Opts {
    #[clap(long, short, action)]
    pub listen: Option<SocketAddr>,
    #[clap(long, short)]
    pub port: Option<u16>,
}
