mod discord;
mod mcp_handler;
mod tools;

use clap::Parser;
use discord::HumanInDiscord;
use rust_mcp_sdk::error::{McpSdkError, SdkResult};
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
};

use rust_mcp_sdk::{
    mcp_server::{server_runtime, ServerRuntime},
    McpServer, StdioTransport, TransportOptions,
};
use serenity::all::{ChannelId, UserId};

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, env = "DISCORD_TOKEN")]
    discord_token: String,
    #[clap(long, env = "DISCORD_CHANNEL_ID")]
    discord_channel_id: ChannelId,
    #[clap(long, env = "DISCORD_USER_ID")]
    discord_user_id: UserId,
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    let Args {
        discord_token,
        discord_channel_id,
        discord_user_id,
    } = Args::parse();

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Human in the loop".to_string(),
            version: "0.1.0".to_string(),
        },
        capabilities: ServerCapabilities {
            // indicates that server support mcp tools
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default() // Using default values for other fields
        },
        meta: None,
        instructions: Some(
            "This is a Human-in-the-Loop MCP server that enables AI assistants to request \
             information from humans via Discord. Use the 'ask_human' tool when you need \
             information that only a human would know, such as: personal preferences, \
             project-specific context, local environment details, or any information that \
             is not publicly available or documented. The human will be notified in Discord \
             and their response will be returned to you."
                .to_string(),
        ),
        protocol_version: "2025-06-18".to_string(),
    };

    let transport = StdioTransport::new(TransportOptions::default())?;

    let human = HumanInDiscord::new(discord_user_id, discord_channel_id);
    let discord = discord::start(&discord_token, human.handler().clone());

    let server: ServerRuntime =
        server_runtime::create_server(server_details, transport, mcp_handler::Handler::new(human));
    let mcp = server.start();

    tokio::select! {
        res = mcp => res?,
        res = discord => res.map_err(|e| McpSdkError::AnyError(e.into_boxed_dyn_error()))?,
    }
    Ok(())
}
