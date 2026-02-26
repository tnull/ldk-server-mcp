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
	pub rest_service_address: String,
	network: String,
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
	let config = config_path.as_ref().and_then(|p| load_config(p).ok());

	let base_url = std::env::var("LDK_BASE_URL").ok().or_else(|| {
		config.as_ref().map(|c| c.node.rest_service_address.clone())
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
