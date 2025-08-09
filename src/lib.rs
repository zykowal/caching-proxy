use core::{
    cache::mem_cache::MemoryCache,
    cmd_args::config::{Command, Config},
    server::proxy_server::ProxyServer,
};
use std::{error::Error, time::Duration};

mod core;

pub async fn run<T>(args: T) -> Result<(), Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    let config = Config::parse_args(args)?;

    match config.command {
        Command::Start { port, origin } => ProxyServer::start(port, origin).await,
        Command::ClearCache => {
            let cache = MemoryCache::new(Duration::from_secs(60), 100);
            println!("Current cache size: {}", cache.size());
            cache.clear();
            println!("Cache clearing completed.");
            Ok(())
        }
    }
}
