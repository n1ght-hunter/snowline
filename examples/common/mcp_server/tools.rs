use std::path::PathBuf;
use std::sync::OnceLock;

use async_trait::async_trait;
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use rust_mcp_sdk::schema::{Tool, ToolAnnotations, ToolInputSchema, ToolOutputSchema};
use serde::{Deserialize, Serialize};

use super::handler::MyServerHandler;

#[async_trait]
pub trait McpTool<State = MyServerHandler>: Default + Send + Sync {
    type Input: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize;
    type Output: schemars::JsonSchema + for<'de> Deserialize<'de> + Serialize;

    /// Intended for programmatic or logical use, but used as a display name in past specs or fallback (if title isn't present).
    fn name(&self) -> &str;

    /// Returns a human-readable description of the tool.
    /// This can be used by clients to improve the LLM's understanding of available tools.
    fn description(&self) -> Option<&str>;

    /// Returns the input schema for the tool.
    fn input_schema(&self) -> &ToolInputSchema {
        static SCHEMA: OnceLock<ToolInputSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            let json_schema = schemars::schema_for!(Self::Input);
            let required = json_schema.get("required").cloned().unwrap_or_default();
            let properties = json_schema.get("properties").cloned().unwrap_or_default();

            ToolInputSchema::new(
                serde_json::from_value(required).unwrap_or_default(),
                serde_json::from_value(properties).unwrap_or_default(),
            )
        })
    }

    /// Returns the output schema for the tool.
    fn output_schema(&self) -> &ToolOutputSchema {
        static SCHEMA: OnceLock<ToolOutputSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            let json_schema = schemars::schema_for!(Self::Output);
            let required = json_schema.get("required").cloned().unwrap_or_default();
            let properties = json_schema.get("properties").cloned().unwrap_or_default();

            ToolOutputSchema::new(
                serde_json::from_value(required).unwrap_or_default(),
                serde_json::from_value(properties).unwrap_or_default(),
            )
        })
    }

    /// Intended for UI and end-user contexts — optimized to be human-readable and easily understood,
    /// even by those unfamiliar with domain-specific terminology.
    /// If not provided, the name should be used for display (except for Tool,
    /// where annotations.title should be given precedence over using name,
    /// if present).
    fn title(&self) -> Option<&str> {
        None
    }

    ///See [specification/2025-06-18/basic/index#general-fields] for notes on _meta usage.
    fn meta(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        None
    }

    /// Optional additional tool information.
    /// Display name precedence order is: title, annotations.title, then name.
    fn annotations(&self) -> Option<&ToolAnnotations> {
        None
    }

    fn as_tool_record(&self) -> Tool {
        Tool {
            name: self.name().to_string(),
            description: self.description().map(|s| s.to_string()),
            input_schema: self.input_schema().clone(),
            annotations: self.annotations().cloned(),
            meta: self.meta().cloned(),
            output_schema: Some(self.output_schema().clone()),
            title: self.title().map(|s| s.to_string()),
        }
    }

    async fn call(
        &self,
        args: Self::Input,
        state: &State,
    ) -> std::result::Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>>;
}

/// Input/Output types for tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotInput {
    pub output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotOutput {
    pub path: String,
    pub success: bool,
}

#[derive(Debug, Default)]
pub struct ScreenShot;

#[async_trait]
impl McpTool for ScreenShot {
    type Input = ScreenshotInput;
    type Output = ScreenshotOutput;

    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> Option<&str> {
        Some("Takes a screenshot and saves it to the specified path.")
    }

    async fn call(
        &self,
        args: Self::Input,
        state: &MyServerHandler,
    ) -> std::result::Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        state
            .sender
            .send_screenshot(args.output.clone())
            .await
            .await??;

        let output = ScreenshotOutput {
            path: args.output.to_string_lossy().to_string(),
            success: true,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string(&output)?,
        )]))
    }
}

#[derive(Debug)]
pub struct Exit {
    input_schema: ToolInputSchema,
    output_schema: ToolOutputSchema,
}

impl Default for Exit {
    fn default() -> Self {
        Self {
            input_schema: ToolInputSchema::new(vec![], None),
            output_schema: ToolOutputSchema::new(vec![], None),
        }
    }
}

#[async_trait]
impl McpTool for Exit {
    type Input = serde_json::Value;
    type Output = serde_json::Value;

    fn name(&self) -> &str {
        "exit"
    }

    fn description(&self) -> Option<&str> {
        Some("Exits the application.")
    }

    fn input_schema(&self) -> &ToolInputSchema {
        &self.input_schema
    }

    fn output_schema(&self) -> &ToolOutputSchema {
        &self.output_schema
    }

    async fn call(
        &self,
        _args: Self::Input,
        state: &MyServerHandler,
    ) -> std::result::Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
        state.sender.send_exit().await;
        Ok(CallToolResult::text_content(vec![]))
    }
}

/// Simple macro to define tools enum
macro_rules! define_tools {
    ($($tool:ident),* $(,)?) => {

        static TOOLS: std::sync::LazyLock<Vec<Tool>> = std::sync::LazyLock::new(|| {
            vec![
                $(
                   $tool::default().as_tool_record(),
                )*
            ]
        });


        #[derive(Debug)]
        pub enum IcedTool {
            $(
                $tool($tool),
            )*
        }

        impl IcedTool {
            pub fn name(&self) -> &str {
                match self {
                    $(
                        IcedTool::$tool(tool) => tool.name(),
                    )*
                }
            }


            /// Returns all available tools as schema records
            pub fn all_tools() -> Vec<Tool> {
               TOOLS.clone()
            }

            pub async fn call_tool(
                &self,
                args: serde_json::Value,
                state: &MyServerHandler,
            ) -> Result<CallToolResult, Box<dyn std::error::Error + Send + Sync>> {
                match self {
                    $(
                        IcedTool::$tool(tool) => {
                            tool.call(serde_json::from_value(args)?, state).await
                        }
                    )*
                }
            }

            pub fn from_str(name: &str) -> Result<Self, CallToolError> {
                match name {
                    "screenshot" => Ok(IcedTool::ScreenShot(ScreenShot::default())),
                    "exit" => Ok(IcedTool::Exit(Exit::default())),
                    _ => Err(CallToolError::unknown_tool(name.to_string())),
                }
            }
        }
    };
}

// Use the macro to define our tools
define_tools!(ScreenShot, Exit);
