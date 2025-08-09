use core::cmd_args::config::Config;
use std::error::Error;

mod core;

pub fn run<T>(args: T) -> Result<(), Box<dyn Error>>
where
    T: Iterator<Item = String>,
{
    let config = Config::parse(args)?;
    println!("{config:?}");

    Ok(())
}
