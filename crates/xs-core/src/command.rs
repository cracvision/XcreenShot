//! Command registry (APP-027). Every action of the app is a command with a stable
//! ID, typed parameters and a typed result. The UI, shortcuts, tray menu, CLI and
//! (from M2) MCP all go through [`Registry::call`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{CommandError, CommandResult, ErrorCode};
use crate::i18n::{tr, Lang};
use crate::schema;

/// Where a command may be invoked from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Ui,
    Shortcut,
    Cli,
    Mcp,
}

/// Who invoked a command. Reported in events and logs (APP-031, APP-032).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Ui,
    Shortcut,
    Cli,
    Mcp,
    Scheduled,
}

impl Source {
    pub fn scope(self) -> Scope {
        match self {
            Source::Ui | Source::Scheduled => Scope::Ui,
            Source::Shortcut => Scope::Shortcut,
            Source::Cli => Scope::Cli,
            Source::Mcp => Scope::Mcp,
        }
    }

    /// CLI and MCP are "automation" for APP-032.
    pub fn is_automation(self) -> bool {
        matches!(self, Source::Cli | Source::Mcp)
    }
}

/// Static description of a command. Titles and descriptions live in the locale
/// files under `commands.<id with '.' replaced by '_'>`.
#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub id: &'static str,
    /// JSON Schema of the parameters object.
    pub params: Value,
    /// JSON Schema of the result.
    pub result: Value,
    pub scopes: &'static [Scope],
    /// Needs the user (e.g. an on-screen selection).
    pub interactive: bool,
    /// Changes data (false = read-only).
    pub mutates: bool,
    /// Goes out to the network.
    pub network: bool,
    /// Deletes or overwrites user data; the CLI demands `--yes`.
    pub destructive: bool,
    /// A shortcut can be assigned to it (APP-022).
    pub assignable: bool,
    /// Parameters the CLI also accepts positionally, in order
    /// (`xcreenshot settings set <key> <value>`).
    pub positional: &'static [&'static str],
    /// Spec IDs this command implements.
    pub spec_ids: &'static [&'static str],
}

impl CommandSpec {
    pub fn i18n_base(&self) -> String {
        format!("commands.{}", self.id.replace('.', "_"))
    }

    pub fn title(&self, lang: Lang) -> String {
        tr(lang, &format!("{}.title", self.i18n_base()), &[])
    }

    pub fn description(&self, lang: Lang) -> String {
        tr(lang, &format!("{}.description", self.i18n_base()), &[])
    }

    pub fn has_scope(&self, scope: Scope) -> bool {
        self.scopes.contains(&scope)
    }

    /// `area` part of `area.action`.
    pub fn area(&self) -> &'static str {
        self.id.split_once('.').map(|(a, _)| a).unwrap_or(self.id)
    }

    /// `action` part of `area.action`.
    pub fn action(&self) -> &'static str {
        self.id.split_once('.').map(|(_, a)| a).unwrap_or("")
    }

    /// Public description used by `app.commands`, the CLI help and MCP.
    pub fn describe(&self, lang: Lang) -> Value {
        json!({
            "id": self.id,
            "title": self.title(lang),
            "description": self.description(lang),
            "params": self.params,
            "result": self.result,
            "scopes": self.scopes,
            "interactive": self.interactive,
            "mutates": self.mutates,
            "network": self.network,
            "destructive": self.destructive,
            "assignable": self.assignable,
            "specIds": self.spec_ids,
        })
    }
}

/// Per-call context.
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub source: Source,
    pub lang: Lang,
}

pub type Handler = Box<dyn Fn(&Context, Value) -> CommandResult + Send + Sync>;
/// Runs before every call; may veto it (automation switch, managed policy).
pub type Gate = Box<dyn Fn(&Context, &CommandSpec) -> Result<(), CommandError> + Send + Sync>;
/// Runs after every call (logging, UI events).
pub type Observer = Box<dyn Fn(&Context, &CommandSpec, &CommandResult) + Send + Sync>;

#[derive(Default)]
pub struct Registry {
    specs: Vec<CommandSpec>,
    index: HashMap<&'static str, usize>,
    handlers: HashMap<&'static str, Handler>,
    gates: Vec<Gate>,
    observers: Vec<Observer>,
}

impl Registry {
    pub fn new(specs: Vec<CommandSpec>) -> Self {
        let mut reg = Registry::default();
        for spec in specs {
            assert!(
                !reg.index.contains_key(spec.id),
                "duplicate command id {}",
                spec.id
            );
            reg.index.insert(spec.id, reg.specs.len());
            reg.specs.push(spec);
        }
        reg
    }

    pub fn handle<F>(&mut self, id: &'static str, f: F)
    where
        F: Fn(&Context, Value) -> CommandResult + Send + Sync + 'static,
    {
        assert!(self.index.contains_key(id), "handler for unknown command {id}");
        self.handlers.insert(id, Box::new(f));
    }

    pub fn add_gate(&mut self, gate: Gate) {
        self.gates.push(gate);
    }

    pub fn add_observer(&mut self, observer: Observer) {
        self.observers.push(observer);
    }

    pub fn spec(&self, id: &str) -> Option<&CommandSpec> {
        self.index.get(id).map(|&i| &self.specs[i])
    }

    pub fn specs(&self) -> &[CommandSpec] {
        &self.specs
    }

    pub fn has_handler(&self, id: &str) -> bool {
        self.handlers.contains_key(id)
    }

    pub fn call(&self, ctx: &Context, id: &str, params: Value) -> CommandResult {
        let Some(spec) = self.spec(id) else {
            return Err(CommandError::new(
                ErrorCode::UnknownCommand,
                tr(ctx.lang, "errors.unknown_command", &[("id", id)]),
            ));
        };
        let result = self.call_spec(ctx, spec, params);
        for observer in &self.observers {
            observer(ctx, spec, &result);
        }
        result
    }

    fn call_spec(&self, ctx: &Context, spec: &CommandSpec, params: Value) -> CommandResult {
        if !spec.has_scope(ctx.source.scope()) {
            return Err(CommandError::new(
                ErrorCode::InvalidParams,
                tr(ctx.lang, "errors.scope_not_allowed", &[("id", spec.id)]),
            ));
        }
        let params = if params.is_null() { json!({}) } else { params };
        if let Err(e) = schema::validate(&spec.params, &params) {
            return Err(CommandError::new(
                ErrorCode::InvalidParams,
                tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", &e.path), ("reason", &e.reason)],
                ),
            )
            .with_details(json!({ "param": e.path, "reason": e.reason })));
        }
        for gate in &self.gates {
            gate(ctx, spec)?;
        }
        let handler = self
            .handlers
            .get(spec.id)
            .ok_or_else(|| CommandError::internal(format!("no handler for {}", spec.id)))?;
        handler(ctx, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &'static str) -> CommandSpec {
        CommandSpec {
            id,
            params: json!({"type": "object", "properties": {"n": {"type": "integer"}}, "additionalProperties": false}),
            result: json!({"type": "integer"}),
            scopes: &[Scope::Ui, Scope::Cli],
            interactive: false,
            mutates: false,
            network: false,
            destructive: false,
            assignable: false,
            positional: &[],
            spec_ids: &["APP-027"],
        }
    }

    fn ctx(source: Source) -> Context {
        Context {
            source,
            lang: Lang::En,
        }
    }

    #[test]
    fn calls_handler_with_validated_params() {
        let mut reg = Registry::new(vec![spec("test.echo")]);
        reg.handle("test.echo", |_, p| Ok(p["n"].clone()));
        let r = reg.call(&ctx(Source::Cli), "test.echo", json!({"n": 3})).unwrap();
        assert_eq!(r, json!(3));
    }

    #[test]
    fn unknown_param_is_invalid_params_and_named() {
        let mut reg = Registry::new(vec![spec("test.echo")]);
        reg.handle("test.echo", |_, p| Ok(p));
        let e = reg
            .call(&ctx(Source::Ui), "test.echo", json!({"zzz": 1}))
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParams);
        assert!(e.message.contains("zzz"), "{}", e.message);
    }

    #[test]
    fn rejects_out_of_scope_source_and_unknown_command() {
        let mut reg = Registry::new(vec![spec("test.echo")]);
        reg.handle("test.echo", |_, p| Ok(p));
        let e = reg.call(&ctx(Source::Mcp), "test.echo", json!({})).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParams);
        let e = reg.call(&ctx(Source::Ui), "nope.nope", json!({})).unwrap_err();
        assert_eq!(e.code, ErrorCode::UnknownCommand);
    }

    #[test]
    fn gate_can_veto() {
        let mut reg = Registry::new(vec![spec("test.echo")]);
        reg.handle("test.echo", |_, p| Ok(p));
        reg.add_gate(Box::new(|c, _| {
            if c.source.is_automation() {
                Err(CommandError::new(ErrorCode::DisabledByPolicy, "off"))
            } else {
                Ok(())
            }
        }));
        assert!(reg.call(&ctx(Source::Ui), "test.echo", json!({})).is_ok());
        let e = reg.call(&ctx(Source::Cli), "test.echo", json!({})).unwrap_err();
        assert_eq!(e.code.exit_code(), 6);
    }
}
