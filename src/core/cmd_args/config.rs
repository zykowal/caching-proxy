use std::error::Error;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub origin: String,
}

impl Config {
    pub fn parse<T>(mut args: T) -> Result<Self, Box<dyn Error>>
    where
        T: Iterator<Item = String>,
    {
        args.next();

        let mut port = 3000;
        let mut origin = String::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" | "-p" => {
                    port = args.next().ok_or("port is required")?.parse::<u16>()?;
                }
                "--origin" | "-o" => {
                    origin = args.next().ok_or("origin is required")?;
                }
                "--help" | "-h" => {
                    print_help();
                    break;
                }
                _ => {
                    print_help();
                    break;
                }
            }
        }

        Ok(Self { port, origin })
    }
}

fn print_help() {
    println!("Usage: demo[EXE] [OPTIONS] --name <NAME>");
    println!();
    println!("Options:");
    println!("  -p, --port   <PORT>    The port on which the caching proxy server will run");
    println!(
        "  -o, --origin <URL>     The URL of the server to which the requests will be forwarded"
    );
    println!("  -h, --help             Print help");
}
