use std::collections::HashMap;

use anyhow::{Ok, Result};
use http::{HeaderMap, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;
use url::Url;

use crate::ExtraArgs;

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
    pub async fn diff(&self , args:ExtraArgs)-> Result<String> {
        println!("proflie:{:?}" ,self);
        println!("args:{:?}" ,args);

        // let res1 = req1.send(args).await?;
        // let res2 = res2.send(args).await?;
        // let text1 = res1.filter_text(&self.res).await?;
        // let text2 = res2.filter_text(&self.res).await?;
        Ok("".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RequestProfile {
    #[serde(
        with = "http_serde::method",
        skip_serializing_if = "is_default",
        default
    )]
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "is_empty_value", default = "default_params")]
    pub params: Value,
    #[serde(skip_serializing_if = "HeaderMap::is_empty", default)]
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user_agent: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

fn is_empty_value(v: &Value) -> bool {
    v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty())
}

fn default_params() -> Value {
    serde_json::json!({})
}
