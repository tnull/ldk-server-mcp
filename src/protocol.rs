// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PARSE_ERROR: i64 = -32700;
pub const METHOD_NOT_FOUND: i64 = -32601;
#[allow(dead_code)]
pub const INVALID_PARAMS: i64 = -32602;
#[allow(dead_code)]
pub const INTERNAL_ERROR: i64 = -32603;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
	#[allow(dead_code)]
	pub jsonrpc: String,
	pub id: Option<Value>,
	pub method: String,
	pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
	pub jsonrpc: String,
	pub id: Value,
	pub result: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcErrorResponse {
	pub jsonrpc: String,
	pub id: Value,
	pub error: JsonRpcError,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
	pub code: i64,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<Value>,
}

impl JsonRpcResponse {
	pub fn new(id: Value, result: Value) -> Self {
		Self { jsonrpc: "2.0".to_string(), id, result }
	}
}

impl JsonRpcErrorResponse {
	pub fn new(id: Value, code: i64, message: String) -> Self {
		Self { jsonrpc: "2.0".to_string(), id, error: JsonRpcError { code, message, data: None } }
	}
}
