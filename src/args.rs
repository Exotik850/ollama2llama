use std::path::PathBuf;

use ollama_file_find::{ScanArgs, ollama_models_dir};

#[derive(clap::Parser, Debug)]
#[command(author, version, about="Import models from Ollama into a config file for use with llama-swap", long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "YAML config file to read/write to (if not specified, a default config is generated)"
    )]
    pub input_config: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "YAML config file to write (if not specified, input file is overwritten or printed to stdout if no input file is given)"
    )]
    pub output_config: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "DIR",
        help = "Directory to search for models, defaults to $OLLAMA_MODELS or ~/.ollama/models"
    )]
    pub model_dir: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "MODEL",
        help = "Specify the names of one or more models to import into the config file (can be used multiple times)",
        value_delimiter = ','
    )]
    pub specify_models: Option<Vec<String>>,

    #[arg(
        short,
        long,
        help = "Import all models from the model directory, this is ignored if `specify_models` is set",
        default_value_t = true
    )]
    pub all_models: bool,

    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    // ---------------------------- Config Generation ----------------------------
    #[arg(
        long,
        value_name = "CMD",
        help = "Override default launch command template. Use {model_path} and {model_name} placeholders."
    )]
    pub cmd_template: Option<String>,

    #[arg(
        long,
        value_name = "CMD",
        help = "Override default stop command template. Use {model_name} placeholder."
    )]
    pub stop_cmd_template: Option<String>,

    #[arg(
        long,
        value_name = "PORT",
        help = "Set starting port macro value overriding config file (e.g. 5800)"
    )]
    pub start_port: Option<u16>,

    #[arg(
        long,
        value_name = "SECONDS",
        help = "Health check timeout override in seconds"
    )]
    pub health_check_timeout: Option<u64>,

    #[arg(
        long,
        value_name = "LEVEL",
        help = "Log level override (debug, info, warn, error)"
    )]
    pub log_level: Option<crate::config::LogLevel>,

    #[arg(
        short='M',
        long,
        value_name = "MACRO=VALUE",
        help = "Add or override macro(s) in output config",
        value_delimiter = ','
    )]
    pub macro_override: Option<Vec<String>>,

    #[arg(
        long,
        value_name = "NAME=ALIAS1|ALIAS2",
        help = "Add aliases for a model (repeat or comma separated)",
        value_delimiter = ','
    )]
    pub alias: Option<Vec<String>>,

    #[arg(
        long,
        value_name = "KEY=VALUE",
        help = "Add model filter key/value (repeat or comma separated)",
        value_delimiter = ','
    )]
    pub filter: Option<Vec<String>>,

    #[arg(long, help = "Mark imported models as unlisted by default")]
    pub unlisted: bool,

    #[arg(long, help = "Group all imported models into a single swap group")]
    pub single_group: bool,

    #[arg(
        long,
        value_name = "NAME",
        help = "Name of the single group created with --single-group (default: imported)"
    )]
    pub single_group_name: Option<String>,

    // ---------------------------- Output Control ----------------------------
    #[arg(long, help = "Do not write any files; just print resulting config")]
    pub dry_run: bool,

    #[arg(
        long,
        help = "Refuse to overwrite an existing output file (error instead)"
    )]
    pub no_clobber: bool,
}

impl Args {
    pub fn scan_args(&self) -> ScanArgs<'static> {
        let model_dir = self.model_dir.clone().unwrap_or_else(ollama_models_dir);
        ScanArgs::new(model_dir.join("manifests"), model_dir.join("blobs"))
            .with_verbose(true) // enables blob paths
    }
}
