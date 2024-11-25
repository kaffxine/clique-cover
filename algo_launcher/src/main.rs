use anyhow::{anyhow, Result};

const URL: &str = "ws://127.0.0.1:8080";

fn main() -> Result<()> {
    println!("{}", URL);
    Ok(())
}
