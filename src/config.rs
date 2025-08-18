#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Logging levels accepted in the config / CLI.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Number of seconds to wait for a model to be ready (default: 120)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_check_timeout: Option<u64>,

    /// Logging level (debug, info, warn, error)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    /// Starting port for `${PORT}` macro (default: 5800)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_port: Option<u16>,

    /// Reusable string macros
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub macros: HashMap<String, String>,

    /// Models keyed by model ID (required)
    pub models: HashMap<String, ModelConfig>,

    /// Optional group definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub groups: HashMap<String, GroupConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            health_check_timeout: Some(120),
            log_level: Some(LogLevel::Info),
            start_port: None,
            macros: HashMap::new(),
            models: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}

// helper to skip serializing false booleans
fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfig {
    /// Command to run to start the inference server (required)
    pub cmd: String,

    /// Environment variables for the command
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,

    /// Command to gracefully stop the model
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmd_stop: Option<String>,

    /// Proxy URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,

    /// Alternative model names
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// Endpoint to check if server is ready
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_endpoint: Option<String>,

    /// Auto-unload timeout (seconds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,

    /// Upstream model name override
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_model_name: Option<String>,

    /// Model filter configuration
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub filters: HashMap<String, String>,

    /// Hide model from /v1/models and /upstream lists
    #[serde(default, skip_serializing_if = "is_false")]
    pub unlisted: bool,
}

impl ModelConfig {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd,
            env: Vec::new(),
            cmd_stop: None,
            proxy: None,
            aliases: Vec::new(),
            check_endpoint: None,
            ttl: None,
            use_model_name: None,
            filters: HashMap::new(),
            unlisted: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupConfig {
    /// Whether to allow only one model running at a time in this group
    #[serde(default)]
    pub swap: Option<bool>,

    /// Whether running a model in this group unloads other groups
    #[serde(default)]
    pub exclusive: Option<bool>,

    /// Prevent other groups from unloading models in this group
    #[serde(default)]
    pub persistent: Option<bool>,

    /// Models in the group (required)
    pub members: Vec<String>,
}
