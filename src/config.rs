// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::path::PathBuf;

use serde::Deserialize;

const DEFAULT_CONFIG_FILE: &str = "config.toml";
const DEFAULT_CERT_FILE: &str = "tls.crt";
const API_KEY_FILE: &str = "api_key";
const DEFAULT_GRPC_SERVICE_ADDRESS: &str = "127.0.0.1:3536";

fn get_default_data_dir() -> Option<PathBuf> {
	#[cfg(target_os = "macos")]
	{
		#[allow(deprecated)]
		std::env::home_dir().map(|home| home.join("Library/Application Support/ldk-server"))
	}
	#[cfg(target_os = "windows")]
	{
		std::env::var("APPDATA").ok().map(|appdata| PathBuf::from(appdata).join("ldk-server"))
	}
	#[cfg(not(any(target_os = "macos", target_os = "windows")))]
	{
		#[allow(deprecated)]
		std::env::home_dir().map(|home| home.join(".ldk-server"))
	}
}

fn get_default_config_path() -> Option<PathBuf> {
	get_default_data_dir().map(|dir| dir.join(DEFAULT_CONFIG_FILE))
}

fn get_default_cert_path() -> Option<PathBuf> {
	get_default_data_dir().map(|path| path.join(DEFAULT_CERT_FILE))
}

fn get_default_api_key_path(network: &str) -> Option<PathBuf> {
	get_default_data_dir().map(|path| path.join(network).join(API_KEY_FILE))
}

fn hex_encode(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Debug, Deserialize)]
pub struct Config {
	pub node: NodeConfig,
	pub tls: Option<TlsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TlsConfig {
	pub cert_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeConfig {
	#[serde(default = "default_grpc_service_address")]
	pub grpc_service_address: String,
	network: String,
}

fn default_grpc_service_address() -> String {
	DEFAULT_GRPC_SERVICE_ADDRESS.to_string()
}

impl Config {
	pub fn network(&self) -> Result<String, String> {
		match self.node.network.as_str() {
			"bitcoin" | "mainnet" => Ok("bitcoin".to_string()),
			"testnet" => Ok("testnet".to_string()),
			"testnet4" => Ok("testnet4".to_string()),
			"signet" => Ok("signet".to_string()),
			"regtest" => Ok("regtest".to_string()),
			other => Err(format!("Unsupported network: {other}")),
		}
	}
}

fn load_config(path: &PathBuf) -> Result<Config, String> {
	let contents = std::fs::read_to_string(path)
		.map_err(|e| format!("Failed to read config file '{}': {}", path.display(), e))?;
	toml::from_str(&contents)
		.map_err(|e| format!("Failed to parse config file '{}': {}", path.display(), e))
}

pub struct ResolvedConfig {
	pub base_url: String,
	pub api_key: String,
	pub tls_cert_pem: Vec<u8>,
}

pub fn resolve_config(config_path: Option<String>) -> Result<ResolvedConfig, String> {
	let config_path = config_path.map(PathBuf::from).or_else(get_default_config_path);
	let config = match config_path {
		Some(ref path) if path.exists() => Some(load_config(path)?),
		_ => None,
	};

	let base_url = std::env::var("LDK_BASE_URL").ok().or_else(|| {
		config.as_ref().map(|c| c.node.grpc_service_address.clone())
	}).or_else(|| {
		config.as_ref().map(|_| DEFAULT_GRPC_SERVICE_ADDRESS.to_string())
	}).ok_or_else(|| {
		"Base URL not provided. Set LDK_BASE_URL or ensure config file exists at ~/.ldk-server/config.toml".to_string()
	})?;

	let api_key = std::env::var("LDK_API_KEY").ok().or_else(|| {
		let network =
			config.as_ref().and_then(|c| c.network().ok()).unwrap_or("bitcoin".to_string());
		get_default_api_key_path(&network)
			.and_then(|path| std::fs::read(&path).ok())
			.map(|bytes| hex_encode(&bytes))
	}).ok_or_else(|| {
		"API key not provided. Set LDK_API_KEY or ensure the api_key file exists at ~/.ldk-server/[network]/api_key".to_string()
	})?;

	let tls_cert_path = std::env::var("LDK_TLS_CERT_PATH").ok().map(PathBuf::from).or_else(|| {
		config
			.as_ref()
			.and_then(|c| c.tls.as_ref().and_then(|t| t.cert_path.as_ref().map(PathBuf::from)))
			.or_else(get_default_cert_path)
	}).ok_or_else(|| {
		"TLS cert path not provided. Set LDK_TLS_CERT_PATH or ensure config file exists at ~/.ldk-server/config.toml".to_string()
	})?;

	let tls_cert_pem = std::fs::read(&tls_cert_path).map_err(|e| {
		format!("Failed to read server certificate file '{}': {}", tls_cert_path.display(), e)
	})?;

	Ok(ResolvedConfig { base_url, api_key, tls_cert_pem })
}

#[cfg(test)]
mod tests {
	use super::{resolve_config, Config, DEFAULT_GRPC_SERVICE_ADDRESS};

	#[test]
	fn config_defaults_grpc_service_address() {
		let config: Config = toml::from_str(
			r#"
				[node]
				network = "regtest"
			"#,
		)
		.unwrap();

		assert_eq!(config.node.grpc_service_address, DEFAULT_GRPC_SERVICE_ADDRESS);
	}

	#[test]
	fn resolve_config_uses_grpc_service_address_from_config() {
		let temp_dir =
			std::env::temp_dir().join(format!("ldk-server-mcp-config-test-{}", std::process::id()));
		std::fs::create_dir_all(&temp_dir).unwrap();

		let config_path = temp_dir.join("config.toml");
		let cert_path = temp_dir.join("tls.crt");
		std::fs::write(&cert_path, b"test-cert").unwrap();
		std::fs::write(
			&config_path,
			format!(
				r#"
					[node]
					network = "regtest"
					grpc_service_address = "127.0.0.1:4242"

					[tls]
					cert_path = "{}"
				"#,
				cert_path.display()
			),
		)
		.unwrap();

		std::env::set_var("LDK_API_KEY", "deadbeef");
		std::env::set_var("LDK_TLS_CERT_PATH", &cert_path);
		let resolved = resolve_config(Some(config_path.display().to_string())).unwrap();
		std::env::remove_var("LDK_API_KEY");
		std::env::remove_var("LDK_TLS_CERT_PATH");

		assert_eq!(resolved.base_url, "127.0.0.1:4242");
		assert_eq!(resolved.api_key, "deadbeef");
		assert_eq!(resolved.tls_cert_pem, b"test-cert");

		std::fs::remove_dir_all(temp_dir).unwrap();
	}
}
