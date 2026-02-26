// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

mod config;
mod mcp;
mod protocol;
mod tools;

fn main() {
	let mut config_path = None;
	let mut args = std::env::args().skip(1);
	while let Some(arg) = args.next() {
		match arg.as_str() {
			"--config" => {
				config_path = args.next();
				if config_path.is_none() {
					eprintln!("Error: --config requires a path argument");
					std::process::exit(1);
				}
			},
			other => {
				eprintln!("Unknown argument: {other}");
				std::process::exit(1);
			},
		}
	}

	match config::resolve_config(config_path) {
		Ok(cfg) => {
			eprintln!("ldk-server-mcp: config loaded, connecting to {}", cfg.base_url);
		},
		Err(e) => {
			eprintln!("Error: {e}");
			std::process::exit(1);
		},
	}
}
