// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

const NUM_TOOLS: usize = 28;

fn test_cert_path() -> String {
	std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/fixtures/test_cert.pem")
		.to_str()
		.unwrap()
		.to_string()
}

fn cargo_bin_path() -> String {
	let output = Command::new("cargo")
		.args(["build", "--message-format=json"])
		.stderr(Stdio::piped())
		.stdout(Stdio::piped())
		.output()
		.expect("Failed to build binary");

	let stdout = String::from_utf8(output.stdout).unwrap();
	for line in stdout.lines() {
		if let Ok(msg) = serde_json::from_str::<Value>(line) {
			if msg.get("reason").and_then(|r| r.as_str()) == Some("compiler-artifact")
				&& msg.get("target").and_then(|t| t.get("name")).and_then(|n| n.as_str())
					== Some("ldk-server-mcp")
				&& msg.get("executable").and_then(|e| e.as_str()).is_some()
			{
				return msg["executable"].as_str().unwrap().to_string();
			}
		}
	}
	panic!("Could not find compiled binary path");
}

struct McpProcess {
	child: std::process::Child,
	stdin: std::process::ChildStdin,
	reader: BufReader<std::process::ChildStdout>,
}

impl McpProcess {
	fn spawn() -> Self {
		let bin = cargo_bin_path();
		let mut child = Command::new(&bin)
			.env("LDK_BASE_URL", "localhost:19999")
			.env("LDK_API_KEY", "deadbeef")
			.env("LDK_TLS_CERT_PATH", test_cert_path())
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.expect("Failed to spawn MCP process");

		let stdin = child.stdin.take().unwrap();
		let stdout = child.stdout.take().unwrap();
		let reader = BufReader::new(stdout);

		McpProcess { child, stdin, reader }
	}

	fn send(&mut self, msg: &Value) {
		let line = serde_json::to_string(msg).unwrap();
		writeln!(self.stdin, "{}", line).expect("Failed to write to stdin");
		self.stdin.flush().expect("Failed to flush stdin");
	}

	fn recv(&mut self) -> Value {
		let mut line = String::new();
		self.reader.read_line(&mut line).expect("Failed to read from stdout");
		serde_json::from_str(line.trim()).expect("Failed to parse JSON response")
	}
}

impl Drop for McpProcess {
	fn drop(&mut self) {
		let _ = self.child.kill();
		let _ = self.child.wait();
	}
}

#[test]
fn test_initialize() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "initialize",
		"params": {
			"protocolVersion": "2024-11-05",
			"capabilities": {},
			"clientInfo": {"name": "test", "version": "0.1"}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
	assert!(resp["result"]["capabilities"]["tools"].is_object());
	assert_eq!(resp["result"]["serverInfo"]["name"], "ldk-server-mcp");
	assert_eq!(resp["result"]["serverInfo"]["version"], "0.1.0");
}

#[test]
fn test_tools_list() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/list",
		"params": {}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);

	let tools = resp["result"]["tools"].as_array().unwrap();
	assert_eq!(tools.len(), NUM_TOOLS, "Expected {NUM_TOOLS} tools, got {}", tools.len());

	for tool in tools {
		assert!(tool["name"].is_string(), "Tool missing name");
		assert!(tool["description"].is_string(), "Tool missing description");
		assert!(tool["inputSchema"].is_object(), "Tool missing inputSchema");
	}
}

#[test]
fn test_tools_call_unknown_tool() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "nonexistent_tool",
			"arguments": {}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["isError"], true);
	let text = resp["result"]["content"][0]["text"].as_str().unwrap();
	assert!(text.contains("Unknown tool"), "Expected 'Unknown tool' in error, got: {text}");
}

#[test]
fn test_tools_call_unreachable_server() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "get_node_info",
			"arguments": {}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["isError"], true);
	let text = resp["result"]["content"][0]["text"].as_str().unwrap();
	assert!(!text.is_empty(), "Expected non-empty error message");
}

#[test]
fn test_notification_no_response() {
	let mut proc = McpProcess::spawn();

	// Send a notification (no id) - should produce no response
	proc.send(&json!({
		"jsonrpc": "2.0",
		"method": "notifications/initialized"
	}));

	// Send a real request after the notification
	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 42,
		"method": "initialize",
		"params": {
			"protocolVersion": "2024-11-05",
			"capabilities": {},
			"clientInfo": {"name": "test", "version": "0.1"}
		}
	}));

	// The first response we get should be for id 42, not for the notification
	let resp = proc.recv();
	assert_eq!(resp["id"], 42);
}

#[test]
fn test_graph_list_channels_unreachable() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "graph_list_channels",
			"arguments": {}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["isError"], true);
	let text = resp["result"]["content"][0]["text"].as_str().unwrap();
	assert!(!text.is_empty(), "Expected non-empty error message");
}

#[test]
fn test_graph_get_channel_unreachable() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "graph_get_channel",
			"arguments": {"short_channel_id": 12345}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["isError"], true);
	let text = resp["result"]["content"][0]["text"].as_str().unwrap();
	assert!(!text.is_empty(), "Expected non-empty error message");
}

#[test]
fn test_graph_list_nodes_unreachable() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "graph_list_nodes",
			"arguments": {}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["isError"], true);
	let text = resp["result"]["content"][0]["text"].as_str().unwrap();
	assert!(!text.is_empty(), "Expected non-empty error message");
}

#[test]
fn test_graph_get_node_unreachable() {
	let mut proc = McpProcess::spawn();

	proc.send(&json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "graph_get_node",
			"arguments": {"node_id": "02deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"}
		}
	}));

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert_eq!(resp["id"], 1);
	assert_eq!(resp["result"]["isError"], true);
	let text = resp["result"]["content"][0]["text"].as_str().unwrap();
	assert!(!text.is_empty(), "Expected non-empty error message");
}

#[test]
fn test_malformed_json() {
	let mut proc = McpProcess::spawn();

	// Send garbage
	writeln!(proc.stdin, "this is not json").unwrap();
	proc.stdin.flush().unwrap();

	let resp = proc.recv();
	assert_eq!(resp["jsonrpc"], "2.0");
	assert!(resp["error"].is_object());
	assert_eq!(resp["error"]["code"], -32700);
	assert_eq!(resp["error"]["message"], "Parse error");
}
