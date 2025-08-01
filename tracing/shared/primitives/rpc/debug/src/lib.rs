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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "before_700")]
use ethereum::Transaction as Transaction;
#[cfg(feature = "_700_to_1200")]
use ethereum::TransactionV0 as Transaction;
#[cfg(feature = "runtime-3000")]
use ethereum::TransactionV2 as Transaction;
#[cfg(all(not(feature = "before_700"), not(feature = "_700_to_1200"), not(feature = "runtime-3000")))]
use ethereum::{TransactionV3 as Transaction, AuthorizationList};

use ethereum_types::{H160, H256, U256};
use sp_std::vec::Vec;


#[cfg(feature = "runtime-3900")]
sp_api::decl_runtime_apis! {
	#[api_version(7)]
	pub trait DebugRuntimeApi {
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_call(
			header: &Block::Header,
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
			authorization_list: Option<AuthorizationList>,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}

#[cfg(feature = "runtime-3000")]
sp_api::decl_runtime_apis! {
	#[api_version(6)]
	pub trait DebugRuntimeApi {
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_call(
			header: &Block::Header,
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError>;		
	}
}

#[cfg(feature = "runtime-2900")]
sp_api::decl_runtime_apis! {
	#[api_version(5)]
	pub trait DebugRuntimeApi {
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}

#[cfg(all(
	not(feature = "before_700"),
	not(feature = "_700_to_1200"),
	not(feature = "runtime-2900"),
	not(feature = "runtime-3000"),
	not(feature = "runtime-3900")
))]
sp_api::decl_runtime_apis! {
	#[api_version(4)]
	pub trait DebugRuntimeApi {
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}

#[cfg(any(feature = "before_700", feature = "_700_to_1200"))]
sp_api::decl_runtime_apis! {
	pub trait DebugRuntimeApi {
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TracerInput {
	None,
	Blockscout,
	CallTracer,
}

/// DebugRuntimeApi V2 result. Trace response is stored in client and runtime api call response is
/// empty.
#[derive(Debug)]
pub enum Response {
	Single,
	Block,
}
