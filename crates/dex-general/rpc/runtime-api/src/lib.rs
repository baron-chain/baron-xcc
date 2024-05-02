// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use dex_general::{AssetBalance, PairInfo};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
     pub trait DexGeneralApi<AccountId, AssetId>
     where
        AccountId: Codec,
        AssetBalance: Codec,
        AssetId: Codec
     {
        fn get_pair_by_asset_id(
            asset_0: AssetId,
            asset_1: AssetId
        ) -> Option<PairInfo<AccountId, AssetBalance, AssetId>>;

        // buy amount asset price
        fn get_amount_in_price(supply: AssetBalance, path: Vec<AssetId>) -> AssetBalance;

        // sell amount asset price
        fn get_amount_out_price(supply: AssetBalance, path: Vec<AssetId>) -> AssetBalance;

        fn get_estimate_lptoken(
            asset_0: AssetId,
            asset_1: AssetId,
            amount_0_desired: AssetBalance,
            amount_1_desired: AssetBalance,
            amount_0_min: AssetBalance,
            amount_1_min: AssetBalance,
        ) -> AssetBalance;

        fn calculate_remove_liquidity(
            asset_0: AssetId,
            asset_1: AssetId,
            amount: AssetBalance,
        ) -> Option<(AssetBalance, AssetBalance)>;
    }
}
