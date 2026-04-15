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

struct ToolSpec {
	name: &'static str,
	description: &'static str,
	input_schema: fn() -> Value,
	handler: ToolHandler,
}

macro_rules! tool_specs {
	($(($name:literal, $description:literal, $schema:path, $handler:path)),+ $(,)?) => {
		[
			$(
				ToolSpec {
					name: $name,
					description: $description,
					input_schema: $schema,
					handler: |client, args| Box::pin($handler(client, args)),
				},
			)+
		]
	};
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
	let tools = tool_specs![
		(
			"get_node_info",
			"Retrieve node info including node_id, sync status, and best block",
			schema::get_node_info_schema,
			handlers::handle_get_node_info
		),
		(
			"get_balances",
			"Retrieve an overview of all known balances (on-chain and Lightning)",
			schema::get_balances_schema,
			handlers::handle_get_balances
		),
		(
			"onchain_receive",
			"Generate a new on-chain Bitcoin funding address",
			schema::onchain_receive_schema,
			handlers::handle_onchain_receive
		),
		(
			"onchain_send",
			"Send an on-chain Bitcoin payment to an address",
			schema::onchain_send_schema,
			handlers::handle_onchain_send
		),
		(
			"bolt11_receive",
			"Create a BOLT11 Lightning invoice to receive a payment",
			schema::bolt11_receive_schema,
			handlers::handle_bolt11_receive
		),
		(
			"bolt11_receive_for_hash",
			"Create a BOLT11 Lightning invoice for a specific payment hash",
			schema::bolt11_receive_for_hash_schema,
			handlers::handle_bolt11_receive_for_hash
		),
		(
			"bolt11_claim_for_hash",
			"Manually claim a BOLT11 payment for a specific payment hash",
			schema::bolt11_claim_for_hash_schema,
			handlers::handle_bolt11_claim_for_hash
		),
		(
			"bolt11_fail_for_hash",
			"Manually fail a BOLT11 payment for a specific payment hash",
			schema::bolt11_fail_for_hash_schema,
			handlers::handle_bolt11_fail_for_hash
		),
		(
			"bolt11_receive_via_jit_channel",
			"Create a BOLT11 Lightning invoice to receive via an LSPS2 JIT channel",
			schema::bolt11_receive_via_jit_channel_schema,
			handlers::handle_bolt11_receive_via_jit_channel
		),
		(
			"bolt11_receive_variable_amount_via_jit_channel",
			"Create a variable-amount BOLT11 Lightning invoice to receive via an LSPS2 JIT channel",
			schema::bolt11_receive_variable_amount_via_jit_channel_schema,
			handlers::handle_bolt11_receive_variable_amount_via_jit_channel
		),
		(
			"bolt11_send",
			"Pay a BOLT11 Lightning invoice",
			schema::bolt11_send_schema,
			handlers::handle_bolt11_send
		),
		(
			"bolt12_receive",
			"Create a BOLT12 offer for receiving Lightning payments",
			schema::bolt12_receive_schema,
			handlers::handle_bolt12_receive
		),
		(
			"bolt12_send",
			"Pay a BOLT12 Lightning offer",
			schema::bolt12_send_schema,
			handlers::handle_bolt12_send
		),
		(
			"spontaneous_send",
			"Send a spontaneous (keysend) payment to a Lightning node",
			schema::spontaneous_send_schema,
			handlers::handle_spontaneous_send
		),
		(
			"unified_send",
			"Send a payment given a BIP 21 URI or BIP 353 Human-Readable Name",
			schema::unified_send_schema,
			handlers::handle_unified_send
		),
		(
			"open_channel",
			"Open a new Lightning channel with a remote node",
			schema::open_channel_schema,
			handlers::handle_open_channel
		),
		(
			"splice_in",
			"Increase a channel's balance by splicing in on-chain funds",
			schema::splice_in_schema,
			handlers::handle_splice_in
		),
		(
			"splice_out",
			"Decrease a channel's balance by splicing out to on-chain",
			schema::splice_out_schema,
			handlers::handle_splice_out
		),
		(
			"close_channel",
			"Cooperatively close a Lightning channel",
			schema::close_channel_schema,
			handlers::handle_close_channel
		),
		(
			"force_close_channel",
			"Force close a Lightning channel unilaterally",
			schema::force_close_channel_schema,
			handlers::handle_force_close_channel
		),
		(
			"list_channels",
			"List all known Lightning channels",
			schema::list_channels_schema,
			handlers::handle_list_channels
		),
		(
			"update_channel_config",
			"Update forwarding fees and CLTV delta for a channel",
			schema::update_channel_config_schema,
			handlers::handle_update_channel_config
		),
		(
			"list_payments",
			"List all payments (supports pagination via page_token)",
			schema::list_payments_schema,
			handlers::handle_list_payments
		),
		(
			"get_payment_details",
			"Get details of a specific payment by its ID",
			schema::get_payment_details_schema,
			handlers::handle_get_payment_details
		),
		(
			"list_forwarded_payments",
			"List all forwarded payments (supports pagination via page_token)",
			schema::list_forwarded_payments_schema,
			handlers::handle_list_forwarded_payments
		),
		(
			"connect_peer",
			"Connect to a Lightning peer without opening a channel",
			schema::connect_peer_schema,
			handlers::handle_connect_peer
		),
		(
			"disconnect_peer",
			"Disconnect from a Lightning peer",
			schema::disconnect_peer_schema,
			handlers::handle_disconnect_peer
		),
		(
			"list_peers",
			"List all known Lightning peers",
			schema::list_peers_schema,
			handlers::handle_list_peers
		),
		(
			"decode_invoice",
			"Decode a BOLT11 invoice and return its parsed fields",
			schema::decode_invoice_schema,
			handlers::handle_decode_invoice
		),
		(
			"decode_offer",
			"Decode a BOLT12 offer and return its parsed fields",
			schema::decode_offer_schema,
			handlers::handle_decode_offer
		),
		(
			"sign_message",
			"Sign a message with the node's secret key",
			schema::sign_message_schema,
			handlers::handle_sign_message
		),
		(
			"verify_signature",
			"Verify a signature against a message and public key",
			schema::verify_signature_schema,
			handlers::handle_verify_signature
		),
		(
			"export_pathfinding_scores",
			"Export the pathfinding scores used by the Lightning router",
			schema::export_pathfinding_scores_schema,
			handlers::handle_export_pathfinding_scores
		),
		(
			"graph_list_channels",
			"List all known short channel IDs in the network graph",
			schema::graph_list_channels_schema,
			handlers::handle_graph_list_channels
		),
		(
			"graph_get_channel",
			"Get channel information from the network graph by short channel ID",
			schema::graph_get_channel_schema,
			handlers::handle_graph_get_channel
		),
		(
			"graph_list_nodes",
			"List all known node IDs in the network graph",
			schema::graph_list_nodes_schema,
			handlers::handle_graph_list_nodes
		),
		(
			"graph_get_node",
			"Get node information from the network graph by node ID",
			schema::graph_get_node_schema,
			handlers::handle_graph_get_node
		),
	]
	.into_iter()
	.map(|spec| {
		(
			ToolDefinition {
				name: spec.name.to_string(),
				description: spec.description.to_string(),
				input_schema: (spec.input_schema)(),
			},
			spec.handler,
		)
	})
	.collect();

	ToolRegistry { tools }
}
