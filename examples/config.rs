use anyhow::{Ok, Result};
use netdiff::DiffConfig;

fn main() -> Result<()> {
    let content = include_str!("../features/test.yml");
    let config = DiffConfig::from_yml(content)?;
    println!("{:?}", config);
    Ok(())
}
