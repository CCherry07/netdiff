use anyhow::{Ok, Result};
use netdiff::{DiffConfig, LoadConfig};

fn main() -> Result<()> {
    let content = include_str!("../features/test.yml");
    let config = DiffConfig::from_yaml(content)?;
    println!("{:?}", config);
    Ok(())
}
