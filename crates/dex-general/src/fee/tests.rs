// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

use super::mock::*;
use crate::{PairMetadata, PairStatus, DEFAULT_FEE_RATE, FEE_ADJUSTMENT};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_core::U256;

const DOT_ASSET_ID: CurrencyId = CurrencyId::Token(2);
const BTC_ASSET_ID: CurrencyId = CurrencyId::Token(3);

const LP_DOT_BTC: CurrencyId = CurrencyId::LpToken(2, 3);

const ALICE: u128 = 1;
const BOB: u128 = 2;
const CHARLIE: u128 = 3;
const DOT_UNIT: u128 = 1000_000_000_000_000;
const BTC_UNIT: u128 = 1000_000_00;

const MAX_BALANCE: u128 = u128::MAX - 1;

#[test]
fn fee_meta_getter_should_work() {
    new_test_ext().execute_with(|| {
        let (fee_receiver, fee_point) = DexPallet::fee_meta();

        assert_eq!(fee_receiver, None);
        assert_eq!(fee_point, 5);
    })
}

#[test]
fn fee_meta_setter_should_not_work() {
    new_test_ext().execute_with(|| {
        let (fee_receiver, fee_point) = DexPallet::fee_meta();

        assert_eq!(fee_receiver, None);
        assert_eq!(fee_point, 5);

        assert_noop!(
            DexPallet::set_fee_receiver(RawOrigin::Signed(BOB).into(), Some(BOB)),
            BadOrigin,
        );

        assert_noop!(DexPallet::set_fee_point(RawOrigin::Signed(BOB).into(), 0), BadOrigin);
    })
}

#[test]
fn create_pair_should_set_fee_rate() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            12,
        ));
        let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);

        assert!(matches!(
            DexPallet::pair_status(sorted_pair),
            PairStatus::Trading(PairMetadata { fee_rate: 12, .. })
        ));
    });
}

#[test]
fn turn_on_protocol_fee_only_add_liquidity_no_fee_should_work() {
    new_test_ext().execute_with(|| {
        // 1. turn on the protocol fee
        // use default rate: 1/6
        assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));

        let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);
        assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            MAX_BALANCE
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            MAX_BALANCE
        ));

        let total_supply_dot: u128 = 1 * DOT_UNIT;
        let total_supply_btc: u128 = 1 * BTC_UNIT;

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        // 2. first add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let lp_of_alice_0 = 316227766016;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(DOT_UNIT) * U256::from(BTC_UNIT)
        );

        let total_supply_dot = 50 * DOT_UNIT;
        let total_supply_btc = 50 * BTC_UNIT;

        // 3. second add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            BTC_ASSET_ID,
            DOT_ASSET_ID,
            total_supply_btc,
            total_supply_dot,
            0,
            0,
            100
        ));

        let lp_of_alice_1 = 16127616066816u128;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_1
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(51 * DOT_UNIT) * U256::from(51 * BTC_UNIT)
        );

        let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
        let balance_dot = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let balance_btc = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

        assert_eq!(balance_dot, 51000000000000000);
        assert_eq!(balance_btc, 5100000000);
        assert_eq!((balance_dot / DOT_UNIT), (balance_btc / BTC_UNIT));

        // 4. third add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            BTC_ASSET_ID,
            DOT_ASSET_ID,
            total_supply_btc,
            total_supply_dot,
            0,
            0,
            100
        ));

        let lp_total = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
        let lp_of_alice_2 = 31939004367616u128;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_2
        );
        let lp_of_bob = 0u128;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB),
            lp_of_bob
        );
        assert_eq!(lp_total, lp_of_alice_2 + lp_of_bob);

        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(101 * DOT_UNIT) * U256::from(101 * BTC_UNIT)
        );
    });
}

#[test]
fn turn_on_protocol_fee_remove_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        // 1. turn on the protocol fee
        // use default rate: 1/6
        assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));

        let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);
        assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            10000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            10000 * BTC_UNIT
        ));

        let total_supply_dot: u128 = 1 * DOT_UNIT;
        let total_supply_btc: u128 = 1 * BTC_UNIT;

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        // 2. first add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let lp_of_alice_0 = 316227766016;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(DOT_UNIT) * U256::from(BTC_UNIT)
        );

        let total_supply_dot = 50 * DOT_UNIT;
        let total_supply_btc = 50 * BTC_UNIT;

        // 3. second add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            BTC_ASSET_ID,
            DOT_ASSET_ID,
            total_supply_btc,
            total_supply_dot,
            0,
            0,
            100
        ));

        let lp_of_alice_1 = 16127616066816u128;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_1
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(51 * DOT_UNIT) * U256::from(51 * BTC_UNIT)
        );

        let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
        let balance_dot = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let balance_btc = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

        assert_eq!(balance_dot, 51000000000000000);
        assert_eq!(balance_btc, 5100000000);
        assert_eq!((balance_dot / DOT_UNIT), (balance_btc / BTC_UNIT));

        // 4. remove_liquidity
        assert_ok!(DexPallet::remove_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            lp_of_alice_0,
            0u128,
            0u128,
            ALICE,
            100
        ));

        let lp_total = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_1 - lp_of_alice_0
        );
        let lp_of_bob = 0u128;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB),
            lp_of_bob
        );
        assert_eq!(lp_total, lp_of_alice_1 - lp_of_alice_0 + lp_of_bob);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(50 * DOT_UNIT) * U256::from(50 * BTC_UNIT)
        );
    });
}

#[test]
fn turn_on_protocol_fee_swap_have_fee_should_work() {
    new_test_ext().execute_with(|| {
        // 1. turn on the protocol fee
        assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
        // use default rate: 1/(1/6)-1=5
        assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), 5u8));

        let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);
        assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            DOT_UNIT * 1000
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            BTC_UNIT * 1000
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            DOT_UNIT * 1000
        ));

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        let total_supply_dot: u128 = 1 * DOT_UNIT;
        let total_supply_btc: u128 = 1 * BTC_UNIT;

        // 2. first add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let lp_of_alice_0 = 316227766016;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(DOT_UNIT) * U256::from(BTC_UNIT)
        );

        // 3. swap
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,
            DOT_UNIT,
            1,
            &vec![DOT_ASSET_ID, BTC_ASSET_ID],
            &CHARLIE,
        ));

        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(DOT_UNIT) * U256::from(BTC_UNIT)
        );

        let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
        let balance_dot = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let balance_btc = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

        assert_eq!(balance_dot, 2000000000000000);
        assert_eq!(balance_btc, 50075113);

        let k_last = DexPallet::k_last(sorted_pair);
        let reserve_0 = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let reserve_1 = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);
        let root_k = U256::from(reserve_0)
            .saturating_mul(U256::from(reserve_1))
            .integer_sqrt();
        let root_k_last = k_last.integer_sqrt();

        assert!(root_k > root_k_last);

        let lp_total = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
        let numerator = U256::from(lp_total).saturating_mul(root_k.saturating_sub(root_k_last));
        let denominator = root_k.saturating_mul(U256::from(5u128)).saturating_add(root_k_last);
        let expect_fee = numerator.checked_div(denominator).unwrap_or_default();

        // 4. second add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            BTC_ASSET_ID,
            DOT_ASSET_ID,
            total_supply_btc,
            total_supply_dot,
            0,
            0,
            100
        ));

        let lp_total = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
        let lp_of_alice_2 = 474361420078u128;
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_2
        );

        let lp_of_bob = 39548424u128;
        assert_eq!(expect_fee, U256::from(lp_of_bob));
        assert_eq!(
            U256::from(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB)),
            expect_fee
        );
        assert_eq!(lp_total, lp_of_alice_2 + lp_of_bob);

        assert_eq!(DexPallet::k_last(sorted_pair), U256::from(225338007000000000000000u128));
    });
}

fn calculate_mint_fee(fee_point: u128) -> U256 {
    let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);
    let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
    let reserve_0 = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
    let reserve_1 = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

    let k_last = DexPallet::k_last(sorted_pair);
    let root_k = U256::from(reserve_0)
        .saturating_mul(U256::from(reserve_1))
        .integer_sqrt();
    let root_k_last = k_last.integer_sqrt();
    assert!(root_k > root_k_last);

    let lp_total = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
    let numerator = U256::from(lp_total).saturating_mul(root_k.saturating_sub(root_k_last));
    let denominator = root_k.saturating_mul(U256::from(fee_point)).saturating_add(root_k_last);
    numerator.checked_div(denominator).unwrap_or_default()
}

#[test]
fn turn_on_protocol_fee_swap_have_fee_at_should_work() {
    new_test_ext().execute_with(|| {
        // 1. turn on the protocol fee
        assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
        // use default rate: 1/(1/6)-1=5
        assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), 5u8));

        let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);
        assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            100_000_000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            100_000_000 * BTC_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            100_000_000 * DOT_UNIT
        ));

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
        let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

        // 2. first add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let lp_of_alice_0 = U256::from(total_supply_btc)
            .saturating_mul(U256::from(total_supply_dot))
            .integer_sqrt()
            .as_u128();
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(total_supply_btc) * U256::from(total_supply_dot)
        );

        // 3. swap
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,
            DOT_UNIT,
            1,
            &vec![DOT_ASSET_ID, BTC_ASSET_ID],
            &CHARLIE,
        ));

        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );

        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);
        assert_eq!(
            DexPallet::k_last(sorted_pair),
            U256::from(total_supply_btc) * U256::from(total_supply_dot)
        );

        let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
        let reserve_0 = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let reserve_1 = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

        assert_eq!(reserve_0, total_supply_dot + 1 * DOT_UNIT);
        assert_eq!(reserve_1, total_supply_btc - 99699900);

        let expect_fee = calculate_mint_fee(5);

        let (added_btc, _) =
            DexPallet::calculate_added_amount(1 * BTC_UNIT, 1 * DOT_UNIT, 0, 0, reserve_1, reserve_0).unwrap();

        // 4. second add_liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            BTC_ASSET_ID,
            DOT_ASSET_ID,
            1 * BTC_UNIT,
            1 * DOT_UNIT,
            0,
            0,
            100
        ));

        let lp_fee = <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB);

        let alice_lp_add =
            (U256::from(lp_of_alice_0 + lp_fee) * U256::from(added_btc) / U256::from(reserve_1)).as_u128();

        let lp_total = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0 + alice_lp_add
        );

        let lp_of_bob = <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB);
        assert_eq!(expect_fee, U256::from(lp_of_bob));
        assert_eq!(
            U256::from(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB)),
            expect_fee
        );
        assert_eq!(lp_total, lp_of_alice_0 + alice_lp_add + lp_of_bob);
    });
}

// https://docs.uniswap.org/contracts/v1/guides/trade-tokens#amount-bought-sell-order
fn get_amount_out(
    input_amount: AssetBalance,
    input_reserve: AssetBalance,
    output_reserve: AssetBalance,
    fee_rate: AssetBalance,
) -> AssetBalance {
    let numerator = input_amount * output_reserve * (FEE_ADJUSTMENT - fee_rate);
    let denominator = input_reserve * FEE_ADJUSTMENT + input_amount * (FEE_ADJUSTMENT - fee_rate);
    numerator / denominator
}

#[test]
fn should_set_lower_custom_exchange_fee() {
    new_test_ext().execute_with(|| {
        // add liquidity
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            100_000_000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            100_000_000 * BTC_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            100_000_000 * DOT_UNIT
        ));

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        // decrease exchange fee to 0.02%
        assert_ok!(DexPallet::set_exchange_fee(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            2,
        ));

        let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
        let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let lp_of_alice_0 = U256::from(total_supply_btc)
            .saturating_mul(U256::from(total_supply_dot))
            .integer_sqrt()
            .as_u128();
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);

        // swap
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,                          // who
            DOT_UNIT,                          // amount_in
            1,                                 // amount_out_min
            &vec![DOT_ASSET_ID, BTC_ASSET_ID], // path
            &CHARLIE,                          // recipient
        ));

        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );

        let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
        let reserve_0 = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let reserve_1 = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

        assert_eq!(reserve_0, total_supply_dot + DOT_UNIT);
        assert_eq!(
            reserve_1,
            total_supply_btc - get_amount_out(DOT_UNIT, total_supply_dot, total_supply_btc, 2)
        );
    });
}

#[test]
fn zero_fees_should_work() {
    new_test_ext().execute_with(|| {
        // add liquidity
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            100_000_000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            100_000_000 * BTC_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            100_000_000 * DOT_UNIT
        ));

        // set zero fees
        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            0,
        ));

        let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
        let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let charlie_initial_dot = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &CHARLIE);
        let charlie_initial_btc = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &CHARLIE);
        let dot_swap = total_supply_dot / 2;
        assert_eq!(charlie_initial_btc, 0);

        // swap one direction...
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,                          // who
            dot_swap,                          // amount_in
            1,                                 // amount_out_min
            &vec![DOT_ASSET_ID, BTC_ASSET_ID], // path
            &CHARLIE,                          // recipient
        ));

        assert!(<Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &CHARLIE) < charlie_initial_dot);

        // swap the same amount back
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,                                                              // who
            <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &CHARLIE), // amount_in
            1,                                                                     // amount_out_min
            &vec![BTC_ASSET_ID, DOT_ASSET_ID],                                     // path
            &CHARLIE,                                                              // recipient
        ));

        // there will be some small difference due to roundings, but check that this loss is insignificant
        let loss = charlie_initial_dot - <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &CHARLIE);
        assert!(loss < dot_swap / 1_000_000_000_000);
    });
}

#[test]
fn should_set_higher_custom_exchange_fee() {
    new_test_ext().execute_with(|| {
        // add liquidity
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            100_000_000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            100_000_000 * BTC_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            100_000_000 * DOT_UNIT
        ));

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
        let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

        // less out with higher swap fee_rate
        assert_eq!(
            99999900,
            get_amount_out(DOT_UNIT, total_supply_dot, total_supply_btc, 0)
        );
        assert_eq!(
            69999951,
            get_amount_out(DOT_UNIT, total_supply_dot, total_supply_btc, 3000)
        );
        assert_eq!(
            49999975,
            get_amount_out(DOT_UNIT, total_supply_dot, total_supply_btc, 5000)
        );
        assert_eq!(
            29999991,
            get_amount_out(DOT_UNIT, total_supply_dot, total_supply_btc, 7000)
        );

        // increase exchange fee to 50%
        assert_ok!(DexPallet::set_exchange_fee(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            5000
        ));

        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        let lp_of_alice_0 = U256::from(total_supply_btc)
            .saturating_mul(U256::from(total_supply_dot))
            .integer_sqrt()
            .as_u128();
        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );
        assert_eq!(<Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB), 0);

        // swap
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,                          // who
            DOT_UNIT,                          // amount_in
            1,                                 // amount_out_min
            &vec![DOT_ASSET_ID, BTC_ASSET_ID], // path
            &CHARLIE,                          // recipient
        ));

        assert_eq!(
            <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &ALICE),
            lp_of_alice_0
        );

        let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
        let reserve_0 = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
        let reserve_1 = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);

        assert_eq!(reserve_0, total_supply_dot + DOT_UNIT);
        assert_eq!(
            reserve_1,
            total_supply_btc - get_amount_out(DOT_UNIT, total_supply_dot, total_supply_btc, 5000)
        );
    });
}

#[test]
fn should_set_max_fee_point() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
        // 1/1-1=0 = 100%
        assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), 0u8));

        // add liquidity
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            100_000_000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            100_000_000 * BTC_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            100_000_000 * DOT_UNIT
        ));

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
        let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        // swap
        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,                          // who
            DOT_UNIT,                          // amount_in
            1,                                 // amount_out_min
            &vec![DOT_ASSET_ID, BTC_ASSET_ID], // path
            &CHARLIE,                          // recipient
        ));

        let expect_fee = calculate_mint_fee(0);

        // add more liquidity
        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            BTC_ASSET_ID,
            DOT_ASSET_ID,
            1 * BTC_UNIT,
            1 * DOT_UNIT,
            0,
            0,
            100
        ));

        let lp_fee = <Test as Config>::MultiCurrency::free_balance(LP_DOT_BTC, &BOB);
        assert_eq!(expect_fee, U256::from(lp_fee));
    });
}

#[test]
fn assert_higher_fee_point_decreases_protocol_fee() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));

        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &ALICE,
            100_000_000 * DOT_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            BTC_ASSET_ID,
            &ALICE,
            100_000_000 * BTC_UNIT
        ));
        assert_ok!(<Test as Config>::MultiCurrency::deposit(
            DOT_ASSET_ID,
            &CHARLIE,
            100_000_000 * DOT_UNIT
        ));

        assert_ok!(DexPallet::create_pair(
            RawOrigin::Root.into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            DEFAULT_FEE_RATE,
        ));

        let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
        let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

        assert_ok!(DexPallet::add_liquidity(
            RawOrigin::Signed(ALICE).into(),
            DOT_ASSET_ID,
            BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0,
            100
        ));

        assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
            &CHARLIE,
            DOT_UNIT,
            1,
            &vec![DOT_ASSET_ID, BTC_ASSET_ID],
            &CHARLIE,
        ));

        fn mint_fee_for_point(fee_point: u8) -> u128 {
            let pair_dot_btc = DexGeneral::pair_account_id(DOT_ASSET_ID, BTC_ASSET_ID);
            let reserve_0 = <Test as Config>::MultiCurrency::free_balance(DOT_ASSET_ID, &pair_dot_btc);
            let reserve_1 = <Test as Config>::MultiCurrency::free_balance(BTC_ASSET_ID, &pair_dot_btc);
            let total_supply = <Test as Config>::MultiCurrency::total_issuance(LP_DOT_BTC);
            assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), fee_point));
            DexPallet::mint_protocol_fee(reserve_0, reserve_1, DOT_ASSET_ID, BTC_ASSET_ID, total_supply).unwrap()
        }

        // 1/(1/1)-1=0
        let total_fee = mint_fee_for_point(0);
        // 1/(1/2)-1=1
        assert_eq!(mint_fee_for_point(1), total_fee / 2);
        // 1/(1/4)-1=3
        assert_eq!(mint_fee_for_point(3), total_fee / 4);
        // 1/(1/6)-1=5
        assert_eq!(mint_fee_for_point(5), total_fee / 6);
        // 1/(1/10)-1=9
        assert_eq!(mint_fee_for_point(9), total_fee / 10);
        // 1/(1/100)-1=99
        assert_eq!(mint_fee_for_point(99), total_fee / 100);
    });
}
