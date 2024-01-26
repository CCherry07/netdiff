use anyhow::{Ok, Result};
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use url::Url;

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

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

fn is_empty_value(v: &Value) -> bool {
    v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty())
}

fn default_params() -> Value {
    serde_json::json!({})
}

impl RequestProfile {
    pub async fn send(&self, extra_args: &super::ExtraArgs) -> Result<Response> {
        let url = self.url.clone();
        let (headers, body, query) = self.gen_req_config(extra_args)?;
        let req_builder = Client::builder().build()?;
        let _ = req_builder
            .request(self.method.clone(), url)
            .headers(headers)
            .query(&query)
            .body(body);

        todo!()
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

        let content_type = headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<mime::Mime>().ok());

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
            _ => Err(anyhow::anyhow!("不是有效的 CONTENT_TYPE"))
        }
    }
}
