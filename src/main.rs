use anyhow::Ok;
use anyhow::Result;
use clap::Parser;
use netdiff::cli::{Action, Args, RunArgs};
use netdiff::DiffConfig;
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("error"),
    }
    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./default.yml".to_string());
    let config = DiffConfig::load_yml(&config_file).await?;

    let profile = config
        .get_profile(&args.profile)
        .ok_or_else(|| anyhow::anyhow!("proflie {} is error:{}", args.profile, config_file))?;

    let extra_args = args.extra_params.into();
    profile.diff(extra_args).await?;
    Ok(())
}
