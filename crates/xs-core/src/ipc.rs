//! Local channel between the CLI (and later the MCP bridge) and the running app
//! (APP-001, APP-028, APP-032).
//!
//! - Windows: a named pipe scoped to the user and the session, with an ACL that only
//!   lets the owner (and SYSTEM) connect.
//! - macOS: a Unix domain socket in the per-user `$TMPDIR`, mode 0600.
//!
//! No network port is ever opened, not even on localhost. The protocol is one JSON
//! request line and one JSON response line per connection.

use std::io::{self, BufRead, BufReader, Write};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use interprocess::local_socket::{prelude::*, ListenerOptions, Name, Stream};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::command::Source;
use crate::error::CommandError;
use crate::i18n::Lang;

pub const PROTOCOL_VERSION: u32 = 1;
/// Requests are small; refuse anything absurd.
const MAX_LINE: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub v: u32,
    pub command: String,
    #[serde(default)]
    pub params: Value,
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<Lang>,
}

/// `{"ok": true, "result": …}` or `{"ok": false, "error": {…}}` — the same shape the
/// CLI prints with `--json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<CommandError>,
}

impl From<Result<Value, CommandError>> for Response {
    fn from(r: Result<Value, CommandError>) -> Self {
        match r {
            Ok(v) => Response {
                ok: true,
                result: Some(v),
                error: None,
            },
            Err(e) => Response {
                ok: false,
                result: None,
                error: Some(e),
            },
        }
    }
}

/// Endpoint identifier for the current user and session. `XCREENSHOT_IPC_NAME`
/// overrides it (tests).
pub fn endpoint() -> String {
    if let Ok(name) = std::env::var("XCREENSHOT_IPC_NAME") {
        return name;
    }
    platform_endpoint()
}

#[cfg(windows)]
fn platform_endpoint() -> String {
    use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    let mut session = 0u32;
    // SAFETY: plain out-parameter call.
    unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut session) };
    let user: String = std::env::var("USERNAME")
        .unwrap_or_default()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("xcreenshot-{user}-s{session}")
}

#[cfg(not(windows))]
fn platform_endpoint() -> String {
    "xcreenshot.sock".to_string()
}

/// Folder only this user can enter (0700). macOS cannot set a mode on the socket
/// itself, so access is enforced by the folder: connecting needs search permission on it.
#[cfg(unix)]
fn private_dir() -> std::path::PathBuf {
    // SAFETY: getuid never fails.
    let uid = unsafe { libc::getuid() };
    std::env::temp_dir().join(format!("xcreenshot-{uid}"))
}

#[cfg(unix)]
fn socket_path(endpoint: &str) -> std::path::PathBuf {
    if endpoint.starts_with('/') {
        std::path::PathBuf::from(endpoint)
    } else {
        private_dir().join(endpoint)
    }
}

/// Creates the private folder, or refuses one that another user owns or others can open.
#[cfg(unix)]
fn ensure_private_dir(path: &std::path::Path) -> io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    let Some(dir) = path.parent() else { return Ok(()) };
    if !dir.exists() {
        std::fs::DirBuilder::new().mode(0o700).create(dir)?;
    }
    let meta = std::fs::symlink_metadata(dir)?;
    // SAFETY: getuid never fails.
    if !meta.is_dir() || meta.uid() != unsafe { libc::getuid() } {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "socket folder is not ours",
        ));
    }
    if meta.permissions().mode() & 0o077 != 0 {
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn name(endpoint: &str) -> io::Result<Name<'_>> {
    #[cfg(windows)]
    {
        endpoint.to_ns_name::<interprocess::local_socket::GenericNamespaced>()
    }
    #[cfg(not(windows))]
    {
        socket_path(endpoint)
            .into_os_string()
            .to_fs_name::<interprocess::local_socket::GenericFilePath>()
    }
}

fn read_line(stream: &mut impl io::Read) -> io::Result<String> {
    let mut reader = BufReader::new(io::Read::take(&mut *stream, MAX_LINE));
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if line.is_empty() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "empty message"));
    }
    Ok(line)
}

/// Sends one request to the running instance. `timeout` bounds the whole exchange;
/// `None` waits forever (interactive commands, APP-028).
pub fn send(endpoint: &str, request: &Request, timeout: Option<Duration>) -> io::Result<Response> {
    let mut stream = Stream::connect(name(endpoint)?)?;
    let mut line = serde_json::to_string(request)?;
    line.push('\n');
    stream.write_all(line.as_bytes())?;
    stream.flush()?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(read_line(&mut stream));
    });
    let line = match timeout {
        Some(t) => rx
            .recv_timeout(t)
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "timed out"))??,
        None => rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "connection closed"))??,
    };
    serde_json::from_str(&line).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Sends a request whose answer comes in many lines (`events.watch`) and hands each
/// line to `on_line` until the app closes the connection or `on_line` returns `false`.
pub fn stream(endpoint: &str, request: &Request, mut on_line: impl FnMut(Value) -> bool) -> io::Result<()> {
    let mut stream = Stream::connect(name(endpoint)?)?;
    let mut line = serde_json::to_string(request)?;
    line.push('\n');
    stream.write_all(line.as_bytes())?;
    stream.flush()?;
    for line in BufReader::new(stream).lines() {
        let value: Value =
            serde_json::from_str(&line?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if !on_line(value) {
            break;
        }
    }
    Ok(())
}

/// Whether an instance is listening on `endpoint`.
pub fn is_running(endpoint: &str) -> bool {
    name(endpoint).and_then(Stream::connect).is_ok()
}

pub type RequestHandler = Arc<dyn Fn(Request) -> Response + Send + Sync>;

/// A request that answers with many lines (`events.watch`, APP-031): it calls `emit`
/// for each line until `emit` fails because the client left. Returns `false` when
/// the request is not one of these, so it gets one response as usual.
pub type StreamHandler = Arc<dyn Fn(&Request, &mut dyn FnMut(&Value) -> bool) -> bool + Send + Sync>;

/// Binds the endpoint and serves requests on background threads. Fails with
/// `AddrInUse` if another instance already owns it (single instance, APP-001).
pub fn serve(endpoint: &str, handler: RequestHandler) -> io::Result<JoinHandle<()>> {
    serve_with_streams(endpoint, handler, None)
}

/// [`serve`], with requests that stream their answer.
pub fn serve_with_streams(
    endpoint: &str,
    handler: RequestHandler,
    streams: Option<StreamHandler>,
) -> io::Result<JoinHandle<()>> {
    if is_running(endpoint) {
        return Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            "another instance is running",
        ));
    }
    let opts = ListenerOptions::new().name(name(endpoint)?.into_owned());
    #[cfg(windows)]
    let opts = {
        use interprocess::os::windows::local_socket::ListenerOptionsExt;
        use interprocess::os::windows::security_descriptor::SecurityDescriptor;
        // Protected DACL: full access for the owner (this user) and SYSTEM only.
        let sddl = widestring::U16CString::from_str("D:P(A;;GA;;;OW)(A;;GA;;;SY)")
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        opts.security_descriptor(SecurityDescriptor::deserialize(&sddl)?)
    };
    #[cfg(unix)]
    let opts = {
        ensure_private_dir(&socket_path(endpoint))?;
        // Nobody is listening (checked above), so a leftover socket file is stale.
        opts.try_overwrite(true)
    };
    let listener = opts.create_sync()?;
    std::thread::Builder::new().name("xs-ipc".into()).spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut conn) = conn else { continue };
            let handler = handler.clone();
            let streams = streams.clone();
            let _ = std::thread::Builder::new()
                .name("xs-ipc-conn".into())
                .spawn(move || {
                    let request = read_line(&mut conn)
                        .and_then(|l| serde_json::from_str::<Request>(&l).map_err(io::Error::other));
                    if let (Ok(req), Some(streams)) = (&request, &streams) {
                        if req.v == PROTOCOL_VERSION {
                            let mut emit = |v: &Value| {
                                let Ok(mut line) = serde_json::to_string(v) else {
                                    return false;
                                };
                                line.push('\n');
                                conn.write_all(line.as_bytes()).and_then(|_| conn.flush()).is_ok()
                            };
                            if streams(req, &mut emit) {
                                return;
                            }
                        }
                    }
                    let response = match request {
                        Ok(req) if req.v == PROTOCOL_VERSION => handler(req),
                        Ok(req) => Response::from(Err(CommandError::new(
                            crate::error::ErrorCode::InvalidParams,
                            format!("unsupported protocol version {}", req.v),
                        ))),
                        Err(e) => Response::from(Err(CommandError::new(
                            crate::error::ErrorCode::InvalidParams,
                            format!("malformed request: {e}"),
                        ))),
                    };
                    if let Ok(mut line) = serde_json::to_string(&response) {
                        line.push('\n');
                        let _ = conn.write_all(line.as_bytes());
                        let _ = conn.flush();
                    }
                });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_and_single_owner() {
        let endpoint = format!("xcreenshot-test-{}", std::process::id());
        let handler: RequestHandler =
            Arc::new(|req: Request| Response::from(Ok(json!({ "echo": req.command, "source": req.source }))));
        let _server = serve(&endpoint, handler.clone()).unwrap();
        assert!(is_running(&endpoint));
        let req = Request {
            v: PROTOCOL_VERSION,
            command: "app.ping".into(),
            params: json!({}),
            source: Source::Cli,
            lang: None,
        };
        let resp = send(&endpoint, &req, Some(Duration::from_secs(5))).unwrap();
        assert!(resp.ok);
        assert_eq!(resp.result.unwrap()["echo"], "app.ping");
        let second = serve(&endpoint, handler).unwrap_err();
        assert_eq!(second.kind(), io::ErrorKind::AddrInUse);
    }

    #[test]
    fn streams_many_lines() {
        let endpoint = format!("xcreenshot-test-stream-{}", std::process::id());
        let handler: RequestHandler = Arc::new(|_req: Request| Response::from(Ok(json!({}))));
        let streams: StreamHandler = Arc::new(|req: &Request, emit: &mut dyn FnMut(&Value) -> bool| {
            if req.command != "events.watch" {
                return false;
            }
            for i in 0..3 {
                emit(&json!({ "event": "test", "n": i }));
            }
            true
        });
        let _server = serve_with_streams(&endpoint, handler, Some(streams)).unwrap();
        let req = |command: &str| Request {
            v: PROTOCOL_VERSION,
            command: command.into(),
            params: json!({}),
            source: Source::Cli,
            lang: None,
        };
        let mut got = Vec::new();
        stream(&endpoint, &req("events.watch"), |v| {
            got.push(v["n"].as_i64().unwrap());
            true
        })
        .unwrap();
        assert_eq!(got, [0, 1, 2]);
        // Other requests still get one answer.
        assert!(
            send(&endpoint, &req("app.ping"), Some(Duration::from_secs(5)))
                .unwrap()
                .ok
        );
    }

    #[test]
    fn nothing_listening() {
        assert!(!is_running("xcreenshot-test-nobody-here"));
    }
}
