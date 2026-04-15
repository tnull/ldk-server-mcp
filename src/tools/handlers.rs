// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use ldk_server_client::client::LdkServerClient;
use ldk_server_client::ldk_server_grpc::api::{
	Bolt11ClaimForHashRequest, Bolt11FailForHashRequest, Bolt11ReceiveForHashRequest,
	Bolt11ReceiveRequest, Bolt11ReceiveVariableAmountViaJitChannelRequest,
	Bolt11ReceiveViaJitChannelRequest, Bolt12ReceiveRequest, Bolt12SendRequest,
	CloseChannelRequest, ConnectPeerRequest, DecodeInvoiceRequest, DecodeOfferRequest,
	DisconnectPeerRequest, ExportPathfindingScoresRequest, ForceCloseChannelRequest,
	GetBalancesRequest, GetNodeInfoRequest, GetPaymentDetailsRequest, GraphGetChannelRequest,
	GraphGetNodeRequest, GraphListChannelsRequest, GraphListNodesRequest, ListChannelsRequest,
	ListForwardedPaymentsRequest, ListPaymentsRequest, ListPeersRequest, OnchainReceiveRequest,
	OnchainSendRequest, OpenChannelRequest, SignMessageRequest, SpliceInRequest, SpliceOutRequest,
	SpontaneousSendRequest, UnifiedSendRequest, UpdateChannelConfigRequest, VerifySignatureRequest,
};
use ldk_server_client::ldk_server_grpc::types::{
	bolt11_invoice_description, Bolt11InvoiceDescription, ChannelConfig, PageToken,
	RouteParametersConfig,
};
use serde_json::{json, Value};

const DEFAULT_MAX_TOTAL_CLTV_EXPIRY_DELTA: u32 = 1008;
const DEFAULT_MAX_PATH_COUNT: u32 = 10;
const DEFAULT_MAX_CHANNEL_SATURATION_POWER_OF_HALF: u32 = 2;
const DEFAULT_EXPIRY_SECS: u32 = 86_400;

fn hex_encode(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn parse_page_token(token_str: &str) -> Result<PageToken, String> {
	let parts: Vec<&str> = token_str.split(':').collect();
	if parts.len() != 2 {
		return Err("Page token must be in format 'token:index'".to_string());
	}
	let index = parts[1].parse::<i64>().map_err(|_| "Invalid page token index".to_string())?;
	Ok(PageToken { token: parts[0].to_string(), index })
}

fn format_page_token(pt: &PageToken) -> String {
	format!("{}:{}", pt.token, pt.index)
}

fn build_route_parameters(args: &Value) -> RouteParametersConfig {
	RouteParametersConfig {
		max_total_routing_fee_msat: args.get("max_total_routing_fee_msat").and_then(|v| v.as_u64()),
		max_total_cltv_expiry_delta: args
			.get("max_total_cltv_expiry_delta")
			.and_then(|v| v.as_u64())
			.map(|v| v as u32)
			.unwrap_or(DEFAULT_MAX_TOTAL_CLTV_EXPIRY_DELTA),
		max_path_count: args
			.get("max_path_count")
			.and_then(|v| v.as_u64())
			.map(|v| v as u32)
			.unwrap_or(DEFAULT_MAX_PATH_COUNT),
		max_channel_saturation_power_of_half: args
			.get("max_channel_saturation_power_of_half")
			.and_then(|v| v.as_u64())
			.map(|v| v as u32)
			.unwrap_or(DEFAULT_MAX_CHANNEL_SATURATION_POWER_OF_HALF),
	}
}

fn build_channel_config(args: &Value) -> Option<ChannelConfig> {
	let forwarding_fee_proportional_millionths = args
		.get("forwarding_fee_proportional_millionths")
		.and_then(|v| v.as_u64())
		.map(|v| v as u32);
	let forwarding_fee_base_msat =
		args.get("forwarding_fee_base_msat").and_then(|v| v.as_u64()).map(|v| v as u32);
	let cltv_expiry_delta =
		args.get("cltv_expiry_delta").and_then(|v| v.as_u64()).map(|v| v as u32);

	if forwarding_fee_proportional_millionths.is_none()
		&& forwarding_fee_base_msat.is_none()
		&& cltv_expiry_delta.is_none()
	{
		return None;
	}

	Some(ChannelConfig {
		forwarding_fee_proportional_millionths,
		forwarding_fee_base_msat,
		cltv_expiry_delta,
		force_close_avoidance_max_fee_satoshis: None,
		accept_underpaying_htlcs: None,
		max_dust_htlc_exposure: None,
	})
}

fn build_bolt11_invoice_description(
	args: &Value,
) -> Result<Option<Bolt11InvoiceDescription>, String> {
	let description_str = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
	let description_hash =
		args.get("description_hash").and_then(|v| v.as_str()).map(|s| s.to_string());

	match (description_str, description_hash) {
		(Some(desc), None) => Ok(Some(Bolt11InvoiceDescription {
			kind: Some(bolt11_invoice_description::Kind::Direct(desc)),
		})),
		(None, Some(hash)) => Ok(Some(Bolt11InvoiceDescription {
			kind: Some(bolt11_invoice_description::Kind::Hash(hash)),
		})),
		(Some(_), Some(_)) => {
			Err("Only one of description or description_hash can be set".to_string())
		},
		(None, None) => Ok(None),
	}
}

pub async fn handle_get_node_info(client: &LdkServerClient, _args: Value) -> Result<Value, String> {
	let response =
		client.get_node_info(GetNodeInfoRequest {}).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_get_balances(client: &LdkServerClient, _args: Value) -> Result<Value, String> {
	let response =
		client.get_balances(GetBalancesRequest {}).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_onchain_receive(
	client: &LdkServerClient, _args: Value,
) -> Result<Value, String> {
	let response =
		client.onchain_receive(OnchainReceiveRequest {}).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_onchain_send(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let address = args
		.get("address")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: address")?
		.to_string();
	let amount_sats = args.get("amount_sats").and_then(|v| v.as_u64());
	let send_all = args.get("send_all").and_then(|v| v.as_bool());
	let fee_rate_sat_per_vb = args.get("fee_rate_sat_per_vb").and_then(|v| v.as_u64());

	let response = client
		.onchain_send(OnchainSendRequest { address, amount_sats, send_all, fee_rate_sat_per_vb })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_receive(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let amount_msat = args.get("amount_msat").and_then(|v| v.as_u64());
	let invoice_description = build_bolt11_invoice_description(&args)?;

	let expiry_secs = args
		.get("expiry_secs")
		.and_then(|v| v.as_u64())
		.map(|v| v as u32)
		.unwrap_or(DEFAULT_EXPIRY_SECS);

	let response = client
		.bolt11_receive(Bolt11ReceiveRequest {
			description: invoice_description,
			expiry_secs,
			amount_msat,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_receive_for_hash(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let amount_msat = args.get("amount_msat").and_then(|v| v.as_u64());
	let description = build_bolt11_invoice_description(&args)?;
	let expiry_secs = args
		.get("expiry_secs")
		.and_then(|v| v.as_u64())
		.map(|v| v as u32)
		.unwrap_or(DEFAULT_EXPIRY_SECS);
	let payment_hash = args
		.get("payment_hash")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: payment_hash")?
		.to_string();

	let response = client
		.bolt11_receive_for_hash(Bolt11ReceiveForHashRequest {
			amount_msat,
			description,
			expiry_secs,
			payment_hash,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_claim_for_hash(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let payment_hash = args.get("payment_hash").and_then(|v| v.as_str()).map(|s| s.to_string());
	let claimable_amount_msat = args.get("claimable_amount_msat").and_then(|v| v.as_u64());
	let preimage = args
		.get("preimage")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: preimage")?
		.to_string();

	let response = client
		.bolt11_claim_for_hash(Bolt11ClaimForHashRequest {
			payment_hash,
			claimable_amount_msat,
			preimage,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_fail_for_hash(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let payment_hash = args
		.get("payment_hash")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: payment_hash")?
		.to_string();

	let response = client
		.bolt11_fail_for_hash(Bolt11FailForHashRequest { payment_hash })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_receive_via_jit_channel(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let amount_msat = args
		.get("amount_msat")
		.and_then(|v| v.as_u64())
		.ok_or("Missing required parameter: amount_msat")?;
	let description = build_bolt11_invoice_description(&args)?;
	let expiry_secs = args
		.get("expiry_secs")
		.and_then(|v| v.as_u64())
		.map(|v| v as u32)
		.unwrap_or(DEFAULT_EXPIRY_SECS);
	let max_total_lsp_fee_limit_msat =
		args.get("max_total_lsp_fee_limit_msat").and_then(|v| v.as_u64());

	let response = client
		.bolt11_receive_via_jit_channel(Bolt11ReceiveViaJitChannelRequest {
			amount_msat,
			description,
			expiry_secs,
			max_total_lsp_fee_limit_msat,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_receive_variable_amount_via_jit_channel(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let description = build_bolt11_invoice_description(&args)?;
	let expiry_secs = args
		.get("expiry_secs")
		.and_then(|v| v.as_u64())
		.map(|v| v as u32)
		.unwrap_or(DEFAULT_EXPIRY_SECS);
	let max_proportional_lsp_fee_limit_ppm_msat =
		args.get("max_proportional_lsp_fee_limit_ppm_msat").and_then(|v| v.as_u64());

	let response = client
		.bolt11_receive_variable_amount_via_jit_channel(
			Bolt11ReceiveVariableAmountViaJitChannelRequest {
				description,
				expiry_secs,
				max_proportional_lsp_fee_limit_ppm_msat,
			},
		)
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt11_send(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let invoice = args
		.get("invoice")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: invoice")?
		.to_string();
	let amount_msat = args.get("amount_msat").and_then(|v| v.as_u64());
	let route_parameters = build_route_parameters(&args);

	let response = client
		.bolt11_send(ldk_server_client::ldk_server_grpc::api::Bolt11SendRequest {
			invoice,
			amount_msat,
			route_parameters: Some(route_parameters),
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt12_receive(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let description = args
		.get("description")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: description")?
		.to_string();
	let amount_msat = args.get("amount_msat").and_then(|v| v.as_u64());
	let expiry_secs = args.get("expiry_secs").and_then(|v| v.as_u64()).map(|v| v as u32);
	let quantity = args.get("quantity").and_then(|v| v.as_u64());

	let response = client
		.bolt12_receive(Bolt12ReceiveRequest { description, amount_msat, expiry_secs, quantity })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_bolt12_send(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let offer = args
		.get("offer")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: offer")?
		.to_string();
	let amount_msat = args.get("amount_msat").and_then(|v| v.as_u64());
	let quantity = args.get("quantity").and_then(|v| v.as_u64());
	let payer_note = args.get("payer_note").and_then(|v| v.as_str()).map(|s| s.to_string());
	let route_parameters = build_route_parameters(&args);

	let response = client
		.bolt12_send(Bolt12SendRequest {
			offer,
			amount_msat,
			quantity,
			payer_note,
			route_parameters: Some(route_parameters),
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_spontaneous_send(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let amount_msat = args
		.get("amount_msat")
		.and_then(|v| v.as_u64())
		.ok_or("Missing required parameter: amount_msat")?;
	let node_id = args
		.get("node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: node_id")?
		.to_string();
	let route_parameters = build_route_parameters(&args);

	let response = client
		.spontaneous_send(SpontaneousSendRequest {
			amount_msat,
			node_id,
			route_parameters: Some(route_parameters),
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_unified_send(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let uri = args
		.get("uri")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: uri")?
		.to_string();
	let amount_msat = args.get("amount_msat").and_then(|v| v.as_u64());
	let route_parameters = build_route_parameters(&args);

	let response = client
		.unified_send(UnifiedSendRequest {
			uri,
			amount_msat,
			route_parameters: Some(route_parameters),
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_open_channel(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let node_pubkey = args
		.get("node_pubkey")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: node_pubkey")?
		.to_string();
	let address = args
		.get("address")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: address")?
		.to_string();
	let channel_amount_sats = args
		.get("channel_amount_sats")
		.and_then(|v| v.as_u64())
		.ok_or("Missing required parameter: channel_amount_sats")?;
	let push_to_counterparty_msat = args.get("push_to_counterparty_msat").and_then(|v| v.as_u64());
	let announce_channel = args.get("announce_channel").and_then(|v| v.as_bool()).unwrap_or(false);
	let channel_config = build_channel_config(&args);

	let response = client
		.open_channel(OpenChannelRequest {
			node_pubkey,
			address,
			channel_amount_sats,
			push_to_counterparty_msat,
			channel_config,
			announce_channel,
			disable_counterparty_reserve: false,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_splice_in(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let user_channel_id = args
		.get("user_channel_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: user_channel_id")?
		.to_string();
	let counterparty_node_id = args
		.get("counterparty_node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: counterparty_node_id")?
		.to_string();
	let splice_amount_sats = args
		.get("splice_amount_sats")
		.and_then(|v| v.as_u64())
		.ok_or("Missing required parameter: splice_amount_sats")?;

	let response = client
		.splice_in(SpliceInRequest { user_channel_id, counterparty_node_id, splice_amount_sats })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_splice_out(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let user_channel_id = args
		.get("user_channel_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: user_channel_id")?
		.to_string();
	let counterparty_node_id = args
		.get("counterparty_node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: counterparty_node_id")?
		.to_string();
	let splice_amount_sats = args
		.get("splice_amount_sats")
		.and_then(|v| v.as_u64())
		.ok_or("Missing required parameter: splice_amount_sats")?;
	let address = args.get("address").and_then(|v| v.as_str()).map(|s| s.to_string());

	let response = client
		.splice_out(SpliceOutRequest {
			user_channel_id,
			counterparty_node_id,
			address,
			splice_amount_sats,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_close_channel(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let user_channel_id = args
		.get("user_channel_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: user_channel_id")?
		.to_string();
	let counterparty_node_id = args
		.get("counterparty_node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: counterparty_node_id")?
		.to_string();

	let response = client
		.close_channel(CloseChannelRequest { user_channel_id, counterparty_node_id })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_force_close_channel(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let user_channel_id = args
		.get("user_channel_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: user_channel_id")?
		.to_string();
	let counterparty_node_id = args
		.get("counterparty_node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: counterparty_node_id")?
		.to_string();
	let force_close_reason =
		args.get("force_close_reason").and_then(|v| v.as_str()).map(|s| s.to_string());

	let response = client
		.force_close_channel(ForceCloseChannelRequest {
			user_channel_id,
			counterparty_node_id,
			force_close_reason,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_list_channels(client: &LdkServerClient, _args: Value) -> Result<Value, String> {
	let response =
		client.list_channels(ListChannelsRequest {}).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_update_channel_config(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let user_channel_id = args
		.get("user_channel_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: user_channel_id")?
		.to_string();
	let counterparty_node_id = args
		.get("counterparty_node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: counterparty_node_id")?
		.to_string();

	let forwarding_fee_proportional_millionths = args
		.get("forwarding_fee_proportional_millionths")
		.and_then(|v| v.as_u64())
		.map(|v| v as u32);
	let forwarding_fee_base_msat =
		args.get("forwarding_fee_base_msat").and_then(|v| v.as_u64()).map(|v| v as u32);
	let cltv_expiry_delta =
		args.get("cltv_expiry_delta").and_then(|v| v.as_u64()).map(|v| v as u32);

	let channel_config = ChannelConfig {
		forwarding_fee_proportional_millionths,
		forwarding_fee_base_msat,
		cltv_expiry_delta,
		force_close_avoidance_max_fee_satoshis: None,
		accept_underpaying_htlcs: None,
		max_dust_htlc_exposure: None,
	};

	let response = client
		.update_channel_config(UpdateChannelConfigRequest {
			user_channel_id,
			counterparty_node_id,
			channel_config: Some(channel_config),
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_list_payments(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let page_token = match args.get("page_token").and_then(|v| v.as_str()) {
		Some(token_str) => Some(parse_page_token(token_str)?),
		None => None,
	};

	let response = client
		.list_payments(ListPaymentsRequest { page_token })
		.await
		.map_err(|e| e.message.clone())?;

	let mut result = serde_json::to_value(&response)
		.map_err(|e| format!("Failed to serialize response: {e}"))?;

	if let Some(ref npt) = response.next_page_token {
		result
			.as_object_mut()
			.unwrap()
			.insert("next_page_token".to_string(), json!(format_page_token(npt)));
	}

	Ok(result)
}

pub async fn handle_get_payment_details(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let payment_id = args
		.get("payment_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: payment_id")?
		.to_string();

	let response = client
		.get_payment_details(GetPaymentDetailsRequest { payment_id })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_list_forwarded_payments(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let page_token = match args.get("page_token").and_then(|v| v.as_str()) {
		Some(token_str) => Some(parse_page_token(token_str)?),
		None => None,
	};

	let response = client
		.list_forwarded_payments(ListForwardedPaymentsRequest { page_token })
		.await
		.map_err(|e| e.message.clone())?;

	let mut result = serde_json::to_value(&response)
		.map_err(|e| format!("Failed to serialize response: {e}"))?;

	if let Some(ref npt) = response.next_page_token {
		result
			.as_object_mut()
			.unwrap()
			.insert("next_page_token".to_string(), json!(format_page_token(npt)));
	}

	Ok(result)
}

pub async fn handle_connect_peer(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let node_pubkey = args
		.get("node_pubkey")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: node_pubkey")?
		.to_string();
	let address = args
		.get("address")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: address")?
		.to_string();
	let persist = args.get("persist").and_then(|v| v.as_bool()).unwrap_or(false);

	let response = client
		.connect_peer(ConnectPeerRequest { node_pubkey, address, persist })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_disconnect_peer(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let node_pubkey = args
		.get("node_pubkey")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: node_pubkey")?
		.to_string();

	let response = client
		.disconnect_peer(DisconnectPeerRequest { node_pubkey })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_list_peers(client: &LdkServerClient, _args: Value) -> Result<Value, String> {
	let response = client.list_peers(ListPeersRequest {}).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_decode_invoice(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let invoice = args
		.get("invoice")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: invoice")?
		.to_string();

	let response = client
		.decode_invoice(DecodeInvoiceRequest { invoice })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_decode_offer(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let offer = args
		.get("offer")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: offer")?
		.to_string();

	let response =
		client.decode_offer(DecodeOfferRequest { offer }).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_sign_message(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let message = args
		.get("message")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: message")?
		.to_string();

	let response = client
		.sign_message(SignMessageRequest { message: message.into_bytes().into() })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_verify_signature(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let message = args
		.get("message")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: message")?
		.to_string();
	let signature = args
		.get("signature")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: signature")?
		.to_string();
	let public_key = args
		.get("public_key")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: public_key")?
		.to_string();

	let response = client
		.verify_signature(VerifySignatureRequest {
			message: message.into_bytes().into(),
			signature,
			public_key,
		})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_export_pathfinding_scores(
	client: &LdkServerClient, _args: Value,
) -> Result<Value, String> {
	let response = client
		.export_pathfinding_scores(ExportPathfindingScoresRequest {})
		.await
		.map_err(|e| e.message.clone())?;
	let scores_hex = hex_encode(&response.scores);
	Ok(json!({ "pathfinding_scores": scores_hex }))
}

pub async fn handle_graph_list_channels(
	client: &LdkServerClient, _args: Value,
) -> Result<Value, String> {
	let response = client
		.graph_list_channels(GraphListChannelsRequest {})
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_graph_get_channel(
	client: &LdkServerClient, args: Value,
) -> Result<Value, String> {
	let short_channel_id = args
		.get("short_channel_id")
		.and_then(|v| v.as_u64())
		.ok_or("Missing required parameter: short_channel_id")?;

	let response = client
		.graph_get_channel(GraphGetChannelRequest { short_channel_id })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_graph_list_nodes(
	client: &LdkServerClient, _args: Value,
) -> Result<Value, String> {
	let response =
		client.graph_list_nodes(GraphListNodesRequest {}).await.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}

pub async fn handle_graph_get_node(client: &LdkServerClient, args: Value) -> Result<Value, String> {
	let node_id = args
		.get("node_id")
		.and_then(|v| v.as_str())
		.ok_or("Missing required parameter: node_id")?
		.to_string();

	let response = client
		.graph_get_node(GraphGetNodeRequest { node_id })
		.await
		.map_err(|e| e.message.clone())?;
	serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {e}"))
}
