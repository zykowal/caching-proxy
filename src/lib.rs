use core::{cmd_args::config::Config, server::proxy_server::ProxyServer};
use std::error::Error;

mod core;

pub async fn run<T>(args: T) -> Result<(), Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    ProxyServer::start(Config::parse_args(args)?).await
}
