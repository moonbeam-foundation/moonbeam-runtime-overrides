// Copyright 2019-2023 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use {
	clap::Parser,
	std::path::{Path, PathBuf},
	toml_edit::{Array, Document, Formatted, Item, Value},
};

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
	#[arg(long)]
	dir: PathBuf,
	#[arg(long)]
	rev: Option<String>,
	#[arg(long)]
	branch: Option<String>,
	#[arg(long)]
	repo: String,
}

const SHARED_PATH: [&str; 4] = [
	"primitives/rpc/evm-tracing-events",
	"runtime/evm_tracer",
	"primitives/rpc/debug",
	"primitives/ext",
];
const LOCAL_PATH: [&str; 4] = [
	"runtime/moonbeam",
	"runtime/moonriver",
	"runtime/moonbase",
	"runtime/common",
];

fn main() {
	// Parse cmd args
	let args = Args::parse();
	match (&args.rev, &args.branch) {
		(None, None) => panic!("--rev xor --branch must be provided"),
		(Some(_), Some(_)) => panic!("cannot use both --rev and --branch"),
		_ => (),
	}

	let runtimes = update_root_toml(&args);

	for runtime in runtimes {
		let mut path = args.dir.clone();
		path.push(runtime);
		path.push("Cargo.toml");
		update_runtime_toml(&path)
	}
}

/// Update the root level Cargo.toml, and returns the list of runtimes.
fn update_root_toml(args: &Args) -> Vec<String> {
	// Load toml file
	let mut toml_path = args.dir.clone();
	toml_path.push("Cargo.toml");
	println!("- Updating {}", toml_path.display());
	let toml = std::fs::read_to_string(&toml_path).expect("cannot open root toml file");
	let mut toml = toml.parse::<Document>().expect("invalid root toml file");

	// remove `workspace.exclude`
	println!("  - Removing `workspace.exclude`");
	let Some(Item::Table(workspace)) = toml.get_mut("workspace") else {
        panic!("cannot get table [workspace]");
    };
	workspace.remove("exclude");

	// filter `workspace.members`
	println!("  - Cleaning up `workspace.members`");
	let Some(Item::Value(Value::Array(members))) = workspace.get_mut("members") else {
        panic!("cannot get array `members`");
    };

	let mut runtime_list = Vec::new();
	let mut indices_to_remove: Vec<_> = members
		.iter()
		.enumerate()
		.filter_map(|(i, member)| {
			let Value::String(member) = member else {
            return None;
        };

			let member = member.value();

			if member.starts_with("runtime/moon") {
				runtime_list.push(member.clone());
				None
			} else {
				Some(i)
			}
		})
		.collect();

	while let Some(index) = indices_to_remove.pop() {
		members.remove(index);
	}

	// update `workspace.dependencies`
	// - if `path` is "primitives/rpc/evm-tracing-events", add "runtime-1600" feature
	// - if `path` is one of the shared crates, replace by shared path
	// - otherwise replace by git dep
	println!("  - Updating `workspace.dependencies`");
	let Some(Item::Table(dependencies)) = workspace.get_mut("dependencies") else {
        panic!("cannot get table `dependencies`");
    };

	for (dep_name, dep_table) in dependencies.iter_mut() {
		let Item::Value(Value::InlineTable(dep_table)) = dep_table else {
            continue
        };

		// Add feature "runtime-1600" to "evm-tracing-event"
		if dep_name == "evm-tracing-events" {
			let Value::Array(features) = dep_table.get_or_insert("features", Array::new()) else {
                panic!("expected features of `{dep_name}` to be an array or missing");
            };
			features.push("runtime-1600");
		}

		// No path => not moonbeam crate.
		let Some(Value::String(ref mut path)) = dep_table.get_mut("path") else {
            continue
        };

		// If this is a shared crate, update the path and stop there.
		if SHARED_PATH.contains(&path.value().as_str()) {
			*path = Formatted::new(format!("shared/{}", path.value()));
			continue;
		}

		// Otherwise we need to change from path to git dep
		if LOCAL_PATH.contains(&path.value().as_str()) {
			continue;
		}

		dep_table.remove("path");
		dep_table.insert("git", args.repo.clone().into());

		if let Some(rev) = &args.rev {
			dep_table.insert("rev", rev.clone().into());
		}

		if let Some(branch) = &args.branch {
			dep_table.insert("branch", branch.clone().into());
		}
	}

	// write modified toml to disk
	std::fs::write(&toml_path, toml.to_string()).expect("cannot write updated toml");

	runtime_list
}

fn update_runtime_toml(path: &Path) {
	println!("- Enabling evm-tracing feature in {}", path.display());
	let toml = std::fs::read_to_string(&path).expect("cannot open runtime toml file");
	let mut toml = toml.parse::<Document>().expect("invalid runtime toml file");

	let Some(Item::Table(features)) = toml.get_mut("features") else {
		panic!("cannot get features table");
	};

	let Some(Item::Value(Value::Array(default_features))) = features.get_mut("default") else {
		panic!("cannot get default features array");
	};

	default_features.push("evm-tracing");

	std::fs::write(&path, toml.to_string()).expect("cannot write updated toml");
}
