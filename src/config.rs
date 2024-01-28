use std::collections::HashMap;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{diff_text_to_terminal_inline, ExtraArgs, RequestProfile};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

impl DiffConfig {
    pub async fn load_yml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_yml(&content)
    }

    pub fn from_yml(content: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    pub res: ResponseProfile,
}

impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req2.send(&args).await?;
        let text1 = res1.filter_text(&self.res).await?;
        let text2 = res2.filter_text(&self.res).await?;
        Ok(diff_text_to_terminal_inline(&text1, &text2)?)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}
