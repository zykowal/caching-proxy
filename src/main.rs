use std::{env::args, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    caching_proxy::run(args()).await
}
