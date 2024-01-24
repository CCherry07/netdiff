use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffConfig {
    pub profiles: HashMap<String, DiffProfile>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    pub res: ResponseProfile,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct  RequestProfile {

}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct  ResponseProfile {

}

