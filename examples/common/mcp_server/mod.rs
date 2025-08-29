mod handler;
mod tools;

use std::path::PathBuf;
use std::time::Duration;

use iced::futures::SinkExt;
use iced::futures::channel::mpsc;
use rust_mcp_sdk::mcp_server::{HyperServerOptions, hyper_server};
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
    ServerCapabilitiesTools,
};

use super::base::BaseMessage;

pub(super) fn server_task<Message: Send + 'static>() -> iced::Task<BaseMessage<Message>> {
    iced::Task::stream(iced::stream::channel(
        10,
        move |sender: mpsc::Sender<AppMessage>| async move {
            let task = async move {
                let server_details = InitializeResult {
                    // server name and version
                    server_info: Implementation {
                        name: "Hello World MCP Server SSE".to_string(),
                        version: "0.1.0".to_string(),
                        title: Some("Hello World MCP Server SSE".to_string()),
                    },
                    capabilities: ServerCapabilities {
                        // indicates that server support mcp tools
                        tools: Some(ServerCapabilitiesTools { list_changed: None }),
                        ..Default::default() // Using default values for other fields
                    },
                    meta: None,
                    instructions: Some("server instructions...".to_string()),
                    protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
                };

                let handler = handler::MyServerHandler {
                    sender: AppSender::new(sender),
                };

                let server = hyper_server::create_server(
                    server_details,
                    handler,
                    HyperServerOptions {
                        host: "127.0.0.1".to_string(),
                        port: 8342,
                        ping_interval: Duration::from_secs(5),
                        ..Default::default()
                    },
                );

                server.start().await.unwrap();

                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            };

            if let Err(e) = task.await {
                eprintln!("Error occurred while running server task: {}", e);
            }
        },
    ))
    .map(BaseMessage::McpServerMessage)
}

#[derive(Debug)]
pub(super) enum AppMessage {
    Screenshot(
        MessageWrap<PathBuf, Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>>,
    ),
    Exit,
}

#[derive(Debug, Clone)]
struct AppSender {
    sender: mpsc::Sender<AppMessage>,
}

impl AppSender {
    pub fn new(sender: mpsc::Sender<AppMessage>) -> Self {
        Self { sender }
    }

    pub async fn send_screenshot(
        &self,
        path: PathBuf,
    ) -> tokio::sync::oneshot::Receiver<
        Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
    > {
        let (responder, receiver) = tokio::sync::oneshot::channel();
        let wrapped = MessageWrap {
            message: path,
            responder,
        };
        self.sender
            .to_owned()
            .send(AppMessage::Screenshot(wrapped))
            .await
            .unwrap();
        receiver
    }

    pub async fn send_exit(&self) {
        self.sender.to_owned().send(AppMessage::Exit).await.unwrap();
    }
}

#[derive(Debug)]
pub(super) struct MessageWrap<Message, Response> {
    pub(super) message: Message,
    pub(super) responder: tokio::sync::oneshot::Sender<Response>,
}
