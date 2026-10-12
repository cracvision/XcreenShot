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
    // $TMPDIR is per-user on macOS; fall back to the config dir elsewhere.
    let dir = std::env::var_os("TMPDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    dir.join("xcreenshot.sock").to_string_lossy().into_owned()
}

fn name(endpoint: &str) -> io::Result<Name<'_>> {
    #[cfg(windows)]
    {
        endpoint.to_ns_name::<interprocess::local_socket::GenericNamespaced>()
    }
    #[cfg(not(windows))]
    {
        if endpoint.starts_with('/') {
            endpoint.to_fs_name::<interprocess::local_socket::GenericFilePath>()
        } else {
            std::env::temp_dir()
                .join(endpoint)
                .to_string_lossy()
                .into_owned()
                .leak()
                .to_fs_name::<interprocess::local_socket::GenericFilePath>()
        }
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

/// Whether an instance is listening on `endpoint`.
pub fn is_running(endpoint: &str) -> bool {
    name(endpoint).and_then(Stream::connect).is_ok()
}

pub type RequestHandler = Arc<dyn Fn(Request) -> Response + Send + Sync>;

/// Binds the endpoint and serves requests on background threads. Fails with
/// `AddrInUse` if another instance already owns it (single instance, APP-001).
pub fn serve(endpoint: &str, handler: RequestHandler) -> io::Result<JoinHandle<()>> {
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
        use interprocess::os::unix::local_socket::ListenerOptionsExt;
        // Nobody is listening (checked above), so a leftover socket file is stale.
        opts.mode(0o600).try_overwrite(true)
    };
    let listener = opts.create_sync()?;
    std::thread::Builder::new().name("xs-ipc".into()).spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut conn) = conn else { continue };
            let handler = handler.clone();
            let _ = std::thread::Builder::new()
                .name("xs-ipc-conn".into())
                .spawn(move || {
                    let response = match read_line(&mut conn)
                        .and_then(|l| serde_json::from_str::<Request>(&l).map_err(io::Error::other))
                    {
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
    fn nothing_listening() {
        assert!(!is_running("xcreenshot-test-nobody-here"));
    }
}
