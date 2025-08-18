# ollama2llama

Import locally installed [Ollama](https://ollama.com) models into a YAML configuration usable by **llama-swap** (a model hot‑swap / lifecycle manager). It scans your Ollama model manifests and generates or updates a config file describing each model's launch command, metadata, aliases, filters, and optional grouping.

## Quick Start
```
cargo binstall ollama2llama
ollama2llama --all-models --output-config llama-swap.yaml
```
This scans `$OLLAMA_MODELS` (or the default `~/.ollama/models`) and writes a `llama-swap.yaml` describing every discovered model.

Dry run to preview:
```
ollama2llama --all-models --dry-run
```
Update an existing config in place:
```
ollama2llama -i llama-swap.yaml --all-models
```
Import only selected models:
```
ollama2llama --specify-models llama3:8b,phi3:3.8b --output-config llama-swap.yaml
```

## Command Template Placeholders
Default launch template: `{model_path}` (the resolved blob path). You can override:
```
--cmd-template "./serve --model {model_path} --name {model_name}"
--stop-cmd-template "pkill -f {model_name}"
```
Available placeholders:
- `{model_path}`: Absolute path to the primary model blob.
- `{model_name}`: The Ollama model identifier (e.g. `llama3:8b`).

## CLI Options 
| Flag | Description |
|------|-------------|
| `-i, --input-config <FILE>` | Existing YAML to load (else start from defaults). |
| `-o, --output-config <FILE>` | Destination YAML (else overwrite input, else stdout). |
| `-m, --model-dir <DIR>` | Override models directory (default: `$OLLAMA_MODELS` or `~/.ollama/models`). |
| `-s, --specify-models <MODEL>` | One or more model names to import (repeat / comma separated). |
| `-a, --all-models` | Import every discovered model (ignored if `--specify-models` used). |
| `-v, --verbose` | Extra progress + warnings. |
| `--cmd-template <CMD>` | Launch command template (use placeholders). |
| `--stop-cmd-template <CMD>` | Stop command template (use `{model_name}`). |
| `--start-port <PORT>` | Override starting port macro value (e.g. 5800). |
| `--health-check-timeout <SECONDS>` | Override health check timeout seconds. |
| `--log-level <LEVEL>` | `debug|info|warn|error` (default Info). |
| `-M, --macro-override <K=V>` | Add/override macros (repeat / comma separated). |
| `--alias <NAME=A1|A2>` | Add aliases for a model (repeatable). |
| `--filter <NAME=k:v|k=v>` | Add filter key/values for a model (repeatable). |
| `--unlisted` | Mark imported models as hidden (`unlisted: true`). |
| `--single-group` | Put all imported models into one swap group. |
| `--single-group-name <NAME>` | Name of that group (default `imported`). |
| `--dry-run` | Print YAML only; no file writes. |
| `--no-clobber` | Error instead of overwriting an existing output file. |
| `-h, --help` | Show help. |
| `-V, --version` | Show version. |

## Generated YAML Structure
Top level fields:
- `health_check_timeout` (u64, default 120) – Wait time for readiness.
- `log_level` (debug|info|warn|error, default info).
- `start_port` (u16) – Starting value used for a `${PORT}` macro expansion by downstream tooling.
- `macros` (map) – User-defined string substitutions (augmented with `-M`).
- `models` (map<string, ModelConfig>) – One entry per model.
- `groups` (map<string, GroupConfig>) – Optional grouping for swap semantics.

### ModelConfig Fields
- `cmd` (string, required) – Launch command (after placeholder substitution).
- `env` (array<string>) – Environment variable assignments (`KEY=VALUE`).
- `cmd_stop` (string) – Optional graceful stop command.
- `proxy` (string) – Proxy URL.
- `aliases` (array<string>) – Alternative names; populated via `--alias`.
- `check_endpoint` (string) – Health probe endpoint.
- `ttl` (u64) – Auto-unload inactivity timeout (seconds).
- `use_model_name` (string) – Upstream name override.
- `filters` (map<string,string>) – Arbitrary metadata; from `--filter`.
- `unlisted` (bool) – Hide from listing endpoints.

### GroupConfig Fields
- `swap` (bool) – If true, only one member active at a time (defaults to true when created by `--single-group`).
- `exclusive` (bool) – Starting a model unloads models in other groups.
- `persistent` (bool) – Protect models from being unloaded by other groups.
- `members` (array<string>, required) – Model names in the group.

## Macro Overrides
Use `-M` or `--macro-override` multiple times or with commas:
```
-M PORT_BASE=5800,HOST=0.0.0.0
```
These key/value pairs land in `macros:`. Existing keys in an input config are overwritten.

## Aliases & Filters Syntax
Alias example:
```
--alias llama3:8b=llama3|llama3-8b --alias phi3:3.8b=phi3
```
Filter example (either `:` or `=` inside pairs accepted):
```
--filter llama3:8b=family:llama|quant:Q4_K_M
```
Produces:
```
filters:
  family: llama
  quant: Q4_K_M
```

## Single Group Mode
`--single-group` + optional `--single-group-name name` creates a group with `swap: true` containing every imported model, enabling simple one-at-a-time swapping in downstream tooling.

## Typical Workflow
1. Generate initial config for all models.
2. Manually refine commands, env vars, or add health checks.
3. Re-run with `--specify-models` to add new models without touching existing custom edits (unless you specify overriding templates).

## Safety & Idempotency
- Existing model entries are only overwritten for `cmd` / `cmd_stop` when you explicitly pass the corresponding template flags; otherwise they are left as-is.
- Macros added via CLI override existing keys of the same name.
- `--no-clobber` guards against accidental overwrites when writing to a new output file.

## License
This crate is licensed under the [MIT License](https://github.com/Exotik850/ollama2llama/blob/master/LICENSE.md). Contributions and suggestions welcome.
