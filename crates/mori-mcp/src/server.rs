//! Stdio JSON-RPC server implementing the MCP protocol.
//!
//! Reads Content-Length framed messages from stdin, dispatches them, and
//! writes Content-Length framed responses to stdout.

use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};

use mori_index::Index;

use crate::protocol::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, ServerCapabilities, ServerInfo,
    ToolCallParams, ToolsCapability, ToolsListResult,
};
use crate::tools;

/// The MCP server state.
pub struct McpServer {
    index: Index,
    graph_built: bool,
}

impl McpServer {
    /// Create a new server rooted at the given project directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the index cannot be opened.
    pub fn new(root: &Path) -> anyhow::Result<Self> {
        let index = Index::open(root)?;
        Ok(Self {
            index,
            graph_built: false,
        })
    }

    /// Run the stdio server loop.
    ///
    /// Reads Content-Length framed JSON-RPC from stdin and writes framed
    /// responses to stdout. Logs go to stderr via tracing.
    ///
    /// # Errors
    ///
    /// Returns an error on fatal I/O failures.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);

        info!("mori MCP server running on stdio");

        loop {
            // Read Content-Length header.
            let content_length = match read_content_length(&mut reader).await {
                Ok(Some(len)) => len,
                Ok(None) => {
                    debug!("stdin closed, shutting down");
                    break;
                }
                Err(e) => {
                    error!("failed to read header: {e}");
                    break;
                }
            };

            // Read the JSON body.
            let mut body = vec![0u8; content_length];
            if let Err(e) = reader.read_exact(&mut body).await {
                error!("failed to read body: {e}");
                break;
            }

            let body_str = match String::from_utf8(body) {
                Ok(s) => s,
                Err(e) => {
                    error!("invalid UTF-8 in body: {e}");
                    continue;
                }
            };

            debug!("recv: {body_str}");

            let req: JsonRpcRequest = match serde_json::from_str(&body_str) {
                Ok(r) => r,
                Err(e) => {
                    warn!("invalid JSON-RPC request: {e}");
                    // Send parse error if we can.
                    let resp = JsonRpcResponse::error(
                        serde_json::Value::Null,
                        -32700,
                        format!("Parse error: {e}"),
                    );
                    write_response(&mut stdout, &resp).await?;
                    continue;
                }
            };

            // Check if this is a notification (no id or null id). Notifications
            // get handled but produce no response.
            let is_notification = req.id.as_ref().map_or(true, serde_json::Value::is_null);

            // Handle the request.
            let resp = self.handle_request(&req);

            if !is_notification {
                if let Some(response) = resp {
                    write_response(&mut stdout, &response).await?;
                }
            }
        }

        Ok(())
    }

    /// Dispatch a single JSON-RPC request.
    fn handle_request(&mut self, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone().unwrap_or(serde_json::Value::Null);

        match req.method.as_str() {
            "initialize" => {
                let result = InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability {}),
                    },
                    server_info: ServerInfo {
                        name: "mori".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };
                let value = serde_json::to_value(&result).unwrap_or_default();
                Some(JsonRpcResponse::success(id, value))
            }

            "notifications/initialized" => {
                // Build/refresh the index when the client signals ready.
                info!("client initialized, updating index...");
                match self.index.update() {
                    Ok(stats) => info!(
                        "index updated: {} files scanned, {} changed, {} symbols added",
                        stats.files_scanned, stats.files_changed, stats.symbols_added
                    ),
                    Err(e) => error!("index update failed: {e}"),
                }
                None // notification, no response
            }

            "tools/list" => {
                let result = ToolsListResult {
                    tools: tools::list_tools(),
                };
                let value = serde_json::to_value(&result).unwrap_or_default();
                Some(JsonRpcResponse::success(id, value))
            }

            "tools/call" => {
                let params: ToolCallParams = match req.params.as_ref() {
                    Some(p) => match serde_json::from_value(p.clone()) {
                        Ok(tc) => tc,
                        Err(e) => {
                            return Some(JsonRpcResponse::error(
                                id,
                                -32602,
                                format!("Invalid params: {e}"),
                            ));
                        }
                    },
                    None => {
                        return Some(JsonRpcResponse::error(id, -32602, "Missing params".into()));
                    }
                };

                // Ensure graph is built for tools that need it.
                if !self.graph_built
                    && (params.name == "get_symbol_context"
                        || params.name == "find_similar_patterns"
                        || params.name == "get_context")
                {
                    if let Err(e) = self.index.rebuild_graph() {
                        error!("graph rebuild failed: {e}");
                    } else {
                        self.graph_built = true;
                    }
                }

                let result = tools::call_tool(&mut self.index, &params.name, &params.arguments);
                let value = serde_json::to_value(&result).unwrap_or_default();
                Some(JsonRpcResponse::success(id, value))
            }

            _ => Some(JsonRpcResponse::error(
                id,
                -32601,
                format!("Method not found: {}", req.method),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Content-Length framing helpers
// ---------------------------------------------------------------------------

/// Read the `Content-Length` header from a buffered reader.
///
/// Returns `Ok(None)` on EOF (stdin closed).
async fn read_content_length<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> anyhow::Result<Option<usize>> {
    let mut header_line = String::new();

    // Read lines until we find Content-Length or hit EOF.
    loop {
        header_line.clear();
        let bytes_read = reader.read_line(&mut header_line).await?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let trimmed = header_line.trim();

        // Empty line signals end of headers.
        if trimmed.is_empty() {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let length: usize = value
                .trim()
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid Content-Length value: {e}"))?;

            // Consume the blank line after the header.
            let mut blank = String::new();
            reader.read_line(&mut blank).await?;

            return Ok(Some(length));
        }
        // Skip unknown headers.
    }
}

/// Write a JSON-RPC response with Content-Length framing.
async fn write_response<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    resp: &JsonRpcResponse,
) -> anyhow::Result<()> {
    let body = serde_json::to_string(resp)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());

    debug!("send: {body}");

    writer.write_all(header.as_bytes()).await?;
    writer.write_all(body.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}
