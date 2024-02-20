use std::collections::HashMap;

use super::{is_default, LoadConfig, ValidateConfig};
use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::{diff_text_to_terminal_inline, ExtraArgs, RequestProfile};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

impl LoadConfig for DiffConfig {}
impl ValidateConfig for DiffConfig {
    fn validate(&self) -> Result<()> {
        for (name, propfile) in &self.profiles {
            propfile.validate().context(format!("profile : {}", name))?;
        }
        Ok(())
    }
}

impl DiffConfig {
    pub fn new(profiles: HashMap<String, DiffProfile>) -> Self {
        Self { profiles }
    }
    
    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub res: ResponseProfile,
}

impl DiffProfile {
    pub fn new(req1: RequestProfile, req2: RequestProfile, res: ResponseProfile) -> Self {
        Self { req1, req2, res }
    }
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req2.send(&args).await?;
        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;
        diff_text_to_terminal_inline(&text1, &text2)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.req1.validate().context("req1 config is failed")?;
        self.req2.validate().context("req2 config is failed")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}
