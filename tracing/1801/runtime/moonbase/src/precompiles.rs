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

use crate::{
	asset_config::{ForeignAssetInstance, LocalAssetInstance},
	xcm_config::XcmExecutorConfig,
	CouncilInstance, TechCommitteeInstance, TreasuryCouncilInstance,
};
use frame_support::parameter_types;
use moonbeam_relay_encoder::westend::WestendEncoder;
use pallet_evm_precompile_author_mapping::AuthorMappingWrapper;
use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_batch::BatchPrecompile;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_call_permit::CallPermitPrecompile;
use pallet_evm_precompile_collective::CollectivePrecompile;
use pallet_evm_precompile_crowdloan_rewards::CrowdloanRewardsWrapper;
use pallet_evm_precompile_democracy::DemocracyWrapper;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_parachain_staking::ParachainStakingWrapper;
use pallet_evm_precompile_proxy::ProxyWrapper;
use pallet_evm_precompile_randomness::RandomnessWrapper;
use pallet_evm_precompile_relay_encoder::RelayEncoderWrapper;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_xcm_transactor::{
	v1::XcmTransactorWrapperV1, v2::XcmTransactorWrapperV2,
};
use pallet_evm_precompile_xcm_utils::XcmUtilsWrapper;
use pallet_evm_precompile_xtokens::XtokensWrapper;
use pallet_evm_precompileset_assets_erc20::{Erc20AssetsPrecompileSet, IsForeign, IsLocal};
use precompile_utils::precompile_set::*;

/// ERC20 metadata for the native token.
pub struct NativeErc20Metadata;

impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"DEV token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"DEV"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		18
	}

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool {
		true
	}
}

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as foreign
pub const FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];
/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as local
pub const LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8, 255u8, 255u8, 254u8];

parameter_types! {
	pub ForeignAssetPrefix: &'static [u8] = FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX;
	pub LocalAssetPrefix: &'static [u8] = LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX;
}

/// The PrecompileSet installed in the Moonbase runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Moonbeam specific
/// 2048-4095 Moonbeam specific precompiles
pub type MoonbasePrecompiles<R> = PrecompileSetBuilder<
	R,
	(
		// Skip precompiles if out of range.
		PrecompilesInRangeInclusive<
			(AddressU64<1>, AddressU64<4095>),
			(
				// Ethereum precompiles:
				// We allow DELEGATECALL to stay compliant with Ethereum behavior.
				PrecompileAt<AddressU64<1>, ECRecover, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<2>, Sha256, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<3>, Ripemd160, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<4>, Identity, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<5>, Modexp, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<6>, Bn128Add, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<7>, Bn128Mul, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<8>, Bn128Pairing, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<9>, Blake2F, ForbidRecursion, AllowDelegateCall>,
				// Non-Moonbeam specific nor Ethereum precompiles :
				PrecompileAt<AddressU64<1024>, Sha3FIPS256>,
				// PrecompileAt<AddressU64<1025>, Dispatch<R>>,
				PrecompileAt<AddressU64<1026>, ECRecoverPublicKey>,
				// Moonbeam specific precompiles:
				PrecompileAt<AddressU64<2048>, ParachainStakingWrapper<R>>,
				PrecompileAt<AddressU64<2049>, CrowdloanRewardsWrapper<R>>,
				PrecompileAt<AddressU64<2050>, Erc20BalancesPrecompile<R, NativeErc20Metadata>>,
				PrecompileAt<AddressU64<2051>, DemocracyWrapper<R>>,
				PrecompileAt<AddressU64<2052>, XtokensWrapper<R>>,
				PrecompileAt<AddressU64<2053>, RelayEncoderWrapper<R, WestendEncoder>>,
				PrecompileAt<AddressU64<2054>, XcmTransactorWrapperV1<R>>,
				PrecompileAt<AddressU64<2055>, AuthorMappingWrapper<R>>,
				PrecompileAt<AddressU64<2056>, BatchPrecompile<R>, LimitRecursionTo<2>>,
				PrecompileAt<AddressU64<2057>, RandomnessWrapper<R>>,
				PrecompileAt<AddressU64<2058>, CallPermitPrecompile<R>>,
				PrecompileAt<AddressU64<2059>, ProxyWrapper<R>>,
				PrecompileAt<AddressU64<2060>, XcmUtilsWrapper<R, XcmExecutorConfig>>,
				PrecompileAt<AddressU64<2061>, XcmTransactorWrapperV2<R>>,
				PrecompileAt<AddressU64<2062>, CollectivePrecompile<R, CouncilInstance>>,
				PrecompileAt<AddressU64<2063>, CollectivePrecompile<R, TechCommitteeInstance>>,
				PrecompileAt<AddressU64<2064>, CollectivePrecompile<R, TreasuryCouncilInstance>>,
			),
		>,
		// Prefixed precompile sets (XC20)
		PrecompileSetStartingWith<
			ForeignAssetPrefix,
			Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>,
		>,
		PrecompileSetStartingWith<
			LocalAssetPrefix,
			Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>,
		>,
	),
>;
