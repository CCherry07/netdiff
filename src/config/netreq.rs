use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::RequestProfile;
use anyhow::{Context, Result};

use super::{LoadConfig, ValidateConfig};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, RequestProfile>,
}

impl LoadConfig for RequestConfig {}
impl ValidateConfig for RequestConfig {
    fn validate(&self) -> Result<()> {
        for (name, propfile) in &self.profiles {
            propfile.validate().context(format!("profile : {}", name))?;
        }
        Ok(())
    }
}

impl RequestConfig {
    pub fn new(profiles: HashMap<String, RequestProfile>) -> Self {
        Self { profiles }
    }
    pub fn get_profile(&self, name: &str) -> Option<&RequestProfile> {
        self.profiles.get(name)
    }
}
