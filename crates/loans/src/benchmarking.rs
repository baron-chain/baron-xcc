//! Loans pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]
#![allow(dead_code)]
#![allow(unused_imports)]
use super::*;
use crate::{AccountBorrows, Pallet as Loans};

use frame_benchmarking::v2::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{self, RawOrigin as SystemOrigin};
use oracle::Pallet as Oracle;
use primitives::{
    Balance,
    CurrencyId::{self, LendToken, Token},
    UnsignedFixedPoint, DOT as DOT_CURRENCY, INTR as INTR_CURRENCY, KBTC as KBTC_CURRENCY, KINT as KINT_CURRENCY,
    KSM as KSM_CURRENCY,
};
use rate_model::{InterestRateModel, JumpModel};
use security::Pallet as Security;
use sp_std::prelude::*;

const SEED: u32 = 0;

const KSM: CurrencyId = Token(KSM_CURRENCY);
const KBTC: CurrencyId = Token(KBTC_CURRENCY);
const LEND_KSM: CurrencyId = LendToken(3);
const LEND_KBTC: CurrencyId = LendToken(4);
const DOT: CurrencyId = Token(DOT_CURRENCY);
const LEND_DOT: CurrencyId = LendToken(1);
const INTR: CurrencyId = Token(INTR_CURRENCY);
const KINT: CurrencyId = Token(KINT_CURRENCY);

const RATE_MODEL_MOCK: InterestRateModel = InterestRateModel::Jump(JumpModel {
    base_rate: Rate::from_inner(Rate::DIV / 100 * 2),
    jump_rate: Rate::from_inner(Rate::DIV / 100 * 10),
    full_rate: Rate::from_inner(Rate::DIV / 100 * 32),
    jump_utilization: Ratio::from_percent(80),
});

fn market_mock<T: Config>() -> Market<BalanceOf<T>> {
    Market {
        close_factor: Ratio::from_percent(50),
        collateral_factor: Ratio::from_percent(50),
        liquidation_threshold: Ratio::from_percent(55),
        liquidate_incentive: Rate::from_inner(Rate::DIV / 100 * 110),
        state: MarketState::Active,
        rate_model: InterestRateModel::Jump(JumpModel {
            base_rate: Rate::from_inner(Rate::DIV / 100 * 2),
            jump_rate: Rate::from_inner(Rate::DIV / 100 * 10),
            full_rate: Rate::from_inner(Rate::DIV / 100 * 32),
            jump_utilization: Ratio::from_percent(80),
        }),
        reserve_factor: Ratio::from_percent(15),
        liquidate_incentive_reserved_factor: Ratio::from_percent(3),
        supply_cap: 1_000_000_000_000_000_000_000u128, // set to 1B
        borrow_cap: 1_000_000_000_000_000_000_000u128, // set to 1B
        lend_token_id: CurrencyId::LendToken(4),
    }
}

fn pending_market_mock<T: Config>(lend_token_id: CurrencyId) -> Market<BalanceOf<T>> {
    let mut market = market_mock::<T>();
    market.state = MarketState::Pending;
    market.lend_token_id = lend_token_id;
    market
}

fn transfer_initial_balance<T: Config + orml_tokens::Config<CurrencyId = CurrencyId, Balance = Balance>>(
    caller: T::AccountId,
) {
    let account_id = T::Lookup::unlookup(caller.clone());
    orml_tokens::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id.clone(),
        LEND_KSM,
        10_000_000_000_000_u128,
        0_u128,
    )
    .unwrap();
    orml_tokens::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id.clone(),
        KBTC,
        10_000_000_000_000_u128,
        0_u128,
    )
    .unwrap();

    orml_tokens::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id.clone(),
        KSM,
        10_000_000_000_000_u128,
        0_u128,
    )
    .unwrap();

    orml_tokens::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id.clone(),
        DOT,
        10_000_000_000_000_u128,
        0_u128,
    )
    .unwrap();

    orml_tokens::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id.clone(),
        INTR,
        10_000_000_000_000_u128,
        0_u128,
    )
    .unwrap();

    orml_tokens::Pallet::<T>::set_balance(
        SystemOrigin::Root.into(),
        account_id,
        KINT,
        10_000_000_000_000_u128,
        0_u128,
    )
    .unwrap();
}

fn set_account_borrows<T: Config>(who: T::AccountId, asset_id: CurrencyId, borrow_balance: BalanceOf<T>) {
    AccountBorrows::<T>::insert(
        asset_id,
        &who,
        BorrowSnapshot {
            principal: borrow_balance,
            borrow_index: Rate::one(),
        },
    );
    TotalBorrows::<T>::insert(asset_id, borrow_balance);
    let amount: Amount<T> = Amount::new(borrow_balance, asset_id);
    amount.lock_on(&who).unwrap();
    amount.burn_from(&who).unwrap();
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks(where
            T: orml_tokens::Config<CurrencyId = CurrencyId, Balance = Balance>, T: security::Config, T: oracle::Config)
            ]
pub mod benchmarks {
    use frame_benchmarking::v2::extrinsic_call;

    use super::*;

    #[benchmark]
    pub fn add_market() {
        #[extrinsic_call]
        Loans::add_market(SystemOrigin::Root, KBTC, pending_market_mock::<T>(LEND_KBTC));
        assert_last_event::<T>(
            Event::<T>::NewMarket {
                underlying_currency_id: KBTC,
                market: pending_market_mock::<T>(LEND_KBTC),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn activate_market() {
        Loans::<T>::add_market(SystemOrigin::Root.into(), KSM, pending_market_mock::<T>(LEND_KSM)).unwrap();
        #[extrinsic_call]
        Loans::activate_market(SystemOrigin::Root, KSM);

        assert_last_event::<T>(
            Event::<T>::ActivatedMarket {
                underlying_currency_id: KSM,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn update_rate_model() {
        Loans::<T>::add_market(SystemOrigin::Root.into(), KSM, pending_market_mock::<T>(LEND_KSM)).unwrap();
        #[extrinsic_call]
        Loans::update_rate_model(SystemOrigin::Root, KSM, RATE_MODEL_MOCK);
        let mut market = pending_market_mock::<T>(LEND_KSM);
        market.rate_model = RATE_MODEL_MOCK;
        assert_last_event::<T>(
            Event::<T>::UpdatedMarket {
                underlying_currency_id: KSM,
                market,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn update_market() {
        Loans::<T>::add_market(SystemOrigin::Root.into(), KSM, pending_market_mock::<T>(LEND_KSM)).unwrap();
        #[extrinsic_call]
        Loans::update_market(
            SystemOrigin::Root,
            KSM,
            Some(Ratio::from_percent(50)),
            Some(Ratio::from_percent(55)),
            Some(Ratio::from_percent(50)),
            Some(Ratio::from_percent(15)),
            Some(Ratio::from_percent(3)),
            Some(Rate::from_inner(Rate::DIV / 100 * 110)),
            Some(1_000_000_000_000_000_000_000u128),
            Some(1_000_000_000_000_000_000_000u128),
        );
        let mut market = pending_market_mock::<T>(LEND_KSM);
        market.reserve_factor = Ratio::from_percent(50);
        market.close_factor = Ratio::from_percent(15);
        assert_last_event::<T>(
            Event::<T>::UpdatedMarket {
                underlying_currency_id: KSM,
                market,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn force_update_market() {
        Loans::<T>::add_market(SystemOrigin::Root.into(), KBTC, pending_market_mock::<T>(LEND_KBTC)).unwrap();
        #[extrinsic_call]
        Loans::force_update_market(SystemOrigin::Root, KBTC, pending_market_mock::<T>(LEND_KBTC));
        assert_last_event::<T>(
            Event::<T>::UpdatedMarket {
                underlying_currency_id: KBTC,
                market: pending_market_mock::<T>(LEND_KBTC),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn add_reward() {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        #[extrinsic_call]
        Loans::add_reward(SystemOrigin::Signed(caller.clone()), 1_000_000_000_000_u128);
        assert_last_event::<T>(
            Event::<T>::RewardAdded {
                payer: caller,
                amount: 1_000_000_000_000_u128,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn update_market_reward_speed() {
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        #[extrinsic_call]
        Loans::update_market_reward_speed(SystemOrigin::Root, KBTC, Some(1_000_000), Some(1_000_000));
        assert_last_event::<T>(
            Event::<T>::MarketRewardSpeedUpdated {
                underlying_currency_id: KBTC,
                supply_reward_per_block: 1_000_000,
                borrow_reward_per_block: 1_000_000,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn claim_reward() {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            100_000_000
        ));
        assert_ok!(Loans::<T>::add_reward(
            SystemOrigin::Signed(caller.clone()).into(),
            1_000_000_000_000_u128
        ));
        assert_ok!(Loans::<T>::update_market_reward_speed(
            SystemOrigin::Root.into(),
            KBTC,
            Some(1_000_000),
            Some(1_000_000)
        ));
        let target_height = frame_system::Pallet::<T>::block_number().saturating_add(One::one());
        frame_system::Pallet::<T>::set_block_number(target_height);
        #[extrinsic_call]
        Loans::claim_reward(SystemOrigin::Signed(caller.clone()));
        assert_last_event::<T>(
            Event::<T>::RewardPaid {
                receiver: caller,
                amount: 1_000_000,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn claim_reward_for_market() {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            100_000_000
        ));
        assert_ok!(Loans::<T>::add_reward(
            SystemOrigin::Signed(caller.clone()).into(),
            1_000_000_000_000_u128
        ));
        assert_ok!(Loans::<T>::update_market_reward_speed(
            SystemOrigin::Root.into(),
            KBTC,
            Some(1_000_000),
            Some(1_000_000)
        ));
        let target_height = frame_system::Pallet::<T>::block_number().saturating_add(One::one());
        frame_system::Pallet::<T>::set_block_number(target_height);
        #[extrinsic_call]
        Loans::claim_reward_for_market(SystemOrigin::Signed(caller.clone()), KBTC);
        assert_last_event::<T>(
            Event::<T>::RewardPaid {
                receiver: caller,
                amount: 1_000_000,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn mint() {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        let amount: u32 = 100_000;
        #[extrinsic_call]
        Loans::mint(SystemOrigin::Signed(caller.clone()), KBTC, amount.into());
        assert_last_event::<T>(
            Event::<T>::Deposited {
                account_id: caller,
                currency_id: KBTC,
                amount: amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn borrow() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::deposit_all_collateral(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC
        ));
        #[extrinsic_call]
        Loans::borrow(SystemOrigin::Signed(caller.clone()), KBTC, borrowed_amount.into());
        assert_last_event::<T>(
            Event::<T>::Borrowed {
                account_id: caller,
                currency_id: KBTC,
                amount: borrowed_amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn redeem() {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 100_000_000;
        let redeem_amount: u32 = 100_000;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        #[extrinsic_call]
        Loans::redeem(SystemOrigin::Signed(caller.clone()), KBTC, redeem_amount.into());
        assert_last_event::<T>(
            Event::<T>::Redeemed {
                account_id: caller,
                currency_id: KBTC,
                amount: redeem_amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn redeem_all() {
        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        #[extrinsic_call]
        Loans::redeem_all(SystemOrigin::Signed(caller.clone()), KBTC);
        assert_last_event::<T>(
            Event::<T>::Redeemed {
                account_id: caller,
                currency_id: KBTC,
                amount: deposit_amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn repay_borrow() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        let repay_amount: u32 = 100;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::deposit_all_collateral(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC
        ));
        assert_ok!(Loans::<T>::borrow(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            borrowed_amount.into()
        ));
        #[extrinsic_call]
        Loans::repay_borrow(SystemOrigin::Signed(caller.clone()), KBTC, repay_amount.into());

        assert_last_event::<T>(
            Event::<T>::RepaidBorrow {
                account_id: caller,
                currency_id: KBTC,
                amount: repay_amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn repay_borrow_all() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 100_000_000;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));

        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::deposit_all_collateral(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC
        ));
        assert_ok!(Loans::<T>::borrow(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            borrowed_amount.into()
        ));
        #[extrinsic_call]
        Loans::repay_borrow_all(SystemOrigin::Signed(caller.clone()), KBTC);
        assert_last_event::<T>(
            Event::<T>::RepaidBorrow {
                account_id: caller,
                currency_id: KBTC,
                amount: borrowed_amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn deposit_all_collateral() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u128 = 200_000_000;
        // Divide by the default exchange rate.
        // Use a hardcoded value becuase the `FixedU128` used
        // in the benchmarks does not support `to_float`, whereas the version in benchmark tests does.
        // let rate = Loans::<T>::min_exchange_rate().to_float();
        let rate: f64 = 0.02;
        let expected_lend_tokens = deposit_amount as f64 / rate;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount
        ));
        #[extrinsic_call]
        Loans::deposit_all_collateral(SystemOrigin::Signed(caller.clone()), KBTC);
        assert_last_event::<T>(
            Event::<T>::DepositCollateral {
                account_id: caller,
                currency_id: LEND_KBTC,
                amount: expected_lend_tokens as u128,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn withdraw_all_collateral() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        transfer_initial_balance::<T>(caller.clone());
        let deposit_amount: u128 = 200_000_000;
        // divide by the default exchange rate
        let rate: f64 = 0.02;
        let expected_lend_tokens = deposit_amount as f64 / rate;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC,
            deposit_amount
        ));
        assert_ok!(Loans::<T>::deposit_all_collateral(
            SystemOrigin::Signed(caller.clone()).into(),
            KBTC
        ));
        #[extrinsic_call]
        Loans::withdraw_all_collateral(SystemOrigin::Signed(caller.clone()), KBTC);
        assert_last_event::<T>(
            Event::<T>::WithdrawCollateral {
                account_id: caller,
                currency_id: LEND_KBTC,
                amount: expected_lend_tokens as u128,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn liquidate_borrow() {
        initialize::<T>();

        Security::<T>::set_active_block_number(1u32.into());
        let alice: T::AccountId = account("Sample", 100, SEED);
        let bob: T::AccountId = account("Sample", 101, SEED);
        transfer_initial_balance::<T>(alice.clone());
        transfer_initial_balance::<T>(bob.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 200_000_000;
        let liquidate_amount: u32 = 100_000_000;
        let incentive_amount: u32 = 110_000_000;
        assert_ok!(Oracle::<T>::_set_exchange_rate(DOT, UnsignedFixedPoint::one()));
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            DOT,
            pending_market_mock::<T>(LEND_DOT)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(bob.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(alice.clone()).into(),
            DOT,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::deposit_all_collateral(
            SystemOrigin::Signed(alice.clone()).into(),
            DOT
        ));
        set_account_borrows::<T>(alice.clone(), KBTC, borrowed_amount.into());
        #[extrinsic_call]
        Loans::liquidate_borrow(
            SystemOrigin::Signed(bob.clone()),
            alice.clone(),
            KBTC,
            liquidate_amount.into(),
            DOT,
        );
        assert_last_event::<T>(
            Event::<T>::LiquidatedBorrow {
                liquidator: bob.clone(),
                borrower: alice.clone(),
                liquidation_currency_id: KBTC,
                collateral_currency_id: DOT,
                repay_amount: liquidate_amount.into(),
                collateral_underlying_amount: incentive_amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn reduce_incentive_reserves() {
        initialize::<T>();

        let alice: T::AccountId = account("Sample", 100, SEED);
        let bob: T::AccountId = account("Sample", 101, SEED);
        transfer_initial_balance::<T>(alice.clone());
        transfer_initial_balance::<T>(bob.clone());
        let deposit_amount: u32 = 200_000_000;
        let borrowed_amount: u32 = 200_000_000;
        let liquidate_amount: u32 = 100_000_000;
        assert_ok!(Oracle::<T>::_set_exchange_rate(DOT, UnsignedFixedPoint::one()));
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            DOT,
            pending_market_mock::<T>(LEND_DOT)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), DOT));
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(bob.clone()).into(),
            KBTC,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::mint(
            SystemOrigin::Signed(alice.clone()).into(),
            DOT,
            deposit_amount.into()
        ));
        assert_ok!(Loans::<T>::deposit_all_collateral(
            SystemOrigin::Signed(alice.clone()).into(),
            DOT
        ));
        set_account_borrows::<T>(alice.clone(), KBTC, borrowed_amount.into());
        assert_ok!(Loans::<T>::liquidate_borrow(
            SystemOrigin::Signed(bob.clone()).into(),
            alice.clone(),
            KBTC,
            liquidate_amount.into(),
            DOT
        ));
        let incentive_reward_account_id = Loans::<T>::incentive_reward_account_id();
        let reward_lend_tokens = orml_tokens::Pallet::<T>::free_balance(LEND_DOT, &incentive_reward_account_id);
        let rate: f64 = 0.02;
        let reward_underlying = (reward_lend_tokens as f64 * rate) as u128;
        let receiver = T::Lookup::unlookup(alice.clone());
        #[extrinsic_call]
        Loans::reduce_incentive_reserves(SystemOrigin::Root, receiver.clone().into(), DOT, reward_underlying);
        assert_last_event::<T>(
            Event::<T>::IncentiveReservesReduced {
                receiver: alice.clone(),
                currency_id: DOT,
                amount: reward_underlying,
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn add_reserves() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        transfer_initial_balance::<T>(caller.clone());
        let amount: u32 = 2000;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        #[extrinsic_call]
        Loans::add_reserves(SystemOrigin::Root, payer, KBTC, amount.into());
        assert_last_event::<T>(
            Event::<T>::ReservesAdded {
                payer: caller,
                currency_id: KBTC,
                amount: amount.into(),
                new_reserve_amount: amount.into(),
            }
            .into(),
        );
    }

    #[benchmark]
    pub fn reduce_reserves() {
        initialize::<T>();

        let caller: T::AccountId = whitelisted_caller();
        let payer = T::Lookup::unlookup(caller.clone());
        transfer_initial_balance::<T>(caller.clone());
        let add_amount: u32 = 2000;
        let reduce_amount: u32 = 1000;
        assert_ok!(Loans::<T>::add_market(
            SystemOrigin::Root.into(),
            KBTC,
            pending_market_mock::<T>(LEND_KBTC)
        ));
        assert_ok!(Loans::<T>::activate_market(SystemOrigin::Root.into(), KBTC));
        assert_ok!(Loans::<T>::add_reserves(
            SystemOrigin::Root.into(),
            payer.clone(),
            KBTC,
            add_amount.into()
        ));
        #[extrinsic_call]
        Loans::reduce_reserves(SystemOrigin::Root, payer, KBTC, reduce_amount.into());
        assert_last_event::<T>(
            Event::<T>::ReservesReduced {
                receiver: caller,
                currency_id: KBTC,
                amount: reduce_amount.into(),
                new_reserve_amount: (add_amount - reduce_amount).into(),
            }
            .into(),
        );
    }

    impl_benchmark_test_suite!(Loans, crate::mock::new_test_ext_no_markets(), crate::mock::Test);
}

fn initialize<
    T: orml_tokens::Config<CurrencyId = CurrencyId>
        + security::Config
        + oracle::Config
        + currency::Config<UnsignedFixedPoint = UnsignedFixedPoint>,
>() {
    Security::<T>::set_active_block_number(1u32.into());
    assert_ok!(Oracle::<T>::_set_exchange_rate(KBTC, UnsignedFixedPoint::one()));
}
