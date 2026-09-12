// SPDX-License-Identifier: AGPL-3.0-or-later
//! Protocol negotiation and framing use rmcp rather than a custom JSON-RPC loop.
use rmcp::{
    ErrorData, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RunInput {
    /// Complete NS2 source; no filesystem or network access is exposed.
    source: String,
}
#[derive(Clone, Debug)]
struct Tools {
    tool_router: ToolRouter<Self>,
}
#[tool_router]
impl Tools {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
    #[tool(description = "Execute exact Native Space 2 source and return its explicit output")]
    fn run(&self, Parameters(input): Parameters<RunInput>) -> Result<String, ErrorData> {
        super::run_source(&input.source, "mcp.ns").map_err(|e| ErrorData::invalid_params(e, None))
    }
}
#[tool_handler(router=self.tool_router)]
impl ServerHandler for Tools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("native-space", "2.0.0"))
    }
}
pub async fn run() -> Result<(), String> {
    let server = Tools::new()
        .serve(stdio())
        .await
        .map_err(|e| e.to_string())?;
    server.waiting().await.map_err(|e| e.to_string())?;
    Ok(())
}
