use anyhow::Ok;
use anyhow::Result;
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use dialoguer::MultiSelect;
use netdiff::cli::{Action, Args, RunArgs};
use netdiff::highlight_text;
use netdiff::DiffConfig;
use netdiff::DiffProfile;
use netdiff::ExtraArgs;
use netdiff::RequestProfile;
use netdiff::ResponseProfile;
use std::io::stdout;
use std::io::Write;
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
        _ => panic!("error"),
    }
    Ok(())
}

async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();
    let url1: String = Input::with_theme(&theme)
        .with_prompt("url1?")
        .interact_text()?;
    let url2: String = Input::with_theme(&theme)
        .with_prompt("url2?")
        .interact_text()?;
    let req1: RequestProfile = url1.parse()?;
    let header_keys = req1.send(&ExtraArgs::default()).await?.get_header_keys();
    let name = Input::with_theme(&theme)
        .with_prompt("profile name?")
        .interact_text()?;
    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("select skip headers")
        .items(&header_keys)
        .interact()?;
    let skip_headers: Vec<String> = chosen.iter().map(|i| header_keys[*i].to_string()).collect();

    let req2: RequestProfile = url2.parse()?;
    let res = ResponseProfile::new(skip_headers, vec![]);
    let profile = DiffProfile::new(req1, req2, res);
    let config = DiffConfig::new(vec![(name, profile)].into_iter().collect());
    let result = serde_yaml::to_string(&config)?;

    let mut stdout = stdout().lock();
    write!(stdout, "======== Parse Yaml ========\n{}", highlight_text(&result, "yaml")?)?;
    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./default.yml".to_string());
    let config = DiffConfig::load_yml(&config_file).await?;

    let profile = config
        .get_profile(&args.profile)
        .ok_or_else(|| anyhow::anyhow!("proflie {} is error:{}", args.profile, config_file))?;

    let extra_args = args.extra_params.into();
    let diff_str = profile.diff(extra_args).await?;
    let mut stdout = stdout().lock();
    write!(stdout, "{}", diff_str)?;
    Ok(())
}
