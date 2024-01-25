use anyhow::{anyhow, Ok, Result};
use clap::{Parser, Subcommand};

use crate::ExtraArgs;

#[derive(Debug, Parser, Clone)]
#[clap(version,author,about,long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}
#[derive(Debug, Subcommand, Clone)]
#[non_exhaustive]
pub enum Action {
    Run(RunArgs),
}

#[derive(Debug, Parser, Clone)]
pub struct RunArgs {
    #[clap(short, long, value_parser)]
    pub profile: String,

    #[clap(short,long,value_parser = parser_key_val , number_of_values = 1)]
    pub extra_params: Vec<KeyVal>,

    #[clap(short, long)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyVal {
    key_type: KeyValType,
    key: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValType {
    Headers,
    Body,
    Query,
}

pub fn parser_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, "=");
    let (key, val) = (
        parts
            .next()
            .ok_or_else(|| anyhow!("not an effective value:{}", s))?
            .trim(),
        parts
            .next()
            .ok_or_else(|| anyhow!("not an effective value:{}", s))?
            .trim(),
    );

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Headers, &key[1..]),
        Some('@') => (KeyValType::Body, &key[1..]),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("not an effective value")),
    };
    Ok(KeyVal {
        key_type,
        key: key.to_string(),
        value: val.to_string(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(value: Vec<KeyVal>) -> Self {
        let mut headers = vec![];
        let mut query = vec![];
        let mut body = vec![];

        for v in value {
            match v.key_type {
                KeyValType::Headers => headers.push((v.key, v.value)),
                KeyValType::Body => body.push((v.key, v.value)),
                KeyValType::Query => query.push((v.key, v.value)),
            }
        }

        Self {
            headers,
            body,
            query,
        }
    }
}
