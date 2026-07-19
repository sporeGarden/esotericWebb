// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP/1.1 POST transport for JSON-RPC 2.0.
//!
//! Speaks JSON-RPC over a single-shot HTTP/1.1 connection (no keep-alive).
//! Used for primals like songBird that expose `/jsonrpc` rather than raw
//! NDJSON TCP streams. Zero external deps — raw `TcpStream` + manual framing.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use super::envelope::{IpcError, JsonRpcRequest, JsonRpcResponse, classify_io_error};

/// Execute a single JSON-RPC request over HTTP/1.1 POST.
///
/// Opens a new TCP connection for each call (Connection: close). Parses
/// the HTTP response, extracts the JSON body, and deserializes into a
/// [`JsonRpcResponse`].
///
/// # Errors
///
/// Returns [`IpcError`] on connection failure, non-200 responses, or
/// body parse errors.
pub(super) fn call(
    addr: &str,
    path: &str,
    request: &JsonRpcRequest,
    timeout: std::time::Duration,
) -> Result<JsonRpcResponse, IpcError> {
    let body = serde_json::to_string(request).map_err(|e| IpcError::Serialization {
        detail: e.to_string(),
    })?;

    let http_request = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {addr}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {len}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        len = body.len(),
    );

    let mut stream = TcpStream::connect(addr).map_err(|e| classify_io_error(&e))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| classify_io_error(&e))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| classify_io_error(&e))?;
    stream
        .write_all(http_request.as_bytes())
        .map_err(|e| classify_io_error(&e))?;

    let mut reader = BufReader::new(&stream);
    let mut status_line = String::new();
    reader
        .read_line(&mut status_line)
        .map_err(|e| classify_io_error(&e))?;

    if !status_line.contains("200") {
        let code = status_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("?")
            .to_owned();
        return Err(IpcError::ProtocolError {
            detail: format!("HTTP {code} from {addr}{path}"),
        });
    }

    let mut content_length: usize = 0;
    loop {
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|e| classify_io_error(&e))?;
        if header.trim().is_empty() {
            break;
        }
        if let Some(val) = header.strip_prefix("Content-Length:") {
            content_length = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = header.strip_prefix("content-length:") {
            content_length = val.trim().parse().unwrap_or(0);
        }
    }

    let response_body = if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        std::io::Read::read_exact(&mut reader, &mut buf).map_err(|e| classify_io_error(&e))?;
        String::from_utf8_lossy(&buf).into_owned()
    } else {
        let mut buf = String::new();
        reader
            .read_line(&mut buf)
            .map_err(|e| classify_io_error(&e))?;
        buf
    };

    if response_body.trim().is_empty() {
        return Err(IpcError::ProtocolError {
            detail: "empty HTTP response body".to_owned(),
        });
    }

    serde_json::from_str::<JsonRpcResponse>(response_body.trim()).map_err(|e| {
        IpcError::ProtocolError {
            detail: format!("HTTP response parse error: {e}"),
        }
    })
}
