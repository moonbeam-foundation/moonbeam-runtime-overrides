// Copyright 2019-2022 PureStake Inc.
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

use codec::{Decode, Encode};

#[derive(Debug, Default, Copy, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Snapshot {
	pub gas_limit: u64,
	pub memory_gas: u64,
	pub used_gas: u64,
	pub refunded_gas: i64,
}

impl Snapshot {
	pub fn gas(&self) -> u64 {
		self.gas_limit - self.used_gas - self.memory_gas
	}
}

#[cfg(feature = "before_1200")]
fn convert_snapshot(snapshot: evm_gasometer::Snapshot) -> Snapshot {
	Snapshot {
		gas_limit: snapshot.gas_limit,
		memory_gas: snapshot.memory_gas,
		used_gas: snapshot.used_gas,
		refunded_gas: snapshot.refunded_gas,
	}
}

#[cfg(not(feature = "before_1200"))]
fn convert_snapshot(snapshot_opt: Option<evm_gasometer::Snapshot>) -> Snapshot {
	if let Some(snapshot) = snapshot_opt {
		Snapshot {
			gas_limit: snapshot.gas_limit,
			memory_gas: snapshot.memory_gas,
			used_gas: snapshot.used_gas,
			refunded_gas: snapshot.refunded_gas,
		}
	} else {
		Snapshot::default()
	}
}

#[derive(Debug, Copy, Clone, Encode, Decode, PartialEq, Eq)]
pub enum GasometerEvent {
	RecordCost {
		cost: u64,
		snapshot: Snapshot,
	},
	RecordRefund {
		refund: i64,
		snapshot: Snapshot,
	},
	RecordStipend {
		stipend: u64,
		snapshot: Snapshot,
	},
	RecordDynamicCost {
		gas_cost: u64,
		memory_gas: u64,
		gas_refund: i64,
		snapshot: Snapshot,
	},
	RecordTransaction {
		cost: u64,
		snapshot: Snapshot,
	},
}

#[cfg(feature = "evm-tracing")]
impl From<evm_gasometer::tracing::Event> for GasometerEvent {
	fn from(i: evm_gasometer::tracing::Event) -> Self {
		match i {
			evm_gasometer::tracing::Event::RecordCost { cost, snapshot } => Self::RecordCost {
				cost,
				snapshot: convert_snapshot(snapshot),
			},
			evm_gasometer::tracing::Event::RecordRefund { refund, snapshot } => {
				Self::RecordRefund {
					refund,
					snapshot: convert_snapshot(snapshot),
				}
			}
			evm_gasometer::tracing::Event::RecordStipend { stipend, snapshot } => {
				Self::RecordStipend {
					stipend,
					snapshot: convert_snapshot(snapshot),
				}
			}
			evm_gasometer::tracing::Event::RecordDynamicCost {
				gas_cost,
				memory_gas,
				gas_refund,
				snapshot,
			} => Self::RecordDynamicCost {
				gas_cost,
				memory_gas,
				gas_refund,
				snapshot: convert_snapshot(snapshot),
			},
			evm_gasometer::tracing::Event::RecordTransaction { cost, snapshot } => {
				Self::RecordTransaction {
					cost,
					snapshot: convert_snapshot(snapshot),
				}
			}
		}
	}
}
