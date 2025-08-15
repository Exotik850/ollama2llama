mod args;
use args::Args;
use clap::Parser;

mod config;
use config::{Config, GroupConfig, ModelConfig};

type Result<T> = std::result::Result<T, anyhow::Error>;


fn main() -> Result<()> {
    let args = Args::parse();
    let file = args.input_config.map(std::fs::read_to_string).transpose()?;
    let config: Option<Config> = file.as_deref().map(serde_yaml::from_str).transpose()?;
    if let Some(config) = &config {
        println!("{:#?}", config);
    }
    Ok(())
}
