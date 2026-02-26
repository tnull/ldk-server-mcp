// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use serde_json::{json, Value};

pub fn get_node_info_schema() -> Value {
	json!({
		"type": "object",
		"properties": {},
		"required": []
	})
}

pub fn get_balances_schema() -> Value {
	json!({
		"type": "object",
		"properties": {},
		"required": []
	})
}

pub fn onchain_receive_schema() -> Value {
	json!({
		"type": "object",
		"properties": {},
		"required": []
	})
}

pub fn onchain_send_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"address": {
				"type": "string",
				"description": "The Bitcoin address to send coins to"
			},
			"amount_sats": {
				"type": "integer",
				"description": "The amount in satoshis to send. Respects on-chain reserve for anchor channels"
			},
			"send_all": {
				"type": "boolean",
				"description": "If true, send full balance (ignores amount_sats). Warning: will not retain on-chain reserves for anchor channels"
			},
			"fee_rate_sat_per_vb": {
				"type": "integer",
				"description": "Fee rate in satoshis per virtual byte. If not set, a reasonable estimate will be used"
			}
		},
		"required": ["address"]
	})
}

pub fn bolt11_receive_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"amount_msat": {
				"type": "integer",
				"description": "Amount in millisatoshis to request. If unset, a variable-amount invoice is returned"
			},
			"description": {
				"type": "string",
				"description": "Description to attach to the invoice. Mutually exclusive with description_hash"
			},
			"description_hash": {
				"type": "string",
				"description": "SHA-256 hash of the description (hex). Use instead of description for longer text. Mutually exclusive with description"
			},
			"expiry_secs": {
				"type": "integer",
				"description": "Invoice expiry time in seconds (default: 86400)"
			}
		},
		"required": []
	})
}

pub fn bolt11_send_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"invoice": {
				"type": "string",
				"description": "A BOLT11 invoice string to pay"
			},
			"amount_msat": {
				"type": "integer",
				"description": "Amount in millisatoshis. Required when paying a zero-amount invoice"
			},
			"max_total_routing_fee_msat": {
				"type": "integer",
				"description": "Maximum total routing fee in millisatoshis. Defaults to 1% of payment + 50 sats"
			},
			"max_total_cltv_expiry_delta": {
				"type": "integer",
				"description": "Maximum total CLTV delta for the route (default: 1008)"
			},
			"max_path_count": {
				"type": "integer",
				"description": "Maximum number of paths for MPP payments (default: 10)"
			},
			"max_channel_saturation_power_of_half": {
				"type": "integer",
				"description": "Maximum channel capacity share as power of 1/2 (default: 2)"
			}
		},
		"required": ["invoice"]
	})
}

pub fn bolt12_receive_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"description": {
				"type": "string",
				"description": "Description to attach to the offer"
			},
			"amount_msat": {
				"type": "integer",
				"description": "Amount in millisatoshis. If unset, a variable-amount offer is returned"
			},
			"expiry_secs": {
				"type": "integer",
				"description": "Offer expiry time in seconds"
			},
			"quantity": {
				"type": "integer",
				"description": "Number of items requested. Can only be set for fixed-amount offers"
			}
		},
		"required": ["description"]
	})
}

pub fn bolt12_send_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"offer": {
				"type": "string",
				"description": "A BOLT12 offer string to pay"
			},
			"amount_msat": {
				"type": "integer",
				"description": "Amount in millisatoshis. Required when paying a zero-amount offer"
			},
			"quantity": {
				"type": "integer",
				"description": "Number of items requested"
			},
			"payer_note": {
				"type": "string",
				"description": "Note to include for the payee. Reflected back in the invoice"
			},
			"max_total_routing_fee_msat": {
				"type": "integer",
				"description": "Maximum total routing fee in millisatoshis. Defaults to 1% of payment + 50 sats"
			},
			"max_total_cltv_expiry_delta": {
				"type": "integer",
				"description": "Maximum total CLTV delta for the route (default: 1008)"
			},
			"max_path_count": {
				"type": "integer",
				"description": "Maximum number of paths for MPP payments (default: 10)"
			},
			"max_channel_saturation_power_of_half": {
				"type": "integer",
				"description": "Maximum channel capacity share as power of 1/2 (default: 2)"
			}
		},
		"required": ["offer"]
	})
}

pub fn spontaneous_send_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"amount_msat": {
				"type": "integer",
				"description": "The amount in millisatoshis to send"
			},
			"node_id": {
				"type": "string",
				"description": "The hex-encoded public key of the destination node"
			},
			"max_total_routing_fee_msat": {
				"type": "integer",
				"description": "Maximum total routing fee in millisatoshis. Defaults to 1% of payment + 50 sats"
			},
			"max_total_cltv_expiry_delta": {
				"type": "integer",
				"description": "Maximum total CLTV delta for the route (default: 1008)"
			},
			"max_path_count": {
				"type": "integer",
				"description": "Maximum number of paths for MPP payments (default: 10)"
			},
			"max_channel_saturation_power_of_half": {
				"type": "integer",
				"description": "Maximum channel capacity share as power of 1/2 (default: 2)"
			}
		},
		"required": ["amount_msat", "node_id"]
	})
}

pub fn open_channel_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"node_pubkey": {
				"type": "string",
				"description": "The hex-encoded public key of the node to open a channel with"
			},
			"address": {
				"type": "string",
				"description": "Address of the remote peer (IPv4:port, IPv6:port, OnionV3:port, or hostname:port)"
			},
			"channel_amount_sats": {
				"type": "integer",
				"description": "The amount in satoshis to commit to the channel"
			},
			"push_to_counterparty_msat": {
				"type": "integer",
				"description": "Amount in millisatoshis to push to the remote side"
			},
			"announce_channel": {
				"type": "boolean",
				"description": "Whether the channel should be public (default: false)"
			},
			"forwarding_fee_proportional_millionths": {
				"type": "integer",
				"description": "Fee in millionths of a satoshi charged per satoshi forwarded"
			},
			"forwarding_fee_base_msat": {
				"type": "integer",
				"description": "Base fee in millisatoshis for forwarded payments"
			},
			"cltv_expiry_delta": {
				"type": "integer",
				"description": "CLTV delta between incoming and outbound HTLCs"
			}
		},
		"required": ["node_pubkey", "address", "channel_amount_sats"]
	})
}

pub fn splice_in_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"user_channel_id": {
				"type": "string",
				"description": "The local user_channel_id of the channel"
			},
			"counterparty_node_id": {
				"type": "string",
				"description": "The hex-encoded public key of the channel's counterparty node"
			},
			"splice_amount_sats": {
				"type": "integer",
				"description": "The amount in satoshis to splice into the channel"
			}
		},
		"required": ["user_channel_id", "counterparty_node_id", "splice_amount_sats"]
	})
}

pub fn splice_out_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"user_channel_id": {
				"type": "string",
				"description": "The local user_channel_id of the channel"
			},
			"counterparty_node_id": {
				"type": "string",
				"description": "The hex-encoded public key of the channel's counterparty node"
			},
			"splice_amount_sats": {
				"type": "integer",
				"description": "The amount in satoshis to splice out of the channel"
			},
			"address": {
				"type": "string",
				"description": "Bitcoin address for the spliced-out funds. If not set, uses the node's on-chain wallet"
			}
		},
		"required": ["user_channel_id", "counterparty_node_id", "splice_amount_sats"]
	})
}

pub fn close_channel_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"user_channel_id": {
				"type": "string",
				"description": "The local user_channel_id of the channel"
			},
			"counterparty_node_id": {
				"type": "string",
				"description": "The hex-encoded public key of the node to close the channel with"
			}
		},
		"required": ["user_channel_id", "counterparty_node_id"]
	})
}

pub fn force_close_channel_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"user_channel_id": {
				"type": "string",
				"description": "The local user_channel_id of the channel"
			},
			"counterparty_node_id": {
				"type": "string",
				"description": "The hex-encoded public key of the node to close the channel with"
			},
			"force_close_reason": {
				"type": "string",
				"description": "The reason for force-closing the channel"
			}
		},
		"required": ["user_channel_id", "counterparty_node_id"]
	})
}

pub fn list_channels_schema() -> Value {
	json!({
		"type": "object",
		"properties": {},
		"required": []
	})
}

pub fn update_channel_config_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"user_channel_id": {
				"type": "string",
				"description": "The local user_channel_id of the channel"
			},
			"counterparty_node_id": {
				"type": "string",
				"description": "The hex-encoded public key of the counterparty node"
			},
			"forwarding_fee_proportional_millionths": {
				"type": "integer",
				"description": "Fee in millionths of a satoshi charged per satoshi forwarded"
			},
			"forwarding_fee_base_msat": {
				"type": "integer",
				"description": "Base fee in millisatoshis for forwarded payments"
			},
			"cltv_expiry_delta": {
				"type": "integer",
				"description": "CLTV delta between incoming and outbound HTLCs"
			}
		},
		"required": ["user_channel_id", "counterparty_node_id"]
	})
}

pub fn list_payments_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"page_token": {
				"type": "string",
				"description": "Pagination token from a previous response (format: token:index)"
			}
		},
		"required": []
	})
}

pub fn get_payment_details_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"payment_id": {
				"type": "string",
				"description": "The payment ID in hex-encoded form"
			}
		},
		"required": ["payment_id"]
	})
}

pub fn list_forwarded_payments_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"page_token": {
				"type": "string",
				"description": "Pagination token from a previous response (format: token:index)"
			}
		},
		"required": []
	})
}

pub fn connect_peer_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"node_pubkey": {
				"type": "string",
				"description": "The hex-encoded public key of the node to connect to"
			},
			"address": {
				"type": "string",
				"description": "Address of the remote peer (IPv4:port, IPv6:port, OnionV3:port, or hostname:port)"
			},
			"persist": {
				"type": "boolean",
				"description": "Whether to persist the connection for automatic reconnection on restart (default: false)"
			}
		},
		"required": ["node_pubkey", "address"]
	})
}

pub fn disconnect_peer_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"node_pubkey": {
				"type": "string",
				"description": "The hex-encoded public key of the node to disconnect from"
			}
		},
		"required": ["node_pubkey"]
	})
}

pub fn sign_message_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"message": {
				"type": "string",
				"description": "The message to sign"
			}
		},
		"required": ["message"]
	})
}

pub fn verify_signature_schema() -> Value {
	json!({
		"type": "object",
		"properties": {
			"message": {
				"type": "string",
				"description": "The message that was signed"
			},
			"signature": {
				"type": "string",
				"description": "The zbase32-encoded signature to verify"
			},
			"public_key": {
				"type": "string",
				"description": "The hex-encoded public key of the signer"
			}
		},
		"required": ["message", "signature", "public_key"]
	})
}

pub fn export_pathfinding_scores_schema() -> Value {
	json!({
		"type": "object",
		"properties": {},
		"required": []
	})
}
