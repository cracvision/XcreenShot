//! What two computers say to each other inside the encrypted connection (SHR-043,
//! SHR-044): messages are a 4-byte length and JSON; a file's bytes follow its `file`
//! message as they are.

use std::io::{Read, Write};

use serde_json::Value;

/// The largest message (not a file): far more than any offer needs.
const MAX_MESSAGE: u32 = 4 << 20;

pub fn send(w: &mut impl Write, msg: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(msg)?;
    w.write_all(&(body.len() as u32).to_be_bytes())?;
    w.write_all(&body)?;
    w.flush()
}

pub fn read(r: &mut impl Read) -> std::io::Result<Value> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_be_bytes(len);
    if len > MAX_MESSAGE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "message too large",
        ));
    }
    let mut body = vec![0u8; len as usize];
    r.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Whether an I/O error is only a read that timed out (nothing came yet).
pub fn timed_out(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn messages_go_and_come_back() {
        let mut buf = Vec::new();
        send(&mut buf, &json!({"t": "offer", "n": 2})).unwrap();
        send(&mut buf, &json!({"t": "end"})).unwrap();
        let mut r = std::io::Cursor::new(buf);
        assert_eq!(read(&mut r).unwrap()["n"], 2);
        assert_eq!(read(&mut r).unwrap()["t"], "end");
        assert!(read(&mut r).is_err());
    }
}
