use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Managed,
    Integrated,
    SelfHosted,
    Experimental,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub service_type: String,
    pub support_level: SupportLevel,
    pub description: String,
    pub operations: Vec<String>,
}
