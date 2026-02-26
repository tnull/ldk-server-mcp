// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

pub mod handlers;
pub mod schema;

use std::future::Future;
use std::pin::Pin;

use ldk_server_client::client::LdkServerClient;
use serde_json::Value;

use crate::mcp::{ToolCallResult, ToolDefinition};

type ToolHandler = for<'a> fn(
	&'a LdkServerClient,
	Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send + 'a>>;

pub struct ToolRegistry {
	tools: Vec<(ToolDefinition, ToolHandler)>,
}

impl ToolRegistry {
	pub fn list_tools(&self) -> Vec<&ToolDefinition> {
		self.tools.iter().map(|(def, _)| def).collect()
	}

	pub async fn call_tool(
		&self, client: &LdkServerClient, name: &str, args: Value,
	) -> ToolCallResult {
		for (def, handler) in &self.tools {
			if def.name == name {
				return match handler(client, args).await {
					Ok(value) => {
						let text = serde_json::to_string_pretty(&value)
							.unwrap_or_else(|e| format!("Failed to serialize response: {e}"));
						ToolCallResult::success(text)
					},
					Err(e) => ToolCallResult::error(e),
				};
			}
		}
		ToolCallResult::error(format!("Unknown tool: {name}"))
	}
}

pub fn build_tool_registry() -> ToolRegistry {
	let tools = Vec::new();
	ToolRegistry { tools }
}
