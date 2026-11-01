use std::sync::Arc;
use anyhow::Result;
use serde_json::json;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::storage::Database;
use super::prompts::{get_prompt_messages, get_prompts};
use super::resources::{get_resources, read_resource};
use super::tools::{call_tool, get_tools};
use super::types::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, PromptsCapability, ResourcesCapability,
    ServerCapabilities, ServerInfo, ToolsCapability, INVALID_PARAMS, METHOD_NOT_FOUND,
    MCP_PROTOCOL_VERSION, PARSE_ERROR,
};

/// Runs the MCP server over standard input and output (stdio).
pub async fn run_stdio(db: Database) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    run_mcp_server(reader, stdout, Arc::new(db)).await
}

/// Runs the MCP server loop with generic async reader and writer.
pub async fn run_mcp_server<R, W>(reader: R, mut writer: W, db: Arc<Database>) -> Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = JsonRpcResponse::error(
                    serde_json::Value::Null,
                    PARSE_ERROR,
                    format!("Parse error: {}", e),
                );
                let resp_str = serde_json::to_string(&err_resp)?;
                writer.write_all(resp_str.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                continue;
            }
        };

        // If request has no ID, it is a notification; per JSON-RPC 2.0, do not send a reply.
        let id = match req.id {
            Some(id) => id,
            None => continue,
        };

        let response = match handle_request(&req.method, req.params, &db).await {
            Ok(result) => JsonRpcResponse::success(id, result),
            Err((code, msg)) => JsonRpcResponse::error(id, code, msg),
        };

        let resp_str = serde_json::to_string(&response)?;
        writer.write_all(resp_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

async fn handle_request(
    method: &str,
    params: Option<serde_json::Value>,
    db: &Database,
) -> Result<serde_json::Value, (i64, String)> {
    match method {
        "initialize" => {
            let res = InitializeResult {
                protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                capabilities: ServerCapabilities {
                    tools: Some(ToolsCapability { list_changed: false }),
                    resources: Some(ResourcesCapability {
                        subscribe: false,
                        list_changed: false,
                    }),
                    prompts: Some(PromptsCapability { list_changed: false }),
                },
                server_info: ServerInfo {
                    name: "rcal".to_string(),
                    version: crate::PKG_VERSION.to_string(),
                },
                instructions: Some(
                    "rcal is a fast, keyboard/mouse-driven calendar with multi-cloud sync. Use MCP tools to manage calendar events, query schedules, search events, and find free slots for meetings.".to_string(),
                ),
            };
            serde_json::to_value(res).map_err(|e| (-32603, format!("Serialization error: {}", e)))
        }

        "ping" => Ok(json!({})),

        "tools/list" => {
            let tools = get_tools();
            Ok(json!({ "tools": tools }))
        }

        "tools/call" => {
            let params = params.ok_or_else(|| (INVALID_PARAMS, "Missing params for tools/call".to_string()))?;
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| (INVALID_PARAMS, "Missing 'name' in tools/call params".to_string()))?;
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            let result = call_tool(name, args, db).await;
            serde_json::to_value(result).map_err(|e| (-32603, format!("Serialization error: {}", e)))
        }

        "resources/list" => {
            let resources = get_resources();
            Ok(json!({ "resources": resources }))
        }

        "resources/read" => {
            let params = params.ok_or_else(|| (INVALID_PARAMS, "Missing params for resources/read".to_string()))?;
            let uri = params
                .get("uri")
                .and_then(|v| v.as_str())
                .ok_or_else(|| (INVALID_PARAMS, "Missing 'uri' in resources/read params".to_string()))?;

            match read_resource(uri, db) {
                Ok(content) => Ok(json!({ "contents": [content] })),
                Err(e) => Err((INVALID_PARAMS, e.to_string())),
            }
        }

        "prompts/list" => {
            let prompts = get_prompts();
            Ok(json!({ "prompts": prompts }))
        }

        "prompts/get" => {
            let params = params.ok_or_else(|| (INVALID_PARAMS, "Missing params for prompts/get".to_string()))?;
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| (INVALID_PARAMS, "Missing 'name' in prompts/get params".to_string()))?;
            let args = params.get("arguments").cloned();

            match get_prompt_messages(name, args, db) {
                Ok((description, messages)) => Ok(json!({
                    "description": description,
                    "messages": messages,
                })),
                Err(e) => Err((INVALID_PARAMS, e.to_string())),
            }
        }

        _ => Err((METHOD_NOT_FOUND, format!("Method not found: '{}'", method))),
    }
}
