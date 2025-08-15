use serde::Deserialize;
use std::collections::HashMap;

// Logging levels accepted in the config / CLI.
#[derive(Debug, Deserialize, Clone, Copy, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Number of seconds to wait for a model to be ready (default: 120)
    #[serde(default)]
    pub health_check_timeout: Option<u64>,

    /// Logging level (debug, info, warn, error)
    #[serde(default)]
    pub log_level: Option<LogLevel>,

    /// Starting port for `${PORT}` macro (default: 5800)
    #[serde(default)]
    pub start_port: Option<u16>,

    /// Reusable string macros
    #[serde(default)]
    pub macros: HashMap<String, String>,

    /// Models keyed by model ID (required)
    pub models: HashMap<String, ModelConfig>,

    /// Optional group definitions
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            health_check_timeout: Some(120),
            log_level: Some(LogLevel::Info),
            start_port: Some(5800),
            macros: HashMap::new(),
            models: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    /// Command to run to start the inference server (required)
    pub cmd: String,

    /// Environment variables for the command
    #[serde(default)]
    pub env: Vec<String>,

    /// Command to gracefully stop the model
    #[serde(default)]
    pub cmd_stop: Option<String>,

    /// Proxy URL
    #[serde(default)]
    pub proxy: Option<String>,

    /// Alternative model names
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Endpoint to check if server is ready
    #[serde(default)]
    pub check_endpoint: Option<String>,

    /// Auto-unload timeout (seconds)
    #[serde(default)]
    pub ttl: Option<u64>,

    /// Upstream model name override
    #[serde(default)]
    pub use_model_name: Option<String>,

    /// Model filter configuration
    #[serde(default)]
    pub filters: HashMap<String, String>,

    /// Hide model from /v1/models and /upstream lists
    #[serde(default)]
    pub unlisted: Option<bool>,
}

#[derive(Debug, Deserialize)]
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
