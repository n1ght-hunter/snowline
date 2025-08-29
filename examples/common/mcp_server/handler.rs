use async_trait::async_trait;
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, RpcError,
    schema_utils::CallToolError,
};
use rust_mcp_sdk::{McpServer, mcp_server::ServerHandler};

use super::AppSender;
use super::tools::IcedTool;

// Custom Handler to handle MCP Messages
pub struct MyServerHandler {
    pub sender: AppSender,
}

#[async_trait]
#[allow(unused)]
impl ServerHandler for MyServerHandler {
    // Handle ListToolsRequest, return list of available tools as ListToolsResult
    async fn handle_list_tools_request(
        &self,
        request: ListToolsRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: IcedTool::all_tools(),
        })
    }

    /// Handles incoming CallToolRequest and processes it using the appropriate tool.
    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        // Attempt to convert request parameters into IcedTool enum
        let tool_params: IcedTool =
            IcedTool::from_str(&request.params.name).map_err(CallToolError::new)?;
        // Extract arguments before moving params
        let args = request.params.arguments.map(|v| serde_json::Value::Object(v)).unwrap_or_default();


        // Match the tool variant and execute its corresponding logic
        let result = tool_params.call_tool(args, self).await;

        result.map_err(|e| {
            CallToolError(e)
        })
    }

    async fn on_server_started(&self, runtime: &dyn McpServer) {}
}
