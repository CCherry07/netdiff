mod config;
mod req;
mod utils;

pub mod cli;
pub use config::{DiffConfig, DiffProfile, ResponseProfile};
pub use req::is_default;
pub use req::RequestProfile;
pub use utils::diff_text_to_terminal_inline;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
}
