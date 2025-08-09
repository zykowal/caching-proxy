use std::{env::args, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    caching_proxy::run(args())
}
