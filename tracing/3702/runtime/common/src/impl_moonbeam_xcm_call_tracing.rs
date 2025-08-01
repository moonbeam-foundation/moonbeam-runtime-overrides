// Copyright 2019-2025 PureStake Inc.
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

#[macro_export]
macro_rules! impl_moonbeam_xcm_call_tracing {
	{} => {
		use moonbeam_evm_tracer::tracer::EthereumTracingStatus;
		type CallResult =
			Result<
				PostDispatchInfoOf<RuntimeCall>,
				DispatchErrorWithPostInfo<PostDispatchInfoOf<RuntimeCall>>
			>;

		pub struct MoonbeamCall;
		impl CallDispatcher<RuntimeCall> for MoonbeamCall {
			fn dispatch(
				call: RuntimeCall,
				origin: RuntimeOrigin,
			) -> CallResult {
				if let Ok(raw_origin) = TryInto::<RawOrigin<AccountId>>::try_into(origin.clone().caller) {
					match call.clone() {
						RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::transact { xcm_transaction }) |
						RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::transact_through_proxy {
							xcm_transaction, ..
						}) |
						RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::force_transact_as {
							xcm_transaction, ..
					 	}) => {
							use crate::EthereumXcm;
							use moonbeam_evm_tracer::tracer::{
								EthereumTracer,
								EvmTracer,
								EthereumTracingStatus
							};
							use xcm_primitives::{
								XcmToEthereum,
							};
							use frame_support::storage::unhashed;
							use frame_support::traits::Get;

							let dispatch_call = || {
								RuntimeCall::dispatch(
									call,
									match raw_origin {
										RawOrigin::Signed(account_id) => {
											pallet_ethereum_xcm::Origin::XcmEthereumTransaction(
												account_id.into()
											).into()
										},
										origin => origin.into()
									}
								)
							};

							return match EthereumTracer::status() {
								// This runtime instance is used for tracing.
								Some(tracing_status) => {
									match tracing_status {
										// Tracing a block, all calls are done using environmental.
										EthereumTracingStatus::Block => {
											// Each known extrinsic is a new call stack.
											EvmTracer::emit_new();
											let mut res: Option<CallResult> = None;
											EvmTracer::new().trace(|| {
												res = Some(dispatch_call());
											});
											res.expect("Invalid dispatch result")
										},
										// Tracing a transaction, the one matching the trace request
										// is done using environmental, the rest dispatched normally.
										EthereumTracingStatus::Transaction(traced_transaction_hash) => {
											let transaction_hash = xcm_transaction.into_transaction_v2(
												EthereumXcm::nonce(),
												<Runtime as pallet_evm::Config>::ChainId::get(),
												false
											)
											.expect("Invalid transaction conversion")
											.hash();
											if transaction_hash == traced_transaction_hash {
												let mut res: Option<CallResult> = None;
												EvmTracer::new().trace(|| {
													res = Some(dispatch_call());
												});
												// Tracing runtime work is done, just signal instance exit.
												EthereumTracer::transaction_exited();
												return res.expect("Invalid dispatch result");
											}
											dispatch_call()
										},
										// Tracing a transaction that has already been found and
										// executed. There's no need to dispatch the rest of the
										// calls.
										EthereumTracingStatus::TransactionExited => Ok(crate::PostDispatchInfo {
											actual_weight: None,
											pays_fee: frame_support::pallet_prelude::Pays::No,
										}),
									}
								}
								// This runtime instance is importing a block.
								None => dispatch_call()
							};
						},
						_ => {}
					}
				}
				RuntimeCall::dispatch(call, origin)
			}
		}
	}
}
