use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::RequestProfile;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, RequestProfile>,
}
