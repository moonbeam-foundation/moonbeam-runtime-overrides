// Copyright 2025 Moonbeam foundation
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


//! Autogenerated weights for `pallet_bridge_messages`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 47.2.0
//! DATE: 2025-07-01, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-10-0-0-176`, CPU: `Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz`
//! WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./frame-omni-bencher
// v1
// benchmark
// pallet
// --runtime=./target/production/wbuild/moonbeam-runtime/moonbeam_runtime.wasm
// --genesis-builder=runtime
// --genesis-builder-preset=development
// --steps=50
// --repeat=20
// --pallet=pallet_bridge_messages
// --extrinsic=*
// --wasm-execution=compiled
// --header=./file_header.txt
// --template=./benchmarking/frame-weight-template.hbs
// --output=./runtime/moonbeam/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for `pallet_bridge_messages`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_messages::WeightInfo for WeightInfo<T> {
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::InboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::InboundLanes` (`max_values`: None, `max_size`: Some(36920), added: 39395, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	fn receive_single_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `755`
		//  Estimated: `40385`
		// Minimum execution time: 47_995_000 picoseconds.
		Weight::from_parts(49_265_000, 40385)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::InboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::InboundLanes` (`max_values`: None, `max_size`: Some(36920), added: 39395, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 4076]`.
	fn receive_n_messages_proof(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `755`
		//  Estimated: `40385`
		// Minimum execution time: 48_586_000 picoseconds.
		Weight::from_parts(49_398_000, 40385)
			// Standard Error: 13_967
			.saturating_add(Weight::from_parts(9_572_341, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::InboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::InboundLanes` (`max_values`: None, `max_size`: Some(36920), added: 39395, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `755`
		//  Estimated: `40385`
		// Minimum execution time: 52_466_000 picoseconds.
		Weight::from_parts(54_525_000, 40385)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::InboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::InboundLanes` (`max_values`: None, `max_size`: Some(36920), added: 39395, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 16384]`.
	fn receive_single_n_bytes_message_proof(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `755`
		//  Estimated: `40385`
		// Minimum execution time: 46_586_000 picoseconds.
		Weight::from_parts(49_000_574, 40385)
			// Standard Error: 7
			.saturating_add(Weight::from_parts(1_990, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::OutboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::OutboundLanes` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::OutboundMessages` (r:0 w:1)
	/// Proof: `BridgeKusamaMessages::OutboundMessages` (`max_values`: None, `max_size`: Some(65596), added: 68071, mode: `MaxEncodedLen`)
	fn receive_delivery_proof_for_single_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `760`
		//  Estimated: `5370`
		// Minimum execution time: 42_487_000 picoseconds.
		Weight::from_parts(43_949_000, 5370)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::OutboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::OutboundLanes` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::OutboundMessages` (r:0 w:2)
	/// Proof: `BridgeKusamaMessages::OutboundMessages` (`max_values`: None, `max_size`: Some(65596), added: 68071, mode: `MaxEncodedLen`)
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `760`
		//  Estimated: `5370`
		// Minimum execution time: 44_281_000 picoseconds.
		Weight::from_parts(46_008_000, 5370)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::OutboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::OutboundLanes` (`max_values`: None, `max_size`: Some(73), added: 2548, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::OutboundMessages` (r:0 w:2)
	/// Proof: `BridgeKusamaMessages::OutboundMessages` (`max_values`: None, `max_size`: Some(65596), added: 68071, mode: `MaxEncodedLen`)
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `760`
		//  Estimated: `5370`
		// Minimum execution time: 44_640_000 picoseconds.
		Weight::from_parts(46_047_000, 5370)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `BridgeKusamaMessages::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaMessages::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(2), added: 497, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaParachains::ImportedParaHeads` (r:1 w:0)
	/// Proof: `BridgeKusamaParachains::ImportedParaHeads` (`max_values`: Some(64), `max_size`: Some(196), added: 1186, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaMessages::InboundLanes` (r:1 w:1)
	/// Proof: `BridgeKusamaMessages::InboundLanes` (`max_values`: None, `max_size`: Some(36920), added: 39395, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::LaneToBridge` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::LaneToBridge` (`max_values`: None, `max_size`: Some(64), added: 2539, mode: `MaxEncodedLen`)
	/// Storage: `BridgeXcmOverMoonriver::Bridges` (r:1 w:0)
	/// Proof: `BridgeXcmOverMoonriver::Bridges` (`max_values`: None, `max_size`: Some(1905), added: 4380, mode: `MaxEncodedLen`)
	/// Storage: `MessageQueue::BookStateFor` (r:1 w:0)
	/// Proof: `MessageQueue::BookStateFor` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 16384]`.
	fn receive_single_n_bytes_message_proof_with_dispatch(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `755`
		//  Estimated: `40385`
		// Minimum execution time: 48_203_000 picoseconds.
		Weight::from_parts(50_358_093, 40385)
			// Standard Error: 10
			.saturating_add(Weight::from_parts(2_460, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}
