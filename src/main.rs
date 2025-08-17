mod args;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use args::Args;
use clap::Parser;

mod config;
use config::{Config, GroupConfig, ModelConfig};
use ollama_file_find::ScanOutcome;

type Result<T> = std::result::Result<T, anyhow::Error>;

fn main() -> Result<()> {
    // 1. Parse CLI and scan manifests
    let args = Args::parse();
    let ScanOutcome { models, errors } = ollama_file_find::scan_manifests(&args.scan_args());
    report_scan_errors(&errors);

    // 2. Load config (or defaults) and apply simple field + macro overrides
    let mut config = load_base_config(&args)?;
    apply_top_level_overrides(&mut config, &args);
    apply_macro_overrides(&mut config, &args);

    // 3. Select which discovered models we will import.
    let selected: Vec<_> = if let Some(specified) = &args.specify_models {
        let wanted: HashSet<_> = specified.iter().collect();
        models.iter().filter(|m| wanted.contains(&m.name)).collect()
    } else if args.all_models {
        models.iter().collect()
    } else {
        vec![]
    };

    if args.verbose {
        eprintln!(
            "Found {} models; importing {}",
            models.len(),
            selected.len()
        );
    }

    let selected_models: Vec<_> = selected
        .iter()
        .filter_map(|m| {
            if m.primary_blob_path.is_none() && args.verbose {
                eprintln!(
                    "Warning: skipping model '{}' with no primary blob path",
                    m.name
                );
            }

            Some(SelectedModel {
                name: m.name.as_str(),
                model_path: m.primary_blob_path.as_ref()?,
            })
        })
        .collect();

    // 4. Build per-model CLI spec maps (aliases, filters) + command template context
    let alias_specs = parse_multi_map(args.alias.as_ref());
    let filter_specs = parse_multi_kv_map(args.filter.as_ref());
    let cmd_templates = command_templates(&args);

    // 5. Import/update model entries in config
    import_models(
        &mut config,
        &args,
        &selected_models,
        &alias_specs,
        &filter_specs,
        &cmd_templates,
    );

    // 6. Optional grouping
    apply_single_group(&mut config, &args, &selected_models);

    // 7. Serialize and write output (respecting dry-run / overwrite rules)
    let yaml = render_config(&config, &args)?;
    write_config_output(&yaml, &config, &args)?;

    Ok(())
}

// ------------------------------ High-level steps ------------------------------
/// Print any scan errors to stderr so the user can still get a partial result.
fn report_scan_errors<E: std::fmt::Display>(errors: &[E]) {
    for e in errors {
        eprintln!("Error scanning manifest: {}", e);
    }
}

/// Load an existing config file if provided, otherwise return defaults.
fn load_base_config(args: &Args) -> Result<Config> {
    if let Some(path) = &args.input_config {
        if args.verbose {
            eprintln!("Reading input config: {:?}", path);
        }
        let file = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&file)?)
    } else {
        if args.verbose {
            eprintln!("No input config provided; starting from defaults");
        }
        Ok(Config::default())
    }
}

/// Apply simple top-level scalar overrides from the CLI to the in-memory config.
fn apply_top_level_overrides(config: &mut Config, args: &Args) {
    if let Some(v) = args.health_check_timeout {
        config.health_check_timeout = Some(v);
    }
    if let Some(v) = args.log_level {
        config.log_level = Some(v);
    }
    if let Some(v) = args.start_port {
        config.start_port = Some(v);
    }
}

/// Apply KEY=VALUE macro overrides passed via CLI.
fn apply_macro_overrides(config: &mut Config, args: &Args) {
    if let Some(macros) = &args.macro_override {
        for kv in macros {
            if let Some((k, v)) = kv.split_once('=') {
                config
                    .macros
                    .insert(k.trim().to_string(), v.trim().to_string());
            } else {
                eprintln!("Ignoring malformed macro override (expected KEY=VALUE): {kv}");
            }
        }
    }
}

/// Struct holding resolved command template strings and override flags.
struct CommandTemplates {
    default_cmd_tpl: String,
    default_stop_tpl: Option<String>,
    override_cmd: bool,
    override_stop: bool,
}

fn command_templates(args: &Args) -> CommandTemplates {
    CommandTemplates {
        default_cmd_tpl: args
            .cmd_template
            .clone()
            .unwrap_or_else(|| "{model_path}".to_string()),
        default_stop_tpl: args.stop_cmd_template.clone(),
        override_cmd: args.cmd_template.is_some(),
        override_stop: args.stop_cmd_template.is_some(),
    }
}

/// Compact representation of a model we plan to import.
struct SelectedModel<'a> {
    name: &'a str,
    model_path: &'a Path,
}

/// Import or update model entries for all selected models.
fn import_models<'a>(
    config: &mut Config,
    args: &Args,
    selected: impl IntoIterator<Item = &'a SelectedModel<'a>>,
    alias_specs: &HashMap<String, Vec<String>>,
    filter_specs: &HashMap<String, Vec<(String, String)>>,
    templates: &CommandTemplates,
) {
    for sm in selected {
        let cmd = templates
            .default_cmd_tpl
            .replace("{model_path}", &sm.model_path.to_string_lossy())
            .replace("{model_name}", &sm.name);
        let cmd_stop = templates
            .default_stop_tpl
            .as_ref()
            .map(|tpl| tpl.replace("{model_name}", &sm.name));

        let entry = config
            .models
            .entry(sm.name.to_string())
            .or_insert_with(|| ModelConfig {
                unlisted: args.unlisted,
                ..ModelConfig::new(cmd.clone())
            });
        if templates.override_cmd {
            entry.cmd = cmd;
        }
        if templates.override_stop {
            entry.cmd_stop = cmd_stop;
        }

        if let Some(add_aliases) = alias_specs.get(sm.name) {
            merge_vec(&mut entry.aliases, add_aliases);
        }
        if let Some(add_filters) = filter_specs.get(sm.name) {
            for (k, v) in add_filters {
                entry.filters.insert(k.clone(), v.clone());
            }
        }
    }
}

/// Apply a single group containing all imported models if requested.
fn apply_single_group(config: &mut Config, args: &Args, selected: &[SelectedModel]) {
    if args.single_group {
        let group_name = args
            .single_group_name
            .clone()
            .unwrap_or_else(|| "imported".to_string());
        let members: Vec<String> = selected.iter().map(|m| m.name.to_string()).collect();
        config.groups.entry(group_name).or_insert(GroupConfig {
            swap: Some(true),
            exclusive: None,
            persistent: None,
            members,
        });
    }
}

/// Render the final YAML string. (Currently compact + pretty use same serializer.)
fn render_config(config: &Config, _args: &Args) -> Result<String> {
    Ok(serde_yaml::to_string(config)?)
}

/// Honor dry-run / overwrite rules and write or print the configuration.
fn write_config_output(yaml: &str, _config: &Config, args: &Args) -> Result<()> {
    if args.dry_run {
        println!("{yaml}");
        return Ok(());
    }
    if let Some(out_path) = &args.output_config {
        if out_path.exists() && args.no_clobber {
            anyhow::bail!(
                "Refusing to overwrite existing file {} (--no-clobber)",
                out_path.display()
            );
        }
        write_file(out_path, yaml.as_bytes())?;
        if args.verbose {
            eprintln!("Wrote config to {}", out_path.display());
        }
    } else if let Some(input_path) = &args.input_config {
        write_file(input_path, yaml.as_bytes())?;
        if args.verbose {
            eprintln!("Updated input config in place: {:?}", input_path);
        }
    } else {
        println!("{yaml}");
    }
    Ok(())
}

// gather models (ollama-file-find)
// extract necessary information
// Structure information for printing
// Comments?
// Write to file/stdout

fn parse_multi_map(spec: Option<&Vec<String>>) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(specs) = spec {
        for s in specs {
            if let Some((model, aliases)) = s.split_once('=') {
                let arr: Vec<String> = aliases
                    .split('|')
                    .map(|a| a.trim().to_string())
                    .filter(|a| !a.is_empty())
                    .collect();
                if !arr.is_empty() {
                    out.entry(model.trim().to_string()).or_default().extend(arr);
                }
            }
        }
    }
    out
}

fn parse_multi_kv_map(spec: Option<&Vec<String>>) -> HashMap<String, Vec<(String, String)>> {
    let mut out: HashMap<String, Vec<(String, String)>> = HashMap::new();
    if let Some(specs) = spec {
        for s in specs {
            if let Some((model, rest)) = s.split_once('=') {
                for kv in rest.split('|') {
                    if let Some((k, v)) = kv.split_once(':').or_else(|| kv.split_once('=')) {
                        // accept k:v or k=v inside
                        out.entry(model.trim().to_string())
                            .or_default()
                            .push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
        }
    }
    out
}

fn merge_vec(target: &mut Vec<String>, additions: &Vec<String>) {
    let mut existing: HashSet<String> = target.iter().cloned().collect();
    for a in additions {
        if existing.insert(a.clone()) {
            target.push(a.clone());
        }
    }
}

fn write_file(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::File::create(path)?;
    f.write_all(data)?;
    Ok(())
}
