mod netdiff;
mod netreq;

use anyhow::{Ok, Result};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method};
use mime::Mime;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use url::Url;

pub use netdiff::{DiffConfig, DiffProfile, ResponseProfile};
pub use netreq::RequestConfig;

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

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub fn is_empty_value(v: &Value) -> bool {
    v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty())
}

pub fn default_params() -> Value {
    serde_json::json!({})
}

#[derive(Debug)]
pub struct ResponseExt(Response);

impl RequestProfile {
    pub fn new(
        method: Method,
        url: Url,
        params: Value,
        headers: HeaderMap,
        body: Option<Value>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            method,
            url,
            params,
            headers,
            body,
            user_agent,
        }
    }
    pub async fn send(&self, extra_args: &super::ExtraArgs) -> Result<ResponseExt> {
        let url = self.url.clone();
        let (headers, body, query) = self.gen_req_config(extra_args)?;
        let req_builder = Client::builder().build()?;
        let res = req_builder
            .request(self.method.clone(), url)
            .headers(headers)
            .query(&query)
            .body(body)
            .send()
            .await?;

        Ok(ResponseExt(res))
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if !self.params.is_object() {
            return Err(anyhow::anyhow!(
                "parmas: {} 不是一个对象",
                serde_yaml::to_string(&self.params)?
            ));
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow::anyhow!(
                    "body:{} 不是一个对象",
                    serde_yaml::to_string(body)?
                ));
            }
        }
        Ok(())
    }

    pub fn gen_req_config(
        &self,
        extra_args: &super::ExtraArgs,
    ) -> Result<(HeaderMap, String, Value)> {
        let mut headers = self.headers.clone();
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));
        let mut query = self.params.clone();

        if !headers.contains_key(CONTENT_TYPE) {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str(&mime::APPLICATION_JSON.to_string())?,
            );
        }

        for (k, v) in &extra_args.headers {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }
        for (k, v) in &extra_args.body {
            body[k] = v.parse()?;
        }
        for (k, v) in &extra_args.query {
            query[k] = v.parse()?;
        }

        let content_type = get_content_type(&headers);

        match content_type {
            Some(content) if content == mime::APPLICATION_JSON => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, body, query))
            }
            Some(content)
                if content == mime::APPLICATION_WWW_FORM_URLENCODED
                    || content == mime::MULTIPART_FORM_DATA =>
            {
                let body = serde_qs::to_string(&body)?;
                Ok((headers, body, query))
            }
            _ => Err(anyhow::anyhow!("不是有效的 CONTENT_TYPE")),
        }
    }
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut url = Url::parse(s)?;
        let qs = Url::query_pairs(&url);
        let mut parmas = json!({});
        for (k, v) in qs {
            parmas[&*k] = v.parse()?;
        }
        url.set_query(None);
        let profile = Self {
            method: Method::GET,
            url,
            params: parmas,
            headers: HeaderMap::new(),
            body: None,
            user_agent: None,
        };
        Ok(profile)
    }
}
impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let res = self.0;
        let output = String::new();
        let output = append_header_str(output, &res, &profile.skip_headers)?;
        let output = append_body_str(output, res, &profile.skip_headers).await?;
        Ok(output)
    }
    pub fn get_header_keys(self) -> Vec<String> {
        let res = self.0;
        let headers = res.headers();
        headers
            .iter()
            .map(|(k, _)| k.to_string())
            .collect::<Vec<String>>()
    }
}

pub fn append_header_str(
    mut output: String,
    res: &Response,
    skip_headers: &[String],
) -> Result<String> {
    output.push_str(&format!("{:?} {:?} \n", res.version(), res.status()));
    let headers = res.headers();
    headers.iter().for_each(|(k, v)| {
        if skip_headers.contains(&k.to_string()) {
            output.push_str(&format!("{}: {:?} \n", k, v));
        }
    });
    return Ok(output);
}

pub async fn append_body_str(
    mut output: String,
    res: Response,
    skip_headers: &[String],
) -> Result<String> {
    let headers = res.headers();
    let content_type = get_content_type(&headers);
    let text = res.text().await?;
    match content_type {
        Some(content) if content == mime::APPLICATION_JSON => {
            let body_text = filter_json(&text, &skip_headers)?;
            output.push_str(&body_text);
            Ok(output)
        }
        _ => Ok(text),
    }
}

pub fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json = serde_json::from_str(text)?;
    if let serde_json::Value::Object(ref mut obj) = json {
        for k in skip {
            obj.remove(k);
        }
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

pub fn get_content_type(headers: &HeaderMap) -> Option<Mime> {
    headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(";").next())
        .and_then(|s| s.parse::<mime::Mime>().ok())
}
