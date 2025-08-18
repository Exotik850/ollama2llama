# ollama2llama

Import locally installed [Ollama](https://ollama.com) models into a YAML configuration consumable by [llama-swap](https://github.com/mostlygeek/llama-swap) (a model hot‑swap / lifecycle manager). It scans your Ollama manifests and (a) generates a fresh config or (b) incrementally augments an existing one without clobbering custom edits unless you explicitly request overrides.

## Installation
From source (latest main):
```bash
cargo install --path .        # in a cloned repo
```
From Git (exact revision):
```bash
cargo install --git https://github.com/Exotik850/ollama2llama --rev <commit>
```
Using cargo-binstall (prebuilt if available):
```bash
cargo binstall ollama2llama
```

## Quick Start
Generate a config for all discovered models:
```bash
ollama2llama --all-models --output-config llama-swap.yaml
```
This scans `$OLLAMA_MODELS` (or `~/.ollama/models`) and writes `llama-swap.yaml` with one entry per model.

Dry run (no file writes):
```bash
ollama2llama --all-models --dry-run
```

Update an existing config in place (retains prior per‑model tweaks unless you pass template overrides):
```bash
ollama2llama -i llama-swap.yaml --all-models
```

Import only selected models:
```bash
ollama2llama --specify-models llama3:8b,phi3:3.8b --output-config llama-swap.yaml
```

Add aliases & filters while importing:
```bash
ollama2llama \
  --specify-models llama3:8b \
  --alias llama3:8b=llama3|meta-llama3 \
  --filter llama3:8b=family:llama|quant:Q4_K_M \
  --output-config llama-swap.yaml
```

## Command Templates
Default launch template: `{model_path}` (the resolved blob path).

Override launch & stop commands (placeholders are substituted literally):
```bash
--cmd-template "./serve --model {model_path} --name {model_name} --port ${PORT}" \
--stop-cmd-template "pkill -f {model_name}"
```
Placeholders:
- `{model_path}` – Absolute path to the primary model blob.
- `{model_name}` – The Ollama model identifier (e.g. `llama3:8b`).

Only when you pass `--cmd-template` / `--stop-cmd-template` are existing `cmd` / `cmd_stop` values overwritten; otherwise they are preserved (idempotent incremental updates).

## CLI Options
| Flag | Description |
|------|-------------|
| `-i, --input-config <FILE>` | Existing YAML to load (else start from defaults). |
| `-o, --output-config <FILE>` | Destination YAML (else overwrite input, else stdout). |
| `-m, --model-dir <DIR>` | Override models directory (default: `$OLLAMA_MODELS` or `~/.ollama/models`). |
| `-s, --specify-models <MODEL>` | One or more model names to import (repeat / comma separated). |
| `-a, --all-models` | Import every discovered model (ignored if `--specify-models` used) (default: enabled). |
| `-v, --verbose` | Progress + warnings (e.g. missing blob path). |
| `--cmd-template <CMD>` | Launch command template. |
| `--stop-cmd-template <CMD>` | Stop command template. |
| `--start-port <PORT>` | Set `${PORT}` macro base value. |
| `--health-check-timeout <SECONDS>` | Override readiness timeout (default 120). |
| `--log-level <LEVEL>` | `debug|info|warn|error` (default info). |
| `-M, --macro-override <K=V>` | Add / override `macros` (repeatable). |
| `--alias <NAME=A1|A2>` | Add model aliases. |
| `--filter <NAME=k:v|k=v>` | Add filter key/value pairs. |
| `--unlisted` | Mark imported models as hidden (`unlisted: true`). |
| `--single-group` | Create one swap group containing all imported models. |
| `--single-group-name <NAME>` | Name of created group (default `imported`). |
| `--dry-run` | Print YAML only. |
| `--no-clobber` | Error if output file already exists. |
| `-h, --help` | Help. |
| `-V, --version` | Version. |

## Sample Output
```yaml
health_check_timeout: 120
log_level: info
models:
  llama3:8b:
    cmd: /home/user/.ollama/models/blobs/sha256-<truncated>
    aliases:
      - llama3
    filters:
      family: llama
      quant: Q4_K_M
groups:
  imported:
    swap: true
    members:
      - llama3:8b
```

## YAML Schema Reference
Top‑level keys:
- `health_check_timeout` – u64 seconds (default 120)
- `log_level` – debug | info | warn | error (default info)
- `start_port` – u16; starting value for a downstream `${PORT}` macro (if used by llama-swap)
- `macros` – map<string,string> of user macros
- `models` – map<string, ModelConfig>
- `groups` – map<string, GroupConfig>

ModelConfig:
- `cmd` – required launch command
- `env` – array of `KEY=VALUE` strings (manually editable)
- `cmd_stop` – optional graceful stop command
- `proxy` – optional proxy URL
- `aliases` – alternative names
- `check_endpoint` – readiness probe endpoint
- `ttl` – inactivity timeout seconds
- `use_model_name` – upstream name override
- `filters` – arbitrary metadata key/value pairs
- `unlisted` – hide from listing endpoints

GroupConfig:
- `swap` – only one member active at a time
- `exclusive` – starting a member unloads models in other groups
- `persistent` – prevent other groups from unloading these models
- `members` – required list of model names

also see [here](https://github.com/mostlygeek/llama-swap/wiki/Configuration) for more details about the configuration of llama-swap

## Macros
Add / override macros:
```bash
ollama2llama -M PORT=5800,HOST=0.0.0.0 --all-models -o llama-swap.yaml
```
They appear under `macros:`; identical keys in an existing file are replaced.

## Aliases & Filters
Aliases use `=` then `|` separated names:
```bash
--alias llama3:8b=llama3|meta-llama3
```
Filters accept `k:v` or `k=v` inside the right-hand side, separated by `|`:
```bash
--filter llama3:8b=family:llama|quant:Q4_K_M
```

## Single Group Mode
Create a simple swapping group (mutual exclusion):
```bash
ollama2llama --all-models --single-group -o llama-swap.yaml
```
Produces a group named `imported` (or your override) with `swap: true`.

## Recommended Workflow
1. Generate initial config for all (or selected) models.
2. Manually refine per-model `cmd`, add `env`, `check_endpoint`, or `ttl` as needed.
3. Re-run later with `--specify-models` to import new additions; existing customized entries remain unless you supply overriding templates.
4. Use version control to track manual refinements.

## Idempotency & Safety
- Only template‑flagged fields (`cmd`, `cmd_stop`) are overwritten when you explicitly pass their flags.
- `--macro-override` always overwrites keys of same name.
- `--no-clobber` ensures you never accidentally overwrite a pre-existing output file.
- Missing blob paths are reported (with `--verbose`) and those models are skipped.

## Design Notes
The tool focuses on discovery + structured augmentation, leaving nuanced runtime parameters (environment, health endpoints) for manual curation so you keep full control while still automating the repetitive parts.

## Potential Future Enhancements
These are reasonable, non-breaking extensions you may consider (PRs welcome):
1. Exclusion flags: `--exclude-models` or pattern / glob filtering.
2. Pattern selection: `--match <regex>` to import models by regex.
3. Per-model overrides via files: `--alias-file`, `--filter-file` (YAML/CSV).
4. Bulk env injection: `--env llama3:8b=KEY=VAL|KEY2=VAL2` syntax.
5. Default field flags: `--ttl 1800`, `--check-endpoint /health` applied to newly imported models only.
6. Grouping strategies: `--group-by family` to auto-create groups from a filter key or from name prefixes.
7. Sorting / stable ordering for deterministic diffs.
8. `--diff` mode to show changes vs existing config without writing.
9. JSON output option (`--format json`).
10. Exit codes signaling “no changes” (CI-friendly) with `--quiet`.

## Contributing
Issues & PRs are welcome. Please keep additions minimally invasive and documented. For feature ideas see the list above; feel free to open a discussion first.

## License
This crate is licensed under the [MIT License](https://github.com/Exotik850/ollama2llama/blob/master/LICENSE.md)
