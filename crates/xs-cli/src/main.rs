//! `xcreenshot` command-line interface (APP-028).
//!
//! The command tree is generated from the registry catalog (APP-027):
//! `xcreenshot <area> <action> [positional…] [--param value…]` runs `<area>.<action>`.
//! The CLI talks to the running app over the local channel (APP-032) and starts it
//! in the background when needed.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::{Arg, ArgAction, ArgMatches, Command};
use serde_json::{json, Map, Value};
use xs_core::catalog;
use xs_core::i18n::tr;
use xs_core::ipc::{self, Request, Response, PROTOCOL_VERSION};
use xs_core::paths::Paths;
use xs_core::settings::SettingsStore;
use xs_core::{CommandError, CommandSpec, ErrorCode, Lang, Source};

/// `camelCase` / `snake_case` parameter name to `--kebab-case`.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else if c == '_' {
            out.push('-');
        } else {
            out.push(c);
        }
    }
    out
}

/// Language for readable messages: `--lang`, else the app setting, else the system.
fn detect_lang(args: &[String]) -> Lang {
    for (i, a) in args.iter().enumerate() {
        let value = a.strip_prefix("--lang=").or_else(|| {
            (a == "--lang")
                .then(|| args.get(i + 1).map(String::as_str))
                .flatten()
        });
        if let Some(lang) = value.and_then(Lang::parse) {
            return lang;
        }
    }
    SettingsStore::load(Paths::standard().settings_file()).lang()
}

fn type_of(schema: &Value) -> Vec<&str> {
    match schema.get("type") {
        Some(Value::String(t)) => vec![t.as_str()],
        Some(Value::Array(ts)) => ts.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    }
}

fn param_arg(name: &str, schema: &Value, positional_index: Option<usize>) -> Arg {
    let id = name.to_string();
    let mut arg = match positional_index {
        Some(i) => Arg::new(id.clone()).index(i + 1).required(false),
        None => Arg::new(id.clone()).long(kebab(name)),
    };
    // A positional parameter can also be passed by name.
    if positional_index.is_some() {
        arg = arg.value_name(name.to_uppercase());
    }
    let types = type_of(schema);
    if types == ["boolean"] && positional_index.is_none() {
        arg = arg
            .num_args(0..=1)
            .default_missing_value("true")
            .value_parser(["true", "false"]);
    } else if let Some(Value::Array(options)) = schema.get("enum") {
        let values: Vec<String> = options
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        arg = arg.value_parser(clap::builder::PossibleValuesParser::new(values));
    }
    arg
}

fn action_command(spec: &CommandSpec, lang: Lang) -> Command {
    let mut cmd = Command::new(kebab(spec.action()))
        .about(spec.title(lang))
        .long_about(format!("{}\n\nID: {}", spec.description(lang), spec.id))
        .arg(
            Arg::new("__params")
                .long("params")
                .value_name("JSON")
                .help(tr(lang, "cli.params_json", &[])),
        );
    let props = spec.params["properties"].as_object().cloned().unwrap_or_default();
    for (i, name) in spec.positional.iter().enumerate() {
        if let Some(schema) = props.get(*name) {
            cmd = cmd.arg(param_arg(name, schema, Some(i)));
        }
    }
    for (name, schema) in &props {
        if spec.positional.contains(&name.as_str()) {
            continue;
        }
        cmd = cmd.arg(param_arg(name, schema, None));
    }
    if spec.destructive {
        cmd = cmd.arg(Arg::new("__yes").long("yes").action(ArgAction::SetTrue).help(tr(
            lang,
            "cli.opt_yes",
            &[],
        )));
    }
    cmd
}

fn build_cli(lang: Lang, specs: &[CommandSpec]) -> Command {
    let mut root = Command::new("xcreenshot")
        .version(xs_core::platform::VERSION)
        .about(tr(lang, "cli.about", &[]))
        .subcommand_required(true)
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help(tr(lang, "cli.opt_json", &[])),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .global(true)
                .action(ArgAction::SetTrue)
                .help(tr(lang, "cli.opt_quiet", &[])),
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .global(true)
                .value_name("SECONDS")
                .value_parser(clap::value_parser!(f64))
                .help(tr(lang, "cli.opt_timeout", &[])),
        )
        .arg(
            Arg::new("no_launch")
                .long("no-launch")
                .global(true)
                .action(ArgAction::SetTrue)
                .help(tr(lang, "cli.opt_no_launch", &[])),
        )
        .arg(
            Arg::new("lang")
                .long("lang")
                .global(true)
                .value_parser(["en", "es"])
                .help(tr(lang, "cli.opt_lang", &[])),
        );
    let mut areas: Vec<&str> = specs
        .iter()
        .filter(|s| s.has_scope(xs_core::Scope::Cli))
        .map(|s| s.area())
        .collect();
    areas.dedup();
    areas.sort();
    areas.dedup();
    for area in areas {
        let mut sub = Command::new(area.to_string()).subcommand_required(true);
        for spec in specs
            .iter()
            .filter(|s| s.area() == area && s.has_scope(xs_core::Scope::Cli))
        {
            sub = sub.subcommand(action_command(spec, lang));
        }
        root = root.subcommand(sub);
    }
    root.subcommand(
        Command::new("run")
            .about(tr(lang, "cli.run_about", &[]))
            .arg(Arg::new("id").required(true))
            .arg(Arg::new("__params").long("params").value_name("JSON").help(tr(
                lang,
                "cli.params_json",
                &[],
            )))
            .arg(
                Arg::new("__yes")
                    .long("yes")
                    .action(ArgAction::SetTrue)
                    .help(tr(lang, "cli.opt_yes", &[])),
            ),
    )
    .subcommand(
        Command::new("help")
            .about(tr(lang, "cli.help_about", &[]))
            .arg(Arg::new("command").required(false)),
    )
    .disable_help_subcommand(true)
}

/// Converts a CLI string to the JSON type the schema expects.
fn convert(raw: &str, schema: &Value) -> Result<Value, String> {
    let types = type_of(schema);
    if types.contains(&"null") && raw.is_empty() {
        return Ok(Value::Null);
    }
    if types.is_empty() {
        // Untyped (e.g. settings.set value): JSON if it parses, else a string.
        return Ok(serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string())));
    }
    for t in &types {
        match *t {
            "boolean" => match raw {
                "true" | "yes" | "on" | "1" => return Ok(json!(true)),
                "false" | "no" | "off" | "0" => return Ok(json!(false)),
                _ => {}
            },
            "integer" => {
                if let Ok(n) = raw.parse::<i64>() {
                    return Ok(json!(n));
                }
            }
            "number" => {
                if let Ok(n) = raw.parse::<f64>() {
                    return Ok(json!(n));
                }
            }
            "object" | "array" => {
                if let Ok(v) = serde_json::from_str(raw) {
                    return Ok(v);
                }
            }
            "string" => return Ok(Value::String(raw.to_string())),
            _ => {}
        }
    }
    Err(format!("expected {}", types.join(" or ")))
}

fn read_params_arg(raw: Option<&String>) -> Result<Map<String, Value>, String> {
    let Some(raw) = raw else { return Ok(Map::new()) };
    let text = if raw == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| e.to_string())?;
        s
    } else {
        raw.clone()
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => Ok(m),
        Ok(_) => Err("--params must be a JSON object".into()),
        Err(e) => Err(format!("--params: {e}")),
    }
}

fn collect_params(spec: &CommandSpec, m: &ArgMatches) -> Result<Value, String> {
    let mut params = read_params_arg(m.get_one::<String>("__params"))?;
    let props = spec.params["properties"].as_object().cloned().unwrap_or_default();
    for (name, schema) in &props {
        if let Some(raw) = m.get_one::<String>(name) {
            let v = convert(raw, schema).map_err(|e| format!("{name}: {e}"))?;
            params.insert(name.clone(), v);
        }
    }
    Ok(Value::Object(params))
}

fn app_binary() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("XCREENSHOT_APP") {
        return Some(PathBuf::from(p));
    }
    let exe = std::env::current_exe().ok()?;
    let name = if cfg!(windows) {
        "xcreenshot-app.exe"
    } else {
        "xcreenshot-app"
    };
    let candidate = exe.parent()?.join(name);
    candidate.exists().then_some(candidate)
}

/// Starts the app in the background (no windows) and waits until it listens.
fn launch(endpoint: &str, lang: Lang, quiet: bool) -> bool {
    let Some(bin) = app_binary() else { return false };
    if !quiet {
        eprintln!("{}", tr(lang, "cli.launching", &[]));
    }
    if std::process::Command::new(bin)
        .arg("--background")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_err()
    {
        return false;
    }
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        if ipc::is_running(endpoint) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

struct Output {
    json: bool,
    quiet: bool,
}

impl Output {
    fn finish(&self, response: Response, lang: Lang) -> ExitCode {
        if self.json {
            println!("{}", serde_json::to_string(&response).unwrap_or_default());
        }
        match (response.ok, response.error) {
            (true, _) => {
                if !self.json && !self.quiet {
                    match response.result {
                        Some(Value::Object(m)) if m.is_empty() => println!("{}", tr(lang, "cli.ok", &[])),
                        Some(Value::String(s)) => println!("{s}"),
                        Some(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default()),
                        None => println!("{}", tr(lang, "cli.ok", &[])),
                    }
                }
                ExitCode::SUCCESS
            }
            (false, Some(e)) => {
                if !self.json {
                    eprintln!("{} ({})", e.message, e.code.as_str());
                }
                ExitCode::from(e.code.exit_code() as u8)
            }
            (false, None) => ExitCode::from(1),
        }
    }

    fn fail(&self, e: CommandError, lang: Lang) -> ExitCode {
        self.finish(Response::from(Err(e)), lang)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let lang = detect_lang(&args);
    let specs = catalog::specs();
    let cli = build_cli(lang, &specs);
    let matches = match cli.clone().try_get_matches_from(&args) {
        Ok(m) => m,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return ExitCode::from(code);
        }
    };
    let out = Output {
        json: matches.get_flag("json"),
        quiet: matches.get_flag("quiet"),
    };
    let timeout = matches
        .get_one::<f64>("timeout")
        .map(|s| Duration::from_secs_f64(*s));
    let no_launch = matches.get_flag("no_launch");

    let (spec, params, yes) = match matches.subcommand() {
        Some(("help", m)) => {
            let mut cli = cli;
            return match m.get_one::<String>("command") {
                None => {
                    let _ = cli.print_long_help();
                    ExitCode::SUCCESS
                }
                Some(id) => {
                    let (area, action) = id.split_once('.').unwrap_or((id.as_str(), ""));
                    let found = cli.find_subcommand_mut(area).and_then(|a| {
                        if action.is_empty() {
                            Some(a)
                        } else {
                            a.find_subcommand_mut(kebab(action))
                        }
                    });
                    match found {
                        Some(c) => {
                            let mut c = c.clone();
                            let _ = c.print_long_help();
                            ExitCode::SUCCESS
                        }
                        None => {
                            eprintln!("{}", tr(lang, "cli.unknown_area", &[("name", id)]));
                            ExitCode::from(2)
                        }
                    }
                }
            };
        }
        Some(("run", m)) => {
            let id = m.get_one::<String>("id").cloned().unwrap_or_default();
            let Some(spec) = specs.iter().find(|s| s.id == id) else {
                return out.fail(
                    CommandError::new(
                        ErrorCode::UnknownCommand,
                        tr(lang, "errors.unknown_command", &[("id", &id)]),
                    ),
                    lang,
                );
            };
            let params = match read_params_arg(m.get_one::<String>("__params")) {
                Ok(p) => Value::Object(p),
                Err(e) => return out.fail(CommandError::new(ErrorCode::InvalidParams, e), lang),
            };
            (spec, params, m.get_flag("__yes"))
        }
        Some((area, m)) => {
            let Some((action, am)) = m.subcommand() else {
                return ExitCode::from(2);
            };
            let id = format!("{area}.{}", action.replace('-', "_"));
            let Some(spec) = specs.iter().find(|s| s.id == id) else {
                return out.fail(
                    CommandError::new(
                        ErrorCode::UnknownCommand,
                        tr(lang, "errors.unknown_command", &[("id", &id)]),
                    ),
                    lang,
                );
            };
            let params = match collect_params(spec, am) {
                Ok(p) => p,
                Err(e) => {
                    return out.fail(
                        CommandError::new(
                            ErrorCode::InvalidParams,
                            tr(
                                lang,
                                "errors.invalid_param",
                                &[
                                    ("param", e.split(':').next().unwrap_or("")),
                                    ("reason", e.split_once(": ").map(|x| x.1).unwrap_or(&e)),
                                ],
                            ),
                        ),
                        lang,
                    )
                }
            };
            let yes = spec.destructive && am.get_flag("__yes");
            (spec, params, yes)
        }
        None => return ExitCode::from(2),
    };

    if spec.destructive && !yes {
        return out.fail(
            CommandError::new(ErrorCode::InvalidParams, tr(lang, "cli.needs_yes", &[])),
            lang,
        );
    }

    let endpoint = ipc::endpoint();
    if !ipc::is_running(&endpoint) && (no_launch || !launch(&endpoint, lang, out.quiet || out.json)) {
        return out.fail(
            CommandError::new(ErrorCode::NotRunning, tr(lang, "cli.not_running", &[])),
            lang,
        );
    }
    let request = Request {
        v: PROTOCOL_VERSION,
        command: spec.id.to_string(),
        params,
        source: Source::Cli,
        lang: Some(lang),
    };
    let _ = std::io::stderr().flush();
    match ipc::send(&endpoint, &request, timeout) {
        Ok(response) => out.finish(response, lang),
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
            out.fail(CommandError::new(ErrorCode::Timeout, e.to_string()), lang)
        }
        Err(e) => out.fail(
            CommandError::new(
                ErrorCode::NotRunning,
                format!("{}: {e}", tr(lang, "cli.not_running", &[])),
            ),
            lang,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_names() {
        assert_eq!(kebab("resetCustom"), "reset-custom");
        assert_eq!(kebab("repeat_last"), "repeat-last");
    }

    #[test]
    fn converts_by_schema() {
        assert_eq!(convert("false", &json!({})).unwrap(), json!(false));
        assert_eq!(convert("dark", &json!({})).unwrap(), json!("dark"));
        assert_eq!(
            convert("", &json!({"type": ["string", "null"]})).unwrap(),
            Value::Null
        );
        assert_eq!(convert("5", &json!({"type": "integer"})).unwrap(), json!(5));
        assert!(convert("x", &json!({"type": "integer"})).is_err());
    }

    #[test]
    fn every_cli_command_parses() {
        let specs = catalog::specs();
        let cli = build_cli(Lang::En, &specs);
        cli.clone().debug_assert();
        let m = cli
            .clone()
            .try_get_matches_from(["xcreenshot", "settings", "set", "general.theme", "dark", "--json"])
            .unwrap();
        let (_, sm) = m.subcommand().unwrap();
        let (_, am) = sm.subcommand().unwrap();
        let spec = specs.iter().find(|s| s.id == "settings.set").unwrap();
        assert_eq!(
            collect_params(spec, am).unwrap(),
            json!({"key": "general.theme", "value": "dark"})
        );
        let m = cli
            .try_get_matches_from([
                "xcreenshot",
                "shortcuts",
                "set-profile",
                "compatibility",
                "--reset-custom",
                "--dry-run",
            ])
            .unwrap();
        let (_, sm) = m.subcommand().unwrap();
        let (_, am) = sm.subcommand().unwrap();
        let spec = specs.iter().find(|s| s.id == "shortcuts.set_profile").unwrap();
        assert_eq!(
            collect_params(spec, am).unwrap(),
            json!({"profile": "compatibility", "resetCustom": true, "dryRun": true})
        );
    }
}
