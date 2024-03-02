use anyhow::Ok;
use anyhow::Result;
use atty::Stream;
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use netdiff::cli::{Action, Args, RunArgs};
use netdiff::highlight_text;
use netdiff::LoadConfig;
use netdiff::{get_body_text, get_header_text, get_status_text, RequestConfig, RequestProfile};
use std::fmt::Write as _;
use std::io::stdout;
use std::io::Write as _;

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
    let url: String = Input::with_theme(&theme)
        .with_prompt("url?")
        .interact_text()?;
    let profile: RequestProfile = url.parse()?;

    let name = Input::with_theme(&theme)
        .with_prompt("profile name?")
        .interact_text()?;

    let config = RequestConfig::new(vec![(name, profile)].into_iter().collect());
    let result = serde_yaml::to_string(&config)?;

    let mut stdout = stdout().lock();

    if atty::is(Stream::Stdout) {
        write!(
            stdout,
            "======== Parse Yaml ========\n{}",
            highlight_text(&result, "yaml", None)?
        )?;
    } else {
        write!(stdout, "{}", &result)?;
    }

    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./default.yml".to_string());
    let config = RequestConfig::load_yaml(&config_file).await?;

    let profile = config
        .get_profile(&args.profile)
        .ok_or_else(|| anyhow::anyhow!("proflie {} is error:{}", args.profile, config_file))?;

    let extra_args = args.extra_params.into();
    let res = profile.send(&extra_args).await?.into_inner();
    let status = get_status_text(&res)?;
    let headers = get_header_text(&res, &[])?;
    let body = get_body_text(res, &[]).await?;
    let url = profile.get_url(&extra_args)?;

    let mut output = String::new();
    write!(&mut output, "send url: {}\n", &url)?;
    write!(&mut output, "{}", status)?;

    if atty::is(Stream::Stdout) {
        write!(
            &mut output,
            "{}",
            highlight_text(&headers, "yaml", Some("InspiredGitHub"))?
        )?;
        write!(&mut output, "{}", highlight_text(&body, "json", None)?)?;
    } else {
        write!(&mut output, "{}", &body)?;
    }

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    write!(stdout, "{}", &output)?;
    Ok(())
}
