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

//! Moonbeam Runtime Integration Tests

#![cfg(test)]

mod common;
use common::*;

use fp_evm::Context;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchClass, Dispatchable},
	traits::{
		fungible::Inspect, fungibles::Inspect as FungiblesInspect, Currency as CurrencyT,
		EnsureOrigin, PalletInfo, StorageInfo, StorageInfoTrait,
	},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	StorageHasher, Twox128,
};

use moonbeam_runtime::{
	asset_config::LocalAssetInstance,
	currency::GLMR,
	xcm_config::{CurrencyId, SelfReserve, UnitWeightCost},
	AccountId, Balances, CouncilCollective, CrowdloanRewards, ParachainStaking, PolkadotXcm,
	Precompiles, Runtime, RuntimeBlockWeights, RuntimeCall, RuntimeEvent, System,
	TechCommitteeCollective, TransactionPayment, TreasuryCouncilCollective, XTokens, XcmTransactor,
	FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX, LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};
use nimbus_primitives::NimbusId;
use pallet_evm::PrecompileSet;
use pallet_evm_precompileset_assets_erc20::{
	AccountIdAssetIdConversion, IsLocal, SELECTOR_LOG_APPROVAL, SELECTOR_LOG_TRANSFER,
};
use pallet_transaction_payment::Multiplier;
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use parity_scale_codec::Encode;
use polkadot_parachain::primitives::Sibling;
use precompile_utils::{precompile_set::IsActivePrecompile, prelude::*, testing::*};
use sha3::{Digest, Keccak256};
use sp_core::{ByteArray, Pair, H160, U256};
use sp_runtime::{traits::Convert, DispatchError, ModuleError, TokenError};
use std::str::from_utf8;
use xcm::latest::prelude::*;
use xcm::{VersionedMultiAsset, VersionedMultiAssets, VersionedMultiLocation};
use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};
use xcm_executor::traits::Convert as XcmConvert;

type BatchPCall = pallet_evm_precompile_batch::BatchPrecompileCall<Runtime>;
type CrowdloanRewardsPCall =
	pallet_evm_precompile_crowdloan_rewards::CrowdloanRewardsPrecompileCall<Runtime>;
type XcmUtilsPCall = pallet_evm_precompile_xcm_utils::XcmUtilsPrecompileCall<
	Runtime,
	moonbeam_runtime::xcm_config::XcmExecutorConfig,
>;
type XtokensPCall = pallet_evm_precompile_xtokens::XtokensPrecompileCall<Runtime>;
type LocalAssetsPCall = pallet_evm_precompileset_assets_erc20::Erc20AssetsPrecompileSetCall<
	Runtime,
	IsLocal,
	LocalAssetInstance,
>;
type XcmTransactorV2PCall =
	pallet_evm_precompile_xcm_transactor::v2::XcmTransactorPrecompileV2Call<Runtime>;

const BASE_FEE_GENESIS: u128 = 10000 * GIGAWEI;

#[test]
fn xcmp_queue_controller_origin_is_root() {
	// important for the XcmExecutionManager impl of PauseExecution which uses root origin
	// to suspend/resume XCM execution in xcmp_queue::on_idle
	assert_ok!(
		<moonbeam_runtime::Runtime as cumulus_pallet_xcmp_queue::Config
		>::ControllerOrigin::ensure_origin(root_origin())
	);
}

#[test]
fn fast_track_available() {
	assert!(moonbeam_runtime::get!(
		pallet_democracy,
		InstantAllowed,
		bool
	));
}

#[test]
fn verify_pallet_prefixes() {
	fn is_pallet_prefix<P: 'static>(name: &str) {
		// Compares the unhashed pallet prefix in the `StorageInstance` implementation by every
		// storage item in the pallet P. This pallet prefix is used in conjunction with the
		// item name to get the unique storage key: hash(PalletPrefix) + hash(StorageName)
		// https://github.com/paritytech/substrate/blob/master/frame/support/procedural/src/pallet/
		// expand/storage.rs#L389-L401
		assert_eq!(
			<moonbeam_runtime::Runtime as frame_system::Config>::PalletInfo::name::<P>(),
			Some(name)
		);
	}
	// TODO: use StorageInfoTrait once https://github.com/paritytech/substrate/pull/9246
	// is pulled in substrate deps.
	is_pallet_prefix::<moonbeam_runtime::System>("System");
	is_pallet_prefix::<moonbeam_runtime::Utility>("Utility");
	is_pallet_prefix::<moonbeam_runtime::ParachainSystem>("ParachainSystem");
	is_pallet_prefix::<moonbeam_runtime::TransactionPayment>("TransactionPayment");
	is_pallet_prefix::<moonbeam_runtime::ParachainInfo>("ParachainInfo");
	is_pallet_prefix::<moonbeam_runtime::EthereumChainId>("EthereumChainId");
	is_pallet_prefix::<moonbeam_runtime::EVM>("EVM");
	is_pallet_prefix::<moonbeam_runtime::Ethereum>("Ethereum");
	is_pallet_prefix::<moonbeam_runtime::ParachainStaking>("ParachainStaking");
	is_pallet_prefix::<moonbeam_runtime::Scheduler>("Scheduler");
	is_pallet_prefix::<moonbeam_runtime::Democracy>("Democracy");
	is_pallet_prefix::<moonbeam_runtime::CouncilCollective>("CouncilCollective");
	is_pallet_prefix::<moonbeam_runtime::TechCommitteeCollective>("TechCommitteeCollective");
	is_pallet_prefix::<moonbeam_runtime::Treasury>("Treasury");
	is_pallet_prefix::<moonbeam_runtime::AuthorInherent>("AuthorInherent");
	is_pallet_prefix::<moonbeam_runtime::AuthorFilter>("AuthorFilter");
	is_pallet_prefix::<moonbeam_runtime::CrowdloanRewards>("CrowdloanRewards");
	is_pallet_prefix::<moonbeam_runtime::AuthorMapping>("AuthorMapping");
	is_pallet_prefix::<moonbeam_runtime::MaintenanceMode>("MaintenanceMode");
	is_pallet_prefix::<moonbeam_runtime::Identity>("Identity");
	is_pallet_prefix::<moonbeam_runtime::XcmpQueue>("XcmpQueue");
	is_pallet_prefix::<moonbeam_runtime::CumulusXcm>("CumulusXcm");
	is_pallet_prefix::<moonbeam_runtime::DmpQueue>("DmpQueue");
	is_pallet_prefix::<moonbeam_runtime::PolkadotXcm>("PolkadotXcm");
	is_pallet_prefix::<moonbeam_runtime::Assets>("Assets");
	is_pallet_prefix::<moonbeam_runtime::XTokens>("XTokens");
	is_pallet_prefix::<moonbeam_runtime::AssetManager>("AssetManager");
	is_pallet_prefix::<moonbeam_runtime::Migrations>("Migrations");
	is_pallet_prefix::<moonbeam_runtime::XcmTransactor>("XcmTransactor");
	is_pallet_prefix::<moonbeam_runtime::ProxyGenesisCompanion>("ProxyGenesisCompanion");
	is_pallet_prefix::<moonbeam_runtime::LocalAssets>("LocalAssets");
	is_pallet_prefix::<moonbeam_runtime::MoonbeamOrbiters>("MoonbeamOrbiters");
	is_pallet_prefix::<moonbeam_runtime::TreasuryCouncilCollective>("TreasuryCouncilCollective");
	let prefix = |pallet_name, storage_name| {
		let mut res = [0u8; 32];
		res[0..16].copy_from_slice(&Twox128::hash(pallet_name));
		res[16..32].copy_from_slice(&Twox128::hash(storage_name));
		res.to_vec()
	};
	assert_eq!(
		<moonbeam_runtime::Timestamp as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Timestamp".to_vec(),
				storage_name: b"Now".to_vec(),
				prefix: prefix(b"Timestamp", b"Now"),
				max_values: Some(1),
				max_size: Some(8),
			},
			StorageInfo {
				pallet_name: b"Timestamp".to_vec(),
				storage_name: b"DidUpdate".to_vec(),
				prefix: prefix(b"Timestamp", b"DidUpdate"),
				max_values: Some(1),
				max_size: Some(1),
			}
		]
	);
	assert_eq!(
		<moonbeam_runtime::Balances as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"TotalIssuance".to_vec(),
				prefix: prefix(b"Balances", b"TotalIssuance"),
				max_values: Some(1),
				max_size: Some(16),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"InactiveIssuance".to_vec(),
				prefix: prefix(b"Balances", b"InactiveIssuance"),
				max_values: Some(1),
				max_size: Some(16),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Account".to_vec(),
				prefix: prefix(b"Balances", b"Account"),
				max_values: None,
				max_size: Some(100),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Locks".to_vec(),
				prefix: prefix(b"Balances", b"Locks"),
				max_values: None,
				max_size: Some(1287),
			},
			StorageInfo {
				pallet_name: b"Balances".to_vec(),
				storage_name: b"Reserves".to_vec(),
				prefix: prefix(b"Balances", b"Reserves"),
				max_values: None,
				max_size: Some(1037),
			},
		]
	);
	assert_eq!(
		<moonbeam_runtime::Proxy as StorageInfoTrait>::storage_info(),
		vec![
			StorageInfo {
				pallet_name: b"Proxy".to_vec(),
				storage_name: b"Proxies".to_vec(),
				prefix: prefix(b"Proxy", b"Proxies"),
				max_values: None,
				max_size: Some(845),
			},
			StorageInfo {
				pallet_name: b"Proxy".to_vec(),
				storage_name: b"Announcements".to_vec(),
				prefix: prefix(b"Proxy", b"Announcements"),
				max_values: None,
				max_size: Some(1837),
			}
		]
	);
	assert_eq!(
		<moonbeam_runtime::MaintenanceMode as StorageInfoTrait>::storage_info(),
		vec![StorageInfo {
			pallet_name: b"MaintenanceMode".to_vec(),
			storage_name: b"MaintenanceMode".to_vec(),
			prefix: prefix(b"MaintenanceMode", b"MaintenanceMode"),
			max_values: Some(1),
			max_size: None,
		},]
	);
}

#[test]
fn test_collectives_storage_item_prefixes() {
	for StorageInfo { pallet_name, .. } in
		<moonbeam_runtime::CouncilCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"CouncilCollective".to_vec());
	}

	for StorageInfo { pallet_name, .. } in
		<moonbeam_runtime::TechCommitteeCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"TechCommitteeCollective".to_vec());
	}

	for StorageInfo { pallet_name, .. } in
		<moonbeam_runtime::TreasuryCouncilCollective as StorageInfoTrait>::storage_info()
	{
		assert_eq!(pallet_name, b"TreasuryCouncilCollective".to_vec());
	}
}

#[test]
fn collective_set_members_root_origin_works() {
	ExtBuilder::default().build().execute_with(|| {
		// CouncilCollective
		assert_ok!(CouncilCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::root(),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		));
		// TechCommitteeCollective
		assert_ok!(TechCommitteeCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::root(),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		));
		// TreasuryCouncilCollective
		assert_ok!(TreasuryCouncilCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::root(),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		));
	});
}

#[test]
fn collective_set_members_signed_origin_does_not_work() {
	let alice = AccountId::from(ALICE);
	ExtBuilder::default().build().execute_with(|| {
		// CouncilCollective
		assert!(CouncilCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::signed(alice),
			vec![alice, AccountId::from(BOB)],
			Some(alice),
			2
		)
		.is_err());
		// TechCommitteeCollective
		assert!(TechCommitteeCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::signed(alice),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		)
		.is_err());
		// TreasuryCouncilCollective
		assert!(TreasuryCouncilCollective::set_members(
			<Runtime as frame_system::Config>::RuntimeOrigin::signed(alice),
			vec![AccountId::from(ALICE), AccountId::from(BOB)],
			Some(AccountId::from(ALICE)),
			2
		)
		.is_err());
	});
}

#[test]
fn verify_pallet_indices() {
	fn is_pallet_index<P: 'static>(index: usize) {
		assert_eq!(
			<moonbeam_runtime::Runtime as frame_system::Config>::PalletInfo::index::<P>(),
			Some(index)
		);
	}

	// System support
	is_pallet_index::<moonbeam_runtime::System>(0);
	is_pallet_index::<moonbeam_runtime::ParachainSystem>(1);
	is_pallet_index::<moonbeam_runtime::Timestamp>(3);
	is_pallet_index::<moonbeam_runtime::ParachainInfo>(4);
	// Monetary
	is_pallet_index::<moonbeam_runtime::Balances>(10);
	is_pallet_index::<moonbeam_runtime::TransactionPayment>(11);
	// Consensus support
	is_pallet_index::<moonbeam_runtime::ParachainStaking>(20);
	is_pallet_index::<moonbeam_runtime::AuthorInherent>(21);
	is_pallet_index::<moonbeam_runtime::AuthorFilter>(22);
	is_pallet_index::<moonbeam_runtime::AuthorMapping>(23);
	is_pallet_index::<moonbeam_runtime::MoonbeamOrbiters>(24);
	// Handy utilities
	is_pallet_index::<moonbeam_runtime::Utility>(30);
	is_pallet_index::<moonbeam_runtime::Proxy>(31);
	is_pallet_index::<moonbeam_runtime::MaintenanceMode>(32);
	is_pallet_index::<moonbeam_runtime::Identity>(33);
	is_pallet_index::<moonbeam_runtime::Migrations>(34);
	is_pallet_index::<moonbeam_runtime::ProxyGenesisCompanion>(35);
	// Ethereum compatibility
	is_pallet_index::<moonbeam_runtime::EthereumChainId>(50);
	is_pallet_index::<moonbeam_runtime::EVM>(51);
	is_pallet_index::<moonbeam_runtime::Ethereum>(52);
	// Governance
	is_pallet_index::<moonbeam_runtime::Scheduler>(60);
	is_pallet_index::<moonbeam_runtime::Democracy>(61);
	// Council
	is_pallet_index::<moonbeam_runtime::CouncilCollective>(70);
	is_pallet_index::<moonbeam_runtime::TechCommitteeCollective>(71);
	is_pallet_index::<moonbeam_runtime::TreasuryCouncilCollective>(72);
	// Treasury
	is_pallet_index::<moonbeam_runtime::Treasury>(80);
	// Crowdloan
	is_pallet_index::<moonbeam_runtime::CrowdloanRewards>(90);
	// XCM Stuff
	is_pallet_index::<moonbeam_runtime::XcmpQueue>(100);
	is_pallet_index::<moonbeam_runtime::CumulusXcm>(101);
	is_pallet_index::<moonbeam_runtime::DmpQueue>(102);
	is_pallet_index::<moonbeam_runtime::PolkadotXcm>(103);
	is_pallet_index::<moonbeam_runtime::Assets>(104);
	is_pallet_index::<moonbeam_runtime::AssetManager>(105);
	is_pallet_index::<moonbeam_runtime::XTokens>(106);
	is_pallet_index::<moonbeam_runtime::XcmTransactor>(107);
	is_pallet_index::<moonbeam_runtime::LocalAssets>(108);
}

#[test]
fn verify_reserved_indices() {
	use frame_support::metadata::*;
	let metadata = moonbeam_runtime::Runtime::metadata();
	let metadata = match metadata.1 {
		RuntimeMetadata::V14(metadata) => metadata,
		_ => panic!("metadata has been bumped, test needs to be updated"),
	};
	// 40: Sudo
	// 53: BaseFee
	let reserved = vec![40, 53];
	let existing = metadata
		.pallets
		.iter()
		.map(|p| p.index)
		.collect::<Vec<u8>>();
	assert!(reserved.iter().all(|index| !existing.contains(index)));
}

#[test]
fn verify_proxy_type_indices() {
	assert_eq!(moonbeam_runtime::ProxyType::Any as u8, 0);
	assert_eq!(moonbeam_runtime::ProxyType::NonTransfer as u8, 1);
	assert_eq!(moonbeam_runtime::ProxyType::Governance as u8, 2);
	assert_eq!(moonbeam_runtime::ProxyType::Staking as u8, 3);
	assert_eq!(moonbeam_runtime::ProxyType::CancelProxy as u8, 4);
	assert_eq!(moonbeam_runtime::ProxyType::Balances as u8, 5);
	assert_eq!(moonbeam_runtime::ProxyType::AuthorMapping as u8, 6);
	assert_eq!(moonbeam_runtime::ProxyType::IdentityJudgement as u8, 7);
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 10_000_000 * GLMR),
			(AccountId::from(BOB), 10_000_000 * GLMR),
			(AccountId::from(CHARLIE), 10_000_000 * GLMR),
			(AccountId::from(DAVE), 10_000_000 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 2_000_000 * GLMR),
			(AccountId::from(BOB), 2_000_000 * GLMR),
		])
		.with_delegations(vec![
			(
				AccountId::from(CHARLIE),
				AccountId::from(ALICE),
				5_000 * GLMR,
			),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 5_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(ALICE)),
					2_000_000 * GLMR,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					2_000_000 * GLMR,
					2u32
				),
				pallet_parachain_staking::Error::<Runtime>::DelegatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				2_000_000 * GLMR,
				2u32
			));
			assert_eq!(
				last_event(),
				RuntimeEvent::ParachainStaking(
					pallet_parachain_staking::Event::JoinedCollatorCandidates {
						account: AccountId::from(DAVE),
						amount_locked: 2_000_000 * GLMR,
						new_total_amt_locked: 6_010_000 * GLMR
					}
				)
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(ALICE));
			assert_eq!(candidates.0[0].amount, 2_005_000 * GLMR);
			assert_eq!(candidates.0[1].owner, AccountId::from(BOB));
			assert_eq!(candidates.0[1].amount, 2_005_000 * GLMR);
			assert_eq!(candidates.0[2].owner, AccountId::from(DAVE));
			assert_eq!(candidates.0[2].amount, 2_000_000 * GLMR);
		});
}

#[test]
fn transfer_through_evm_to_stake() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 10_000_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Charlie has no balance => fails to stake
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					2_000_000 * GLMR,
					2u32
				),
				DispatchError::Module(ModuleError {
					index: 20,
					error: [8, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
			);
			// Alice transfer from free balance 3_000_000 GLMR to Bob
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				3_000_000 * GLMR,
			));
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				3_000_000 * GLMR
			);

			let gas_limit = 100000u64;
			let gas_price: U256 = BASE_FEE_GENESIS.into();
			// Bob transfers 2_000_000 GLMR to Charlie via EVM
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(CHARLIE),
				input: vec![],
				value: (2_000_000 * GLMR).into(),
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None,
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				2_000_000 * GLMR,
			);

			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				2_000_000 * GLMR,
				2u32
			),);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(candidates.0[0].owner, AccountId::from(CHARLIE));
			assert_eq!(candidates.0[0].amount, 2_000_000 * GLMR);
		});
}

#[test]
fn reward_block_authors() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 10k extra tokens for her mapping deposit
			(AccountId::from(ALICE), 10_010_000 * GLMR),
			(AccountId::from(BOB), 10_000_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 2_000_000 * GLMR)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			50_000 * GLMR,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			for x in 2..3599 {
				run_to_block(x, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			}
			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8_000_000 * GLMR,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9_950_000 * GLMR,
			);
			run_to_block(3601, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8980978048702400000000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9969521950497200000000000,
			);
		});
}

#[test]
fn reward_block_authors_with_parachain_bond_reserved() {
	ExtBuilder::default()
		.with_balances(vec![
			// Alice gets 10k extra tokens for her mapping deposit
			(AccountId::from(ALICE), 10_010_000 * GLMR),
			(AccountId::from(BOB), 10_000_000 * GLMR),
			(AccountId::from(CHARLIE), 10_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 2_000_000 * GLMR)])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			50_000 * GLMR,
		)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			assert_ok!(ParachainStaking::set_parachain_bond_account(
				root_origin(),
				AccountId::from(CHARLIE),
			),);
			for x in 2..3599 {
				run_to_block(x, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			}
			// no rewards doled out yet
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8_000_000 * GLMR,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9_950_000 * GLMR,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				10_000 * GLMR,
			);
			run_to_block(3601, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			// rewards minted and distributed
			assert_eq!(
				Balances::usable_balance(AccountId::from(ALICE)),
				8688492682878000000000000,
			);
			assert_eq!(
				Balances::usable_balance(AccountId::from(BOB)),
				9962207316621500000000000,
			);
			// 30% reserved for parachain bond
			assert_eq!(
				Balances::usable_balance(AccountId::from(CHARLIE)),
				310300000000000000000000,
			);
		});
}

#[test]
fn initialize_crowdloan_addresses_with_batch_and_pay() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 200_000 * GLMR),
			(AccountId::from(BOB), 100_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(300_000_000 * GLMR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									150_000_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									150_000_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);
			// 30 percent initial payout
			assert_eq!(
				Balances::balance(&AccountId::from(CHARLIE)),
				45_000_000 * GLMR
			);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 45_000_000 * GLMR);
			let expected = RuntimeEvent::Utility(pallet_utility::Event::BatchCompleted);
			assert_eq!(last_event(), expected);
			// This one should fail, as we already filled our data
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch {
					calls: vec![RuntimeCall::CrowdloanRewards(
						pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
							rewards: vec![(
								[4u8; 32].into(),
								Some(AccountId::from(ALICE)),
								43_200_000
							)]
						}
					)]
				})
				.dispatch(root_origin())
			);
			let expected_fail = RuntimeEvent::Utility(pallet_utility::Event::BatchInterrupted {
				index: 0,
				error: DispatchError::Module(ModuleError {
					index: 90,
					error: [8, 0, 0, 0],
					message: None,
				}),
			});
			assert_eq!(last_event(), expected_fail);
			// Claim 1 block.
			assert_ok!(CrowdloanRewards::claim(origin_of(AccountId::from(CHARLIE))));
			assert_ok!(CrowdloanRewards::claim(origin_of(AccountId::from(DAVE))));

			let vesting_period = 4 * WEEKS as u128;
			let per_block = (105_000_000 * GLMR) / vesting_period;

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE))
					.unwrap()
					.claimed_reward,
				(45_000_000 * GLMR) + per_block
			);
			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(DAVE))
					.unwrap()
					.claimed_reward,
				(45_000_000 * GLMR) + per_block
			);
			// The total claimed reward should be equal to the account balance at this point.
			assert_eq!(
				Balances::balance(&AccountId::from(CHARLIE)),
				(45_000_000 * GLMR) + per_block
			);
			assert_eq!(
				Balances::balance(&AccountId::from(DAVE)),
				(45_000_000 * GLMR) + per_block
			);
			assert_noop!(
				CrowdloanRewards::claim(origin_of(AccountId::from(ALICE))),
				pallet_crowdloan_rewards::Error::<Runtime>::NoAssociatedClaim
			);
		});
}

#[test]
fn initialize_crowdloan_address_and_change_with_relay_key_sig() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * GLMR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;

			let (pair1, _) = sp_core::sr25519::Pair::generate();
			let (pair2, _) = sp_core::sr25519::Pair::generate();

			let public1 = pair1.public();
			let public2 = pair2.public();

			// signature is new_account || previous_account
			let mut message = pallet_crowdloan_rewards::WRAPPED_BYTES_PREFIX.to_vec();
			message.append(&mut b"moonbeam-".to_vec());
			message.append(&mut AccountId::from(DAVE).encode());
			message.append(&mut AccountId::from(CHARLIE).encode());
			message.append(&mut pallet_crowdloan_rewards::WRAPPED_BYTES_POSTFIX.to_vec());
			let signature1 = pair1.sign(&message);
			let signature2 = pair2.sign(&message);

			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				// two relay accounts pointing at the same reward account
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public1.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									public2.into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 900_000 * GLMR);

			// this should fail, as we are only providing one signature
			assert_noop!(
				CrowdloanRewards::change_association_with_relay_keys(
					origin_of(AccountId::from(CHARLIE)),
					AccountId::from(DAVE),
					AccountId::from(CHARLIE),
					vec![(public1.into(), signature1.clone().into())]
				),
				pallet_crowdloan_rewards::Error::<Runtime>::InsufficientNumberOfValidProofs
			);

			// this should be valid
			assert_ok!(CrowdloanRewards::change_association_with_relay_keys(
				origin_of(AccountId::from(CHARLIE)),
				AccountId::from(DAVE),
				AccountId::from(CHARLIE),
				vec![
					(public1.into(), signature1.into()),
					(public2.into(), signature2.into())
				]
			));

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(DAVE))
					.unwrap()
					.claimed_reward,
				(900_000 * GLMR)
			);
		});
}

#[test]
fn claim_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * GLMR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			assert!(CrowdloanRewards::initialized());

			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(CHARLIE)), 450_000 * GLMR);
			// 30 percent initial payout
			assert_eq!(Balances::balance(&AccountId::from(DAVE)), 450_000 * GLMR);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Alice uses the crowdloan precompile to claim through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = BASE_FEE_GENESIS.into();

			// Construct the call data (selector, amount)
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&Keccak256::digest(b"claim()")[0..4]);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(CHARLIE),
				target: crowdloan_precompile_address,
				input: call_data,
				value: U256::zero(), // No value sent in EVM
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let vesting_period = 4 * WEEKS as u128;
			let per_block = (1_050_000 * GLMR) / vesting_period;

			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE))
					.unwrap()
					.claimed_reward,
				(450_000 * GLMR) + per_block
			);
		})
}

#[test]
fn is_contributor_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 200_000 * GLMR),
			(AccountId::from(BOB), 100_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000_000 * GLMR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Assert precompile reports Bob is not a contributor
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					CrowdloanRewardsPCall::is_contributor {
						contributor: Address(AccountId::from(BOB).into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(false);

			// Assert precompile reports Charlie is a nominator
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					CrowdloanRewardsPCall::is_contributor {
						contributor: Address(AccountId::from(CHARLIE).into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(true);
		})
}

#[test]
fn reward_info_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 200_000 * GLMR),
			(AccountId::from(BOB), 100_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * GLMR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			let expected_total: U256 = (1_500_000 * GLMR).into();
			let expected_claimed: U256 = (450_000 * GLMR).into();

			// Assert precompile reports correct Charlie reward info.
			Precompiles::new()
				.prepare_test(
					ALICE,
					crowdloan_precompile_address,
					CrowdloanRewardsPCall::reward_info {
						contributor: Address(AccountId::from(CHARLIE).into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(expected_total)
						.write(expected_claimed)
						.build(),
				);
		})
}

#[test]
fn update_reward_address_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_crowdloan_fund(3_000_000 * GLMR)
		.build()
		.execute_with(|| {
			// set parachain inherent data
			set_parachain_inherent_data();
			let init_block = CrowdloanRewards::init_vesting_block();
			// This matches the previous vesting
			let end_block = init_block + 4 * WEEKS;
			// Batch calls always succeed. We just need to check the inner event
			assert_ok!(
				RuntimeCall::Utility(pallet_utility::Call::<Runtime>::batch_all {
					calls: vec![
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[4u8; 32].into(),
									Some(AccountId::from(CHARLIE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::initialize_reward_vec {
								rewards: vec![(
									[5u8; 32].into(),
									Some(AccountId::from(DAVE)),
									1_500_000 * GLMR
								)]
							}
						),
						RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::<Runtime>::complete_initialization {
								lease_ending_block: end_block
							}
						)
					]
				})
				.dispatch(root_origin())
			);

			let crowdloan_precompile_address = H160::from_low_u64_be(2049);

			// Charlie uses the crowdloan precompile to update address through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = BASE_FEE_GENESIS.into();

			// Construct the input data to check if Bob is a contributor
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"update_reward_address(address)")[0..4]);
			call_data[16..36].copy_from_slice(&ALICE);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(CHARLIE),
				target: crowdloan_precompile_address,
				input: call_data,
				value: U256::zero(), // No value sent in EVM
				gas_limit,
				max_fee_per_gas: gas_price,
				max_priority_fee_per_gas: None,
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			assert!(CrowdloanRewards::accounts_payable(&AccountId::from(CHARLIE)).is_none());
			assert_eq!(
				CrowdloanRewards::accounts_payable(&AccountId::from(ALICE))
					.unwrap()
					.claimed_reward,
				(450_000 * GLMR)
			);
		})
}

fn run_with_system_weight<F>(w: Weight, mut assertions: F)
where
	F: FnMut() -> (),
{
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap()
		.into();
	t.execute_with(|| {
		System::set_block_consumed_resources(w, 0);
		assertions()
	});
}

#[test]
#[rustfmt::skip]
fn length_fee_is_sensible() {
	use sp_runtime::testing::TestXt;

	// tests that length fee is sensible for a few hypothetical transactions
	ExtBuilder::default().build().execute_with(|| {
		let call = frame_system::Call::remark::<Runtime> { remark: vec![] };
		let uxt: TestXt<_, ()> = TestXt::new(call, Some((1u64, ())));

		let calc_fee = |len: u32| -> Balance {
			moonbeam_runtime::TransactionPayment::query_fee_details(uxt.clone(), len)
				.inclusion_fee
				.expect("fee should be calculated")
				.len_fee
		};

		// editorconfig-checker-disable
		//                  left: cost of length fee, right: size in bytes
		//                             /------------- proportional component: O(N * 1B)
		//                             |           /- exponential component: O(N ** 3)
		//                             |           |
		assert_eq!(                    100_000_000_100, calc_fee(1));
		assert_eq!(                  1_000_000_100_000, calc_fee(10));
		assert_eq!(                 10_000_100_000_000, calc_fee(100));
		assert_eq!(                100_100_000_000_000, calc_fee(1_000));
		assert_eq!(              1_100_000_000_000_000, calc_fee(10_000)); // inflection point
		assert_eq!(            110_000_000_000_000_000, calc_fee(100_000));
		assert_eq!(        100_100_000_000_000_000_000, calc_fee(1_000_000)); // 100 GLMR, ~ 1MB
		assert_eq!(    100_001_000_000_000_000_000_000, calc_fee(10_000_000));
		assert_eq!(100_000_010_000_000_000_000_000_000, calc_fee(100_000_000));
		// editorconfig-checker-enable
	});
}

#[test]
fn multiplier_can_grow_from_zero() {
	use frame_support::traits::Get;

	let minimum_multiplier = moonbeam_runtime::MinimumMultiplier::get();
	let target = moonbeam_runtime::TargetBlockFullness::get()
		* RuntimeBlockWeights::get()
			.get(DispatchClass::Normal)
			.max_total
			.unwrap();
	// if the min is too small, then this will not change, and we are doomed forever.
	// the weight is 1/100th bigger than target.
	run_with_system_weight(target * 101 / 100, || {
		let next = moonbeam_runtime::SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
		assert!(
			next > minimum_multiplier,
			"{:?} !>= {:?}",
			next,
			minimum_multiplier
		);
	})
}

#[test]
fn ethereum_invalid_transaction() {
	ExtBuilder::default().build().execute_with(|| {
		// Ensure an extrinsic not containing enough gas limit to store the transaction
		// on chain is rejected.
		assert_eq!(
			Executive::apply_extrinsic(unchecked_eth_tx(INVALID_ETH_TX)),
			Err(
				sp_runtime::transaction_validity::TransactionValidityError::Invalid(
					sp_runtime::transaction_validity::InvalidTransaction::Custom(3u8)
				)
			)
		);
	});
}

#[test]
fn initial_gas_fee_is_correct() {
	use fp_evm::FeeCalculator;

	ExtBuilder::default().build().execute_with(|| {
		let multiplier = TransactionPayment::next_fee_multiplier();
		assert_eq!(multiplier, Multiplier::from(1u128));

		assert_eq!(
			TransactionPaymentAsGasPrice::min_gas_price(),
			(
				125_000_000_000u128.into(),
				Weight::from_ref_time(25_000_000u64)
			)
		);
	});
}

#[test]
fn min_gas_fee_is_correct() {
	use fp_evm::FeeCalculator;
	use frame_support::traits::Hooks;

	ExtBuilder::default().build().execute_with(|| {
		pallet_transaction_payment::NextFeeMultiplier::<Runtime>::put(Multiplier::from(0));
		TransactionPayment::on_finalize(System::block_number()); // should trigger min to kick in

		let multiplier = TransactionPayment::next_fee_multiplier();
		assert_eq!(multiplier, Multiplier::from(1u128));

		assert_eq!(
			TransactionPaymentAsGasPrice::min_gas_price(),
			(
				125_000_000_000u128.into(),
				Weight::from_ref_time(25_000_000u64)
			)
		);
	});
}

#[test]
fn transfer_ed_0_substrate() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), (1 * GLMR) + (1 * WEI)),
			(AccountId::from(BOB), 0),
		])
		.build()
		.execute_with(|| {
			// Substrate transfer
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				1 * GLMR,
			));
			// 1 WEI is left in the account
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1 * WEI);
		});
}

#[test]
fn transfer_ed_0_evm() {
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(ALICE),
				((1 * GLMR) + (21_000 * BASE_FEE_GENESIS)) + (1 * WEI),
			),
			(AccountId::from(BOB), 0),
		])
		.build()
		.execute_with(|| {
			// EVM transfer
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: Some(BASE_FEE_GENESIS.into()),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// 1 WEI is left in the account
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1 * WEI,);
		});
}

#[test]
fn refund_ed_0_evm() {
	ExtBuilder::default()
		.with_balances(vec![
			(
				AccountId::from(ALICE),
				((1 * GLMR) + (21_777 * BASE_FEE_GENESIS)),
			),
			(AccountId::from(BOB), 0),
		])
		.build()
		.execute_with(|| {
			// EVM transfer that zeroes ALICE
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(ALICE),
				target: H160::from(BOB),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_777u64,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: Some(BASE_FEE_GENESIS.into()),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// ALICE is refunded
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				777 * BASE_FEE_GENESIS,
			);
		});
}

#[test]
fn author_does_not_receive_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * GLMR) + (21_000 * (500 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			// Some block author as seen by pallet-evm.
			let author = AccountId::from(<pallet_evm::Pallet<Runtime>>::find_author());
			// Currently the default impl of the evm uses `deposit_into_existing`.
			// If we were to use this implementation, and for an author to receive eventual tips,
			// the account needs to be somehow initialized, otherwise the deposit would fail.
			Balances::make_free_balance_be(&author, 100 * GLMR);

			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(300 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(200 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
			// Author free balance didn't change.
			assert_eq!(Balances::free_balance(author), 100 * GLMR,);
		});
}

#[test]
fn total_issuance_after_evm_transaction_with_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * GLMR) + (21_000 * (200 * GIGAWEI)),
		)])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: U256::from(200 * GIGAWEI),
				max_priority_fee_per_gas: Some(U256::from(100 * GIGAWEI)),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// Fee is 100 GWEI base fee + 100 GWEI tip.
			let fee = ((200 * GIGAWEI) * 21_000) as f64;
			// 80% was burned.
			let expected_burn = (fee * 0.8) as u128;
			assert_eq!(issuance_after, issuance_before - expected_burn,);
			// 20% was sent to treasury.
			let expected_treasury = (fee * 0.2) as u128;
			assert_eq!(moonbeam_runtime::Treasury::pot(), expected_treasury);
		});
}

#[test]
fn total_issuance_after_evm_transaction_without_priority_fee() {
	ExtBuilder::default()
		.with_balances(vec![(
			AccountId::from(BOB),
			(1 * GLMR) + (21_000 * BASE_FEE_GENESIS),
		)])
		.build()
		.execute_with(|| {
			let issuance_before = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// EVM transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::<Runtime>::call {
				source: H160::from(BOB),
				target: H160::from(ALICE),
				input: Vec::new(),
				value: (1 * GLMR).into(),
				gas_limit: 21_000u64,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: Some(BASE_FEE_GENESIS.into()),
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let issuance_after = <Runtime as pallet_evm::Config>::Currency::total_issuance();
			// Fee is 100 GWEI base fee.
			let fee = (BASE_FEE_GENESIS * 21_000) as f64;
			// 80% was burned.
			let expected_burn = (fee * 0.8) as u128;
			assert_eq!(issuance_after, issuance_before - expected_burn,);
			// 20% was sent to treasury.
			let expected_treasury = (fee * 0.2) as u128;
			assert_eq!(moonbeam_runtime::Treasury::pot(), expected_treasury);
		});
}

#[test]
fn root_can_change_default_xcm_vers() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			};
			let source_id: moonbeam_runtime::AssetId = source_location.clone().into();
			// Default XCM version is not set yet, so xtokens should fail because it does not
			// know with which version to send
			assert_noop!(
				XTokens::transfer(
					origin_of(AccountId::from(ALICE)),
					CurrencyId::ForeignAsset(source_id),
					100_000_000_000_000,
					Box::new(xcm::VersionedMultiLocation::V3(dest.clone())),
					WeightLimit::Limited(4000000000.into())
				),
				orml_xtokens::Error::<Runtime>::XcmExecutionFailed
			);

			// Root sets the defaultXcm
			assert_ok!(PolkadotXcm::force_default_xcm_version(
				root_origin(),
				Some(2)
			));

			// Now transferring does not fail
			assert_ok!(XTokens::transfer(
				origin_of(AccountId::from(ALICE)),
				CurrencyId::ForeignAsset(source_id),
				100_000_000_000_000,
				Box::new(xcm::VersionedMultiLocation::V3(dest)),
				WeightLimit::Limited(4000000000.into())
			));
		})
}

#[test]
fn asset_can_be_registered() {
	ExtBuilder::default().build().execute_with(|| {
		let source_location = AssetType::Xcm(MultiLocation::parent());
		let source_id: moonbeam_runtime::AssetId = source_location.clone().into();
		let asset_metadata = AssetRegistrarMetadata {
			name: b"RelayToken".to_vec(),
			symbol: b"Relay".to_vec(),
			decimals: 12,
			is_frozen: false,
		};
		assert_ok!(AssetManager::register_foreign_asset(
			moonbeam_runtime::RuntimeOrigin::root(),
			source_location,
			asset_metadata,
			1u128,
			true
		));
		assert!(AssetManager::asset_id_type(source_id).is_some());
	});
}

#[test]
fn local_assets_cannot_be_create_by_signed_origins() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR * SUPPLY_FACTOR),
			(AccountId::from(BOB), 1_000 * GLMR * SUPPLY_FACTOR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				RuntimeCall::LocalAssets(
					pallet_assets::Call::<Runtime, LocalAssetInstance>::create {
						id: 11u128.into(),
						admin: AccountId::from(ALICE),
						min_balance: 1u128
					}
				)
				.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		});
}

#[test]
fn asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			// Assert the asset has been created with the correct supply
			assert_eq!(LocalAssets::total_supply(0u128), 1_000 * GLMR);

			// Convert the assetId to its corresponding precompile address
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Access totalSupply through precompile.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::total_supply {},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000 * GLMR));

			// Access balanceOf through precompile
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::balance_of {
						who: Address(ALICE.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000 * GLMR));
		});
}

#[test]
fn asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Transfer tokens from Aice to Bob, 400 GLMR.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::transfer {
						to: Address(BOB.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(23497)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Make sure BOB has 400 GLMR
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					LocalAssetsPCall::balance_of {
						who: Address(BOB.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400 * GLMR));
		});
}

#[test]
fn asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Aprove Bob for spending 400 GLMR from Alice
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::approve {
						spender: Address(BOB.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(13862)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_APPROVAL,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Transfer tokens from Alice to Charlie by using BOB as origin
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					LocalAssetsPCall::transfer_from {
						from: Address(ALICE.into()),
						to: Address(CHARLIE.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(29021)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(CHARLIE),
					EvmDataWriter::new().write(U256::from(400 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Make sure CHARLIE has 400 GLMR
			Precompiles::new()
				.prepare_test(
					CHARLIE,
					asset_precompile_address,
					LocalAssetsPCall::balance_of {
						who: Address(CHARLIE.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400 * GLMR));
		});
}

#[test]
fn asset_erc20_precompiles_mint_burn() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Mint 1000 MOVRS to BOB
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::mint {
						to: Address(BOB.into()),
						value: { 1000 * GLMR }.into(),
					},
				)
				.expect_cost(12787)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::default(),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(1000 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Assert the asset has been minted
			assert_eq!(LocalAssets::total_supply(0u128), 2_000 * GLMR);
			assert_eq!(
				LocalAssets::balance(0u128, AccountId::from(BOB)),
				1_000 * GLMR
			);

			// Burn tokens
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::burn {
						from: Address(BOB.into()),
						value: { 500 * GLMR }.into(),
					},
				)
				.expect_cost(13016)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(BOB),
					H160::default(),
					EvmDataWriter::new().write(U256::from(500 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Assert the asset has been burnt
			assert_eq!(LocalAssets::total_supply(0u128), 1_500 * GLMR);
			assert_eq!(
				LocalAssets::balance(0u128, AccountId::from(BOB)),
				500 * GLMR
			);
		});
}

#[test]
fn asset_erc20_precompiles_freeze_thaw_account() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Freeze account
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::freeze {
						account: Address(ALICE.into()),
					},
				)
				.expect_cost(6699)
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Assert account is frozen
			assert_eq!(
				LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1).into_result(),
				Err(TokenError::Frozen.into())
			);

			// Thaw Account
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::thaw {
						account: Address(ALICE.into()),
					},
				)
				.expect_cost(6713)
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Assert account is not frozen
			assert!(LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1)
				.into_result()
				.is_ok());
		});
}

#[test]
fn asset_erc20_precompiles_freeze_thaw_asset() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Freeze Asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::freeze_asset {},
				)
				.expect_cost(5548)
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Assert account is frozen
			assert_eq!(
				LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1).into_result(),
				Err(TokenError::Frozen.into())
			);

			// Thaw Asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::thaw_asset {},
				)
				.expect_cost(5550)
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Assert account is not frozen
			assert!(LocalAssets::can_withdraw(0u128, &AccountId::from(ALICE), 1)
				.into_result()
				.is_ok());
		});
}

#[test]
fn asset_erc20_precompiles_freeze_transfer_ownership() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Transfer ownerhsip of an asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::transfer_ownership {
						owner: Address(BOB.into()),
					},
				)
				.expect_cost(6614)
				.expect_no_logs()
				.execute_returns_encoded(true);

			// No clear way of testing BOB is new owner, other than doing a priviledged function
			// e.g., transfer_ownership again
			assert_ok!(LocalAssets::transfer_ownership(
				origin_of(AccountId::from(BOB)),
				0u128.into(),
				AccountId::from(ALICE)
			));
		});
}

#[test]
fn asset_erc20_precompiles_freeze_set_team() {
	ExtBuilder::default()
		.with_local_assets(vec![(
			0u128,
			vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			let asset_precompile_address =
				Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, 0u128);

			// Set Bob as issuer, admin and freezer
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::set_team {
						admin: Address(BOB.into()),
						freezer: Address(BOB.into()),
						issuer: Address(BOB.into()),
					},
				)
				.expect_cost(5577)
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Bob should be able to mint, freeze, and thaw
			assert_ok!(LocalAssets::mint(
				origin_of(AccountId::from(BOB)),
				0u128.into(),
				AccountId::from(BOB),
				1_000 * GLMR
			));
			assert_ok!(LocalAssets::freeze(
				origin_of(AccountId::from(BOB)),
				0u128.into(),
				AccountId::from(ALICE)
			));
			assert_ok!(LocalAssets::thaw(
				origin_of(AccountId::from(BOB)),
				0u128.into(),
				AccountId::from(ALICE)
			));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_supply_and_balance() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Assert the asset has been created with the correct supply
			assert_eq!(Assets::total_supply(relay_asset_id), 1_000 * GLMR);

			// Access totalSupply through precompile. Important that the context is correct
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::total_supply {},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000 * GLMR));

			// Access balanceOf through precompile
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::balance_of {
						who: Address(ALICE.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000 * GLMR));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Transfer tokens from Aice to Bob, 400 GLMR.
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::transfer {
						to: Address(BOB.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(23497)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Make sure BOB has 400 GLMR
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					LocalAssetsPCall::balance_of {
						who: Address(BOB.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400 * GLMR));
		});
}

#[test]
fn xcm_asset_erc20_precompiles_approve() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000 * GLMR)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.build()
		.execute_with(|| {
			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Aprove Bob for spending 400 GLMR from Alice
			Precompiles::new()
				.prepare_test(
					ALICE,
					asset_precompile_address,
					LocalAssetsPCall::approve {
						spender: Address(BOB.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(13862)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_APPROVAL,
					H160::from(ALICE),
					H160::from(BOB),
					EvmDataWriter::new().write(U256::from(400 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Transfer tokens from Alice to Charlie by using BOB as origin
			Precompiles::new()
				.prepare_test(
					BOB,
					asset_precompile_address,
					LocalAssetsPCall::transfer_from {
						from: Address(ALICE.into()),
						to: Address(CHARLIE.into()),
						value: { 400 * GLMR }.into(),
					},
				)
				.expect_cost(29021)
				.expect_log(log3(
					asset_precompile_address,
					SELECTOR_LOG_TRANSFER,
					H160::from(ALICE),
					H160::from(CHARLIE),
					EvmDataWriter::new().write(U256::from(400 * GLMR)).build(),
				))
				.execute_returns_encoded(true);

			// Make sure CHARLIE has 400 GLMR
			Precompiles::new()
				.prepare_test(
					CHARLIE,
					asset_precompile_address,
					LocalAssetsPCall::balance_of {
						who: Address(CHARLIE.into()),
					},
				)
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400 * GLMR));
		});
}

#[test]
fn xtokens_precompile_transfer() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// We have the assetId that corresponds to the relay chain registered
			let relay_asset_id: moonbeam_runtime::AssetId =
				AssetType::Xcm(MultiLocation::parent()).into();

			// Its address is
			let asset_precompile_address = Runtime::asset_id_to_account(
				FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
				relay_asset_id,
			);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			);

			// We use the address of the asset as an identifier of the asset we want to transfer
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					XtokensPCall::transfer {
						currency_address: Address(asset_precompile_address.into()),
						amount: 500_000_000_000_000u128.into(),
						destination: destination.clone(),
						weight: 4_000_000,
					},
				)
				.expect_cost(24000)
				.expect_no_logs()
				.execute_returns(vec![])
		})
}

#[test]
fn xtokens_precompile_transfer_multiasset() {
	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let xtokens_precompile_address = H160::from_low_u64_be(2052);

			// Alice has 1000 tokens. She should be able to send through precompile
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			);

			// This time we transfer it through TransferMultiAsset
			// Instead of the address, we encode directly the multilocation referencing the asset
			Precompiles::new()
				.prepare_test(
					ALICE,
					xtokens_precompile_address,
					XtokensPCall::transfer_multiasset {
						// We want to transfer the relay token
						asset: MultiLocation::parent(),
						amount: 500_000_000_000_000u128.into(),
						destination: destination.clone(),
						weight: 4_000_000,
					},
				)
				.expect_cost(24000)
				.expect_no_logs()
				.execute_returns(vec![]);
		})
}

#[test]
fn make_sure_glmr_can_be_transferred_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			};
			assert_ok!(XTokens::transfer_multiasset(
				origin_of(AccountId::from(ALICE)),
				Box::new(VersionedMultiAsset::V3(MultiAsset {
					id: Concrete(moonbeam_runtime::xcm_config::SelfReserve::get()),
					fun: Fungible(1000)
				})),
				Box::new(VersionedMultiLocation::V3(dest)),
				WeightLimit::Limited(40000.into())
			));
		});
}

#[test]
fn make_sure_glmr_can_be_transferred() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			};
			assert_ok!(XTokens::transfer(
				origin_of(AccountId::from(ALICE)),
				CurrencyId::SelfReserve,
				100,
				Box::new(VersionedMultiLocation::V3(dest)),
				WeightLimit::Limited(40000.into())
			));
		});
}

#[test]
fn make_sure_polkadot_xcm_cannot_be_called() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			};
			let multiassets: MultiAssets = [MultiAsset {
				id: Concrete(moonbeam_runtime::xcm_config::SelfLocation::get()),
				fun: Fungible(1000),
			}]
			.to_vec()
			.into();
			assert_noop!(
				RuntimeCall::PolkadotXcm(pallet_xcm::Call::<Runtime>::reserve_transfer_assets {
					dest: Box::new(VersionedMultiLocation::V3(dest.clone())),
					beneficiary: Box::new(VersionedMultiLocation::V3(dest)),
					assets: Box::new(VersionedMultiAssets::V3(multiassets)),
					fee_asset_item: 0,
				})
				.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::signed(
					AccountId::from(ALICE)
				)),
				frame_system::Error::<Runtime>::CallFiltered
			);
		});
}

#[test]
fn transact_through_signed_precompile_works_v2() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			// Destination
			let dest = MultiLocation::parent();

			let fee_payer_asset = MultiLocation::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			let xcm_transactor_v2_precompile_address = H160::from_low_u64_be(2061);

			Precompiles::new()
				.prepare_test(
					ALICE,
					xcm_transactor_v2_precompile_address,
					XcmTransactorV2PCall::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.expect_cost(19078)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn transact_through_signed_cannot_send_to_local_chain() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(2)
		.build()
		.execute_with(|| {
			// Destination
			let dest = MultiLocation::here();

			let fee_payer_asset = MultiLocation::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			let xcm_transactor_v2_precompile_address = H160::from_low_u64_be(2061);

			Precompiles::new()
				.prepare_test(
					ALICE,
					xcm_transactor_v2_precompile_address,
					XcmTransactorV2PCall::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error:")
						&& from_utf8(&output).unwrap().contains("ErrorValidating")
				});
		});
}

#[test]
fn transactor_cannot_use_more_than_max_weight() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let source_id: moonbeam_runtime::AssetId = source_location.clone().into();
			assert_ok!(XcmTransactor::register(
				root_origin(),
				AccountId::from(ALICE),
				0,
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V3(MultiLocation::parent())),
				// Relay charges 1000 for every instruction, and we have 3, so 3000
				3000.into(),
				20000.into(),
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				root_origin(),
				Box::new(xcm::VersionedMultiLocation::V3(MultiLocation::parent())),
				1,
			));

			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonbeam_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V3(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![],
					// 2000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001.into(),
						overall_weight: None
					}
				),
				pallet_xcm_transactor::Error::<Runtime>::MaxWeightTransactReached
			);
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					origin_of(AccountId::from(ALICE)),
					moonbeam_runtime::xcm_config::Transactors::Relay,
					0,
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::ForeignAsset(source_id)),
						fee_amount: None
					},
					vec![],
					// 20000 is the max
					TransactWeights {
						transact_required_weight_at_most: 17001.into(),
						overall_weight: None
					}
				),
				pallet_xcm_transactor::Error::<Runtime>::MaxWeightTransactReached
			);
		})
}

#[test]
fn call_xtokens_with_fee() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_safe_xcm_version(2)
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_type: AssetType::Xcm(MultiLocation::parent()),
			metadata: AssetRegistrarMetadata {
				name: b"RelayToken".to_vec(),
				symbol: b"Relay".to_vec(),
				decimals: 12,
				is_frozen: false,
			},
			balances: vec![(AccountId::from(ALICE), 1_000_000_000_000_000)],
			is_sufficient: true,
		}])
		.build()
		.execute_with(|| {
			let source_location = AssetType::Xcm(MultiLocation::parent());
			let dest = MultiLocation {
				parents: 1,
				interior: X1(AccountId32 {
					network: None,
					id: [1u8; 32],
				}),
			};
			let source_id: moonbeam_runtime::AssetId = source_location.clone().into();

			let before_balance = Assets::balance(source_id, &AccountId::from(ALICE));

			// We are able to transfer with fee
			assert_ok!(XTokens::transfer_with_fee(
				origin_of(AccountId::from(ALICE)),
				CurrencyId::ForeignAsset(source_id),
				100_000_000_000_000,
				100,
				Box::new(xcm::VersionedMultiLocation::V3(dest.clone())),
				WeightLimit::Limited(4000000000.into())
			));

			let after_balance = Assets::balance(source_id, &AccountId::from(ALICE));
			// At least these much (plus fees) should have been charged
			assert_eq!(before_balance - 100_000_000_000_000 - 100, after_balance);
		});
}

#[test]
fn test_xcm_utils_ml_to_account() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let expected_address_parent: H160 =
			ParentIsPreset::<AccountId>::convert_ref(MultiLocation::parent())
				.unwrap()
				.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					multilocation: MultiLocation::parent(),
				},
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address_parent))
					.build(),
			);

		let parachain_2000_multilocation = MultiLocation::new(1, X1(Parachain(2000)));
		let expected_address_parachain: H160 =
			SiblingParachainConvertsVia::<Sibling, AccountId>::convert_ref(
				parachain_2000_multilocation.clone(),
			)
			.unwrap()
			.into();

		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					multilocation: parachain_2000_multilocation,
				},
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address_parachain))
					.build(),
			);

		let alice_in_parachain_2000_multilocation = MultiLocation::new(
			1,
			X2(
				Parachain(2000),
				AccountKey20 {
					network: None,
					key: ALICE,
				},
			),
		);

		// this should fail, this convertor is not allowed in moonriver
		Precompiles::new()
			.prepare_test(
				ALICE,
				xcm_utils_precompile_address,
				XcmUtilsPCall::multilocation_to_address {
					multilocation: alice_in_parachain_2000_multilocation,
				},
			)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_reverts(|output| output == b"multilocation: Failed multilocation conversion");
	});
}

#[test]
fn test_xcm_utils_weight_message() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let expected_weight = UnitWeightCost::get().ref_time();

		let message: Vec<u8> = xcm::VersionedXcm::<()>::V3(Xcm(vec![ClearOrigin])).encode();

		let input = XcmUtilsPCall::weight_message {
			message: message.into(),
		};

		Precompiles::new()
			.prepare_test(ALICE, xcm_utils_precompile_address, input)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(expected_weight).build());
	});
}

#[test]
fn test_xcm_utils_get_units_per_second() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_utils_precompile_address = H160::from_low_u64_be(2060);
		let multilocation = SelfReserve::get();

		let input = XcmUtilsPCall::get_units_per_second { multilocation };

		let expected_units =
			WEIGHT_REF_TIME_PER_SECOND as u128 * moonbeam_runtime::currency::WEIGHT_FEE;

		Precompiles::new()
			.prepare_test(ALICE, xcm_utils_precompile_address, input)
			.expect_cost(1000)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(expected_units).build());
	});
}

#[test]
fn precompile_existence() {
	ExtBuilder::default().build().execute_with(|| {
		let precompiles = Precompiles::new();
		let precompile_addresses: std::collections::BTreeSet<_> = vec![
			1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 2048, 2049, 2050, 2051, 2052, 2053, 2054,
			2055, 2056, 2057, 2058, 2059, 2060, 2061, 2062, 2063, 2064, 2067, 2069,
		]
		.into_iter()
		.map(H160::from_low_u64_be)
		.collect();

		for i in 0..3000 {
			let address = H160::from_low_u64_be(i);

			if precompile_addresses.contains(&address) {
				assert!(
					precompiles.is_precompile(address),
					"is_precompile({}) should return true",
					i
				);

				assert!(
					precompiles
						.execute(&mut MockHandle::new(
							address,
							Context {
								address,
								caller: H160::zero(),
								apparent_value: U256::zero()
							}
						),)
						.is_some(),
					"execute({},..) should return Some(_)",
					i
				);
			} else {
				assert!(
					!precompiles.is_precompile(address),
					"is_precompile({}) should return false",
					i
				);

				assert!(
					precompiles
						.execute(&mut MockHandle::new(
							address,
							Context {
								address,
								caller: H160::zero(),
								apparent_value: U256::zero()
							}
						),)
						.is_none(),
					"execute({},..) should return None",
					i
				);
			}
		}
	});
}

#[test]
fn removed_precompiles() {
	ExtBuilder::default().build().execute_with(|| {
		let precompiles = Precompiles::new();
		let removed_precompiles = [1025];

		for i in 1..3000 {
			let address = H160::from_low_u64_be(i);

			if !precompiles.is_precompile(address) {
				continue;
			}

			if !removed_precompiles.contains(&i) {
				assert!(
					precompiles.is_active_precompile(address),
					"{i} should be an active precompile"
				);
				continue;
			}

			assert!(
				!precompiles.is_active_precompile(address),
				"{i} shouldn't be an active precompile"
			);

			precompiles
				.prepare_test(Alice, address, [])
				.execute_reverts(|out| out == b"Removed precompile");
		}
	})
}

#[test]
fn evm_revert_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			// Batch a transfer followed by an invalid call to batch.
			// Thus BatchAll will revert the transfer.
			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,

				input: BatchPCall::batch_all {
					to: vec![Address(BOB.into()), Address(batch_precompile_address)].into(),
					value: vec![U256::from(1 * GLMR), U256::zero()].into(),
					call_data: vec![].into(),
					gas_limit: vec![].into()
				}
				.into(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let transfer_count = System::events()
				.iter()
				.filter(|r| match r.event {
					RuntimeEvent::Balances(pallet_balances::Event::Transfer { .. }) => true,
					_ => false,
				})
				.count();

			assert_eq!(transfer_count, 0, "there should be no transfer event");
		});
}

#[test]
fn evm_success_keeps_substrate_events() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 100_000 * GLMR)])
		.build()
		.execute_with(|| {
			let batch_precompile_address = H160::from_low_u64_be(2056);

			assert_ok!(RuntimeCall::EVM(pallet_evm::Call::call {
				source: ALICE.into(),
				target: batch_precompile_address,
				input: BatchPCall::batch_all {
					to: vec![Address(BOB.into())].into(),
					value: vec![U256::from(1 * GLMR)].into(),
					call_data: vec![].into(),
					gas_limit: vec![].into()
				}
				.into(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: 500_000,
				max_fee_per_gas: BASE_FEE_GENESIS.into(),
				max_priority_fee_per_gas: None,
				nonce: Some(U256::from(0)),
				access_list: Vec::new(),
			})
			.dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));

			let transfer_count = System::events()
				.iter()
				.filter(|r| match r.event {
					RuntimeEvent::Balances(pallet_balances::Event::Transfer { .. }) => true,
					_ => false,
				})
				.count();

			assert_eq!(transfer_count, 1, "there should be 1 transfer event");
		});
}

#[cfg(test)]
mod fee_tests {
	use super::*;
	use frame_support::{
		traits::ConstU128,
		weights::{ConstantMultiplier, WeightToFee},
	};
	use moonbeam_runtime::{
		currency, LengthToFee, MinimumMultiplier, RuntimeBlockWeights, SlowAdjustingFeeUpdate,
		TargetBlockFullness, TransactionPayment,
	};
	use sp_core::Get;
	use sp_runtime::FixedPointNumber;

	fn run_with_system_weight<F>(w: Weight, mut assertions: F)
	where
		F: FnMut() -> (),
	{
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	#[test]
	fn test_multiplier_can_grow_from_zero() {
		let minimum_multiplier = MinimumMultiplier::get();
		let target = TargetBlockFullness::get()
			* RuntimeBlockWeights::get()
				.get(DispatchClass::Normal)
				.max_total
				.unwrap();
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target * 101 / 100, || {
			let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
			assert!(
				next > minimum_multiplier,
				"{:?} !>= {:?}",
				next,
				minimum_multiplier
			);
		})
	}

	#[test]
	fn test_fee_calculation() {
		let base_extrinsic = RuntimeBlockWeights::get()
			.get(DispatchClass::Normal)
			.base_extrinsic;
		let multiplier = sp_runtime::FixedU128::from_float(0.999000000000000000);
		let extrinsic_len = 100u32;
		let extrinsic_weight = 5_000u64;
		let tip = 42u128;
		type WeightToFeeImpl = ConstantMultiplier<u128, ConstU128<{ currency::WEIGHT_FEE }>>;
		type LengthToFeeImpl = LengthToFee;

		// base_fee + (multiplier * extrinsic_weight_fee) + extrinsic_length_fee + tip
		let expected_fee = WeightToFeeImpl::weight_to_fee(&base_extrinsic)
			+ multiplier.saturating_mul_int(WeightToFeeImpl::weight_to_fee(
				&Weight::from_ref_time(extrinsic_weight),
			)) + LengthToFeeImpl::weight_to_fee(&Weight::from_ref_time(
			extrinsic_len as u64,
		)) + tip;

		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into();
		t.execute_with(|| {
			pallet_transaction_payment::NextFeeMultiplier::<Runtime>::set(multiplier);
			let actual_fee = TransactionPayment::compute_fee(
				extrinsic_len,
				&frame_support::dispatch::DispatchInfo {
					class: DispatchClass::Normal,
					pays_fee: frame_support::dispatch::Pays::Yes,
					weight: Weight::from_ref_time(extrinsic_weight),
				},
				tip,
			);

			assert_eq!(
				expected_fee,
				actual_fee,
				"The actual fee did not match the expected fee, diff {}",
				actual_fee - expected_fee
			);
		});
	}
}
