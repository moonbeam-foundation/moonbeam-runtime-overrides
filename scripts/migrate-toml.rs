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
	std::path::PathBuf,
	toml_edit::{Array, Document, Formatted, Item, Value},
};

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
	#[arg(long)]
	file: PathBuf,
	#[arg(long)]
	rev: Option<String>,
	#[arg(long)]
	branch: Option<String>,
	#[arg(long)]
	repo: String,
}

fn main() {
	// Parse cmd args
	let args = Args::parse();
	match (&args.rev, &args.branch) {
		(None, None) => panic!("--rev xor --branch must be provided"),
		(Some(_), Some(_)) => panic!("cannot use both --rev and --branch"),
		_ => (),
	}

	// Load toml file
	let toml = std::fs::read_to_string(&args.file).expect("cannot open toml file");
	let mut toml = toml.parse::<Document>().expect("invalid toml file");

	// remove `workspace.exclude`
	let Some(Item::Table(workspace)) = toml.get_mut("workspace") else {
        panic!("cannot get table [workspace]");
    };
	workspace.remove("exclude");

	// filter `workspace.members`
	let Some(Item::Value(Value::Array(members))) = workspace.get_mut("members") else {
        panic!("cannot get array `members`");
    };

	let mut indices_to_remove: Vec<_> = members
		.iter()
		.enumerate()
		.filter_map(|(i, member)| {
			let Value::String(member) = member else {
            return None;
        };

			let member = member.value();

			if member.starts_with("runtime/moon") {
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
	let shared = [
		"primitives/rpc/evm-tracing-events",
		"runtime/evm_tracer",
		"primitives/rpc/debug",
		"primitives/ext",
	];
	let ignore_git = [
		"runtime/moonbeam",
		"runtime/moonriver",
		"runtime/moonbase",
		"runtime/common",
	];

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
		if shared.contains(&path.value().as_str()) {
			*path = Formatted::new(format!("shared/{}", path.value()));
			continue;
		}

		// Otherwise we need to change from path to git dep
		if ignore_git.contains(&path.value().as_str()) {
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
	std::fs::write(&args.file, toml.to_string()).expect("cannot write updated toml");
}
