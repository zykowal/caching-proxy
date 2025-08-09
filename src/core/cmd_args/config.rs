use std::error::Error;

#[derive(Debug)]
pub enum Command {
    Start { port: u16, origin: String },
    ClearCache,
}

#[derive(Debug)]
pub struct Config {
    pub command: Command,
}

impl Config {
    pub fn parse_args<T>(mut args: T) -> Result<Self, Box<dyn Error>>
    where
        T: Iterator<Item = String>,
    {
        args.next();

        let mut port = 3000;
        let mut origin = String::new();
        let mut clear_cache = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" | "-p" => {
                    port = args.next().ok_or("port is required")?.parse::<u16>()?;
                }
                "--origin" | "-o" => {
                    origin = args.next().ok_or("origin is required")?;
                }
                "--clear-cache" => {
                    clear_cache = true;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {arg}");
                    print_help();
                    std::process::exit(1);
                }
            }
        }

        let command = if clear_cache {
            Command::ClearCache
        } else {
            if origin.is_empty() {
                return Err("origin is required when starting server".into());
            }
            Command::Start { port, origin }
        };

        Ok(Self { command })
    }
}

fn print_help() {
    println!("Usage: caching-proxy [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -p, --port <PORT>      The port on which the caching proxy server will run");
    println!("  -o, --origin <URL>     The URL of the server to which requests will be forwarded");
    println!("      --clear-cache      Clear the cache and exit");
    println!("  -h, --help             Print help");
    println!();
}
