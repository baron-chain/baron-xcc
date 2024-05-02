use crate::setup::{assert_eq, *};
use frame_support::traits::{Currency, OnInitialize};
use sp_io::hashing::blake2_256;
use sp_runtime::Permill;

type EscrowAnnuityPallet = annuity::Pallet<Runtime, EscrowAnnuityInstance>;

type VaultAnnuityCall = annuity::Call<Runtime, VaultAnnuityInstance>;
type VaultAnnuityPallet = annuity::Pallet<Runtime, VaultAnnuityInstance>;
type VaultAnnuityEvent = annuity::Event<Runtime, VaultAnnuityInstance>;

type SupplyPallet = supply::Pallet<Runtime>;

fn get_last_reward() -> Balance {
    SystemPallet::events()
        .iter()
        .rev()
        .find_map(|record| {
            if let RuntimeEvent::VaultAnnuity(VaultAnnuityEvent::BlockReward(reward)) = record.event {
                Some(reward)
            } else {
                None
            }
        })
        .expect("nothing was rewarded")
}

#[test]
fn integration_test_annuity() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(RuntimeCall::Tokens(TokensCall::set_balance {
            who: VaultAnnuityPallet::account_id(),
            currency_id: DEFAULT_NATIVE_CURRENCY,
            new_free: 10_000_000_000_000,
            new_reserved: 0,
        })
        .dispatch(root()));
        VaultAnnuityPallet::update_reward_per_block();
        VaultAnnuityPallet::on_initialize(1);

        let emission_period = <Runtime as annuity::Config<VaultAnnuityInstance>>::EmissionPeriod::get() as u128;
        let expected_reward = 10_000_000_000_000 / emission_period as u128;
        for i in 1..1000 {
            VaultAnnuityPallet::on_initialize(i);
            assert_eq!(get_last_reward(), expected_reward);
        }
    })
}

#[test]
fn integration_test_annuity_capped() {
    ExtBuilder::build().execute_with(|| {
        let reward_per_wrapped = 1;
        assert_ok!(
            RuntimeCall::VaultAnnuity(VaultAnnuityCall::set_reward_per_wrapped { reward_per_wrapped }).dispatch(root())
        );

        let wrapped_issuance = 100_000; // 0.001 BTC
        let native_issuance = 10_000_000_000_000; // 1000 INTR

        assert_ok!(RuntimeCall::Tokens(TokensCall::set_balance {
            who: account_of(FAUCET),
            currency_id: DEFAULT_WRAPPED_CURRENCY,
            new_free: wrapped_issuance,
            new_reserved: 0,
        })
        .dispatch(root()));

        assert_ok!(RuntimeCall::Tokens(TokensCall::set_balance {
            who: VaultAnnuityPallet::account_id(),
            currency_id: DEFAULT_NATIVE_CURRENCY,
            new_free: native_issuance,
            new_reserved: 0,
        })
        .dispatch(root()));

        let emission_period = <Runtime as annuity::Config<VaultAnnuityInstance>>::EmissionPeriod::get() as u128;

        VaultAnnuityPallet::update_reward_per_block();

        // reward should be capped to wrapped_issuance
        VaultAnnuityPallet::on_initialize(1);
        assert_eq!(get_last_reward(), reward_per_wrapped * wrapped_issuance);

        // issuance is increased
        let wrapped_issuance = 100_000_000; // 1 BTC
        assert_ok!(RuntimeCall::Tokens(TokensCall::set_balance {
            who: account_of(FAUCET),
            currency_id: DEFAULT_WRAPPED_CURRENCY,
            new_free: wrapped_issuance,
            new_reserved: 0,
        })
        .dispatch(root()));

        // now the cap is the original rewards_per_block
        VaultAnnuityPallet::on_initialize(2);
        assert_eq!(get_last_reward(), native_issuance / emission_period);
    })
}

#[test]
fn rewards_are_not_distributed_if_annuity_has_no_balance() {
    ExtBuilder::build().execute_with(|| {
        VaultAnnuityPallet::update_reward_per_block();
        VaultAnnuityPallet::on_initialize(1);

        let expected_reward = 0;
        for i in 1..1000 {
            VaultAnnuityPallet::on_initialize(i);
            assert_eq!(get_last_reward(), expected_reward);
        }
    })
}

#[test]
fn should_distribute_vault_rewards_from_supply() {
    ExtBuilder::build().execute_with(|| {
        // full distribution is minted on genesis
        assert_eq!(
            NativeCurrency::total_balance(&SupplyPallet::account_id()),
            token_distribution::INITIAL_ALLOCATION
        );

        // distribute the four year supply (300 million INTR) for the vault block rewards
        let total_rewards = Permill::from_percent(30) * token_distribution::INITIAL_ALLOCATION;
        // NOTE: start height cannot be the current height or in the past
        let start_height = SystemPallet::block_number() + 1;
        assert_ok!(RuntimeCall::Utility(UtilityCall::batch {
            calls: vec![
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 1"[..]),
                    when: start_height + YEARS * 0,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: VaultAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(40) * total_rewards,
                    })),
                }),
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 2"[..]),
                    when: start_height + YEARS * 1,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: VaultAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(30) * total_rewards,
                    })),
                }),
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 3"[..]),
                    when: start_height + YEARS * 2,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: VaultAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(20) * total_rewards,
                    })),
                }),
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 4"[..]),
                    when: start_height + YEARS * 3,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: VaultAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(10) * total_rewards,
                    })),
                })
            ],
        })
        .dispatch(root()));

        // Year 1: 120 million INTR are distributed to the vault annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 0);
        assert_eq!(
            NativeCurrency::total_balance(&VaultAnnuityPallet::account_id()),
            Permill::from_percent(40) * total_rewards
        );

        // Year 2: 90 million INTR are distributed to the vault annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 1);
        assert_eq!(
            NativeCurrency::total_balance(&VaultAnnuityPallet::account_id()),
            Permill::from_percent(70) * total_rewards
        );

        // Year 3: 60 million INTR are distributed to the vault annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 2);
        assert_eq!(
            NativeCurrency::total_balance(&VaultAnnuityPallet::account_id()),
            Permill::from_percent(90) * total_rewards
        );

        // Year 4: 30 million INTR are distributed to the vault annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 3);
        assert_eq!(
            NativeCurrency::total_balance(&VaultAnnuityPallet::account_id()),
            Permill::from_percent(100) * total_rewards
        );
    })
}

#[test]
fn should_distribute_escrow_rewards_from_supply() {
    ExtBuilder::build().execute_with(|| {
        // full distribution is minted on genesis
        assert_eq!(
            NativeCurrency::total_balance(&SupplyPallet::account_id()),
            token_distribution::INITIAL_ALLOCATION
        );

        // distribute the four year supply (50 million INTR) for the stake to vote rewards
        let total_rewards = Permill::from_percent(5) * token_distribution::INITIAL_ALLOCATION;
        // NOTE: start height cannot be the current height or in the past
        let start_height = SystemPallet::block_number() + 1;
        assert_ok!(RuntimeCall::Utility(UtilityCall::batch {
            calls: vec![
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 1"[..]),
                    when: start_height + YEARS * 0,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: EscrowAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(25) * total_rewards,
                    })),
                }),
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 2"[..]),
                    when: start_height + YEARS * 1,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: EscrowAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(25) * total_rewards,
                    })),
                }),
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 3"[..]),
                    when: start_height + YEARS * 2,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: EscrowAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(25) * total_rewards,
                    })),
                }),
                RuntimeCall::Scheduler(SchedulerCall::schedule_named {
                    id: blake2_256(&b"Year 4"[..]),
                    when: start_height + YEARS * 3,
                    maybe_periodic: None,
                    priority: 63,
                    call: Box::new(RuntimeCall::Tokens(TokensCall::force_transfer {
                        source: SupplyPallet::account_id(),
                        dest: EscrowAnnuityPallet::account_id(),
                        currency_id: DEFAULT_NATIVE_CURRENCY,
                        amount: Permill::from_percent(25) * total_rewards,
                    })),
                })
            ],
        })
        .dispatch(root()));

        // Year 1: 12,500,000 INTR are distributed to the stake-to-vote annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 0);
        assert_eq!(
            NativeCurrency::total_balance(&EscrowAnnuityPallet::account_id()),
            Permill::from_percent(25) * total_rewards
        );

        // Year 2: 12,500,000 INTR are distributed to the stake-to-vote annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 1);
        assert_eq!(
            NativeCurrency::total_balance(&EscrowAnnuityPallet::account_id()),
            Permill::from_percent(50) * total_rewards
        );

        // Year 3: 12,500,000 INTR are distributed to the stake-to-vote annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 2);
        assert_eq!(
            NativeCurrency::total_balance(&EscrowAnnuityPallet::account_id()),
            Permill::from_percent(75) * total_rewards
        );

        // Year 4: 12,500,000 INTR are distributed to the stake-to-vote annuity pallet
        SchedulerPallet::on_initialize(start_height + YEARS * 3);
        assert_eq!(
            NativeCurrency::total_balance(&EscrowAnnuityPallet::account_id()),
            Permill::from_percent(100) * total_rewards
        );
    })
}
