mod config;

pub mod cli;
pub use config::{DiffConfig, DiffProfile, RequestProfile, ResponseProfile};

#[derive(Debug,Clone,PartialEq, Eq)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
}
