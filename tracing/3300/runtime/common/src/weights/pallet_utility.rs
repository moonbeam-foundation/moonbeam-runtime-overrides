// Copyright 2024 Moonbeam foundation
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

//! Autogenerated weights for `pallet_utility`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2024-10-10, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-10-0-0-176`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! WASM-EXECUTION: Compiled, CHAIN: Some("moonbase-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/moonbeam
// benchmark
// pallet
// --chain=moonbase-dev
// --steps=50
// --repeat=20
// --pallet=pallet_utility
// --extrinsic=*
// --wasm-execution=compiled
// --header=./file_header.txt
// --template=./benchmarking/frame-weight-template.hbs
// --output=./runtime/common/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for `pallet_utility`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	/// Storage: `MaintenanceMode::MaintenanceMode` (r:1 w:0)
	/// Proof: `MaintenanceMode::MaintenanceMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 3_990_000 picoseconds.
		Weight::from_parts(12_342_261, 1527)
			// Standard Error: 3_575
			.saturating_add(Weight::from_parts(4_042_106, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `MaintenanceMode::MaintenanceMode` (r:1 w:0)
	/// Proof: `MaintenanceMode::MaintenanceMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn as_derivative() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 6_245_000 picoseconds.
		Weight::from_parts(6_479_000, 1527)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `MaintenanceMode::MaintenanceMode` (r:1 w:0)
	/// Proof: `MaintenanceMode::MaintenanceMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 4_108_000 picoseconds.
		Weight::from_parts(7_073_838, 1527)
			// Standard Error: 3_710
			.saturating_add(Weight::from_parts(4_248_482, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	fn dispatch_as() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_148_000 picoseconds.
		Weight::from_parts(6_399_000, 0)
	}
	/// Storage: `MaintenanceMode::MaintenanceMode` (r:1 w:0)
	/// Proof: `MaintenanceMode::MaintenanceMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `42`
		//  Estimated: `1527`
		// Minimum execution time: 3_982_000 picoseconds.
		Weight::from_parts(12_200_562, 1527)
			// Standard Error: 3_522
			.saturating_add(Weight::from_parts(4_044_301, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
}
