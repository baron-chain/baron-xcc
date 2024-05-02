use crate::{
    setup::{assert_eq, *},
    utils::{loans_utils::activate_lending_and_mint, nomination_utils::*},
};
use currency::Amount;

fn test_with<R>(execute: impl Fn(VaultId) -> R) {
    let test_with = |currency_id, wrapped_id| {
        ExtBuilder::build().execute_with(|| {
            SecurityPallet::set_active_block_number(1);
            for currency_id in iter_collateral_currencies().filter(|c| !c.is_lend_token()) {
                assert_ok!(OraclePallet::_set_exchange_rate(currency_id, FixedU128::one()));
            }
            if wrapped_id != DEFAULT_WRAPPED_CURRENCY {
                assert_ok!(OraclePallet::_set_exchange_rate(wrapped_id, FixedU128::one()));
            }
            activate_lending_and_mint(Token(DOT), LendToken(1));
            UserData::force_to(USER, default_user_state());
            let vault_id = PrimitiveVaultId::new(account_of(VAULT), currency_id, wrapped_id);
            LiquidationVaultData::force_to(default_liquidation_vault_state(&vault_id.currencies));
            CoreVaultData::force_to(&vault_id, default_vault_state(&vault_id));

            execute(vault_id)
        });
    };
    test_with(Token(DOT), Token(KBTC));
    test_with(Token(KSM), Token(IBTC));
    test_with(Token(DOT), Token(IBTC));
    test_with(ForeignAsset(1), Token(IBTC));
    test_with(LendToken(1), Token(IBTC));
}

fn test_with_nomination_enabled<R>(execute: impl Fn(VaultId) -> R) {
    test_with(|vault_id| {
        enable_nomination();
        execute(vault_id)
    })
}

fn test_with_nomination_enabled_and_vault_opted_in<R>(execute: impl Fn(VaultId) -> R) {
    test_with_nomination_enabled(|vault_id| {
        assert_nomination_opt_in(&vault_id);
        set_commission(&vault_id, FixedU128::from_float(COMMISSION));

        execute(vault_id)
    })
}

fn default_nomination(vault_id: &VaultId) -> Amount<Runtime> {
    Amount::new(DEFAULT_NOMINATION, vault_id.collateral_currency())
}

mod spec_based_tests {
    use super::{assert_eq, *};
    use sp_runtime::DispatchError;

    #[test]
    fn integration_test_enable_nomination() {
        // PRECONDITION: The calling account MUST be root or the function MUST be called from a passed governance
        // referendum. POSTCONDITION: The `NominationEnabled` scalar MUST be set to the value of the `enabled`
        // parameter.
        test_with(|_| {
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::set_nomination_enabled { enabled: true })
                    .dispatch(origin_of(account_of(CAROL))),
                DispatchError::BadOrigin
            );
            let mut nomination_enabled = true;
            assert_ok!(RuntimeCall::Nomination(NominationCall::set_nomination_enabled {
                enabled: nomination_enabled
            })
            .dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
            assert_eq!(NominationPallet::is_nomination_enabled(), nomination_enabled);
            nomination_enabled = false;
            assert_ok!(RuntimeCall::Nomination(NominationCall::set_nomination_enabled {
                enabled: nomination_enabled
            })
            .dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
            assert_eq!(NominationPallet::is_nomination_enabled(), nomination_enabled);
        })
    }

    #[test]
    fn integration_test_opt_in() {
        // PRECONDITIONS:
        //   - The BTC Parachain status in the Security component MUST be `RUNNING:0`.
        //   - A Vault with id `vaultId` MUST be registered.
        //   - The Vault MUST NOT be opted in.
        // POSTCONDITION: The Vault MUST be allowed to receive nominated collateral.
        test_with(|_| {
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::set_nomination_enabled { enabled: true })
                    .dispatch(origin_of(account_of(CAROL))),
                DispatchError::BadOrigin
            );
            let mut nomination_enabled = true;
            assert_ok!(RuntimeCall::Nomination(NominationCall::set_nomination_enabled {
                enabled: nomination_enabled
            })
            .dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
            assert_eq!(NominationPallet::is_nomination_enabled(), nomination_enabled);
            nomination_enabled = false;
            assert_ok!(RuntimeCall::Nomination(NominationCall::set_nomination_enabled {
                enabled: nomination_enabled
            })
            .dispatch(<Runtime as frame_system::Config>::RuntimeOrigin::root()));
            assert_eq!(NominationPallet::is_nomination_enabled(), nomination_enabled);
        })
    }

    #[test]
    fn integration_test_opt_out_preconditions() {
        // PRECONDITIONS:
        //   - A Vault with id `vaultId` MUST be registered.
        //   - A Vault with id `vaultId` MUST exist in the Vaults mapping.
        test_with(|_| {
            assert_noop!(
                nomination_opt_out(&vault_id_of(USER, Token(DOT))),
                NominationError::VaultNotOptedInToNomination
            );
            assert_noop!(
                nomination_opt_out(&vault_id_of(VAULT, Token(DOT))),
                NominationError::VaultNotOptedInToNomination
            );
        })
    }

    #[test]
    fn integration_test_opt_out_postconditions() {
        // POSTCONDITIONS:
        //   - The Vault MUST be removed from the `Vaults` mapping.
        //   - The Vault MUST remain above the secure collateralization threshold.
        //   - `get_total_nominated_collateral(vault_id)` must return zero.
        //   - For all nominators, `get_nominator_collateral(vault_id, user_id)` must return zero.
        //   - Staking pallet `nonce` must be incremented by one.
        //   - `compute_reward_at_index(nonce - 1, Token(IBTC), vault_id, user_id)` in the Staking pallet must be equal
        //     to the user’s nomination just before the vault opted out.
        test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
            assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
            assert_eq!(
                NominationPallet::get_total_nominated_collateral(&vault_id).unwrap(),
                default_nomination(&vault_id)
            );
            assert_eq!(
                NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap(),
                default_nomination(&vault_id)
            );
            assert_eq!(NominationPallet::is_opted_in(&vault_id), true);
            assert_ok!(nomination_opt_out(&vault_id));
            assert_eq!(NominationPallet::is_opted_in(&vault_id), false);
            assert_eq!(
                VaultRegistryPallet::get_collateralization_from_vault(vault_id.clone(), false).unwrap()
                    >= VaultRegistryPallet::secure_collateral_threshold(&vault_id.currencies).unwrap(),
                true
            );
            assert_eq!(
                NominationPallet::get_total_nominated_collateral(&vault_id).unwrap(),
                Amount::new(0, vault_id.collateral_currency())
            );
            assert_eq!(
                NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap(),
                Amount::new(0, vault_id.collateral_currency())
            );
            let nonce: u32 = VaultStakingPallet::nonce(&vault_id);
            assert_eq!(nonce, 1);
            assert_eq!(
                VaultStakingPallet::compute_stake_at_index(nonce - 1, &vault_id, &account_of(USER)).unwrap(),
                DEFAULT_NOMINATION as i128
            );
        })
    }

    #[test]
    fn integration_test_deposit_collateral_preconditions() {
        // PRECONDITIONS:
        //   - The global nomination flag MUST be enabled.
        //   - A Vault with id `vaultId` MUST be registered.
        //   - A Vault with id `vaultId` MUST exist in the `Vaults` mapping.
        //   - The Vault MUST remain below the max nomination ratio.
        test_with(|vault_id| {
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::deposit_collateral {
                    vault_id: vault_id.clone(),
                    amount: DEFAULT_BACKING_COLLATERAL
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::VaultNominationDisabled
            );
            enable_nomination();
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::deposit_collateral {
                    vault_id: vault_id.clone(),
                    amount: DEFAULT_BACKING_COLLATERAL
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::VaultNotOptedInToNomination
            );
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::deposit_collateral {
                    vault_id: vault_id.clone(),
                    amount: DEFAULT_BACKING_COLLATERAL
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::VaultNotOptedInToNomination
            );
            assert_nomination_opt_in(&vault_id);
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::deposit_collateral {
                    vault_id: vault_id.clone(),
                    amount: 100000000000000000000000
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::NominationExceedsLimit
            );
            assert_ok!(RuntimeCall::Nomination(NominationCall::deposit_collateral {
                vault_id: vault_id.clone(),
                amount: DEFAULT_NOMINATION
            })
            .dispatch(origin_of(account_of(USER))));
        })
    }

    #[test]
    fn integration_test_deposit_collateral_postconditions() {
        // POSTCONDITIONS:
        //   - The Vault’s collateral MUST increase by the amount nominated.
        //   - The Nominator’s balance MUST decrease by the amount nominated.
        test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
            let vault_backing_collateral_before_nomination =
                VaultRegistryPallet::get_backing_collateral(&vault_id).unwrap();
            let user_collateral_before_nomination =
                NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap();
            assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
            let vault_backing_collateral_after_nomination =
                VaultRegistryPallet::get_backing_collateral(&vault_id).unwrap();
            let user_collateral_after_nomination =
                NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap();
            assert_eq!(
                vault_backing_collateral_after_nomination,
                vault_backing_collateral_before_nomination + default_nomination(&vault_id)
            );
            assert_eq!(
                user_collateral_after_nomination,
                user_collateral_before_nomination + default_nomination(&vault_id)
            );
        })
    }

    #[test]
    fn integration_test_withdraw_collateral_preconditions() {
        // PRECONDITIONS:
        //   - The global nomination flag MUST be enabled.
        //   - A Vault with id vaultId MUST be registered.
        //   - A Vault with id vaultId MUST exist in the Vaults mapping.
        //   - Nominator MUST have nominated at least amount.
        test_with(|vault_id| {
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::withdraw_collateral {
                    vault_id: vault_id.clone(),
                    amount: Some(1),
                    index: None
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::VaultNominationDisabled
            );
            enable_nomination();
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::withdraw_collateral {
                    vault_id: vault_id_of(CAROL, vault_id.collateral_currency()),
                    amount: Some(1),
                    index: None
                })
                .dispatch(origin_of(account_of(USER))),
                VaultRegistryError::VaultNotFound
            );
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::withdraw_collateral {
                    vault_id: vault_id.clone(),
                    amount: Some(1),
                    index: None
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::VaultNotOptedInToNomination
            );
            assert_nomination_opt_in(&vault_id);
            assert_noop!(
                RuntimeCall::Nomination(NominationCall::withdraw_collateral {
                    vault_id: vault_id.clone(),
                    amount: Some(DEFAULT_BACKING_COLLATERAL),
                    index: None
                })
                .dispatch(origin_of(account_of(USER))),
                NominationError::CannotWithdrawCollateral
            );
        })
    }

    #[test]
    fn integration_test_withdraw_collateral_preconditions_collateralization() {
        // PRECONDITION: The Vault MUST remain above the secure collateralization threshold.
        test_with_nomination_enabled(|vault_id| {
            assert_ok!(RuntimeCall::Nomination(NominationCall::withdraw_collateral {
                vault_id: vault_id.clone(),
                index: None,
                amount: Some(750000)
            })
            .dispatch(origin_of(account_of(VAULT))));
            assert_nomination_opt_in(&vault_id);
            assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
            set_collateral_exchange_rate(&vault_id, FixedU128::checked_from_integer(3u128).unwrap());

            assert_noop!(
                withdraw_nominator_collateral(account_of(USER), &vault_id, default_nomination(&vault_id)),
                NominationError::CannotWithdrawCollateral
            );
        });
    }

    #[test]
    fn integration_test_withdraw_collateral_postconditions() {
        // POSTCONDITIONS:
        //   - The Vault’s collateral MUST decrease by the amount nominated.
        //   - The Nominator’s balance MUST increase by the amount nominated.
        test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
            assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
            let vault_backing_collateral_before_withdrawal =
                VaultRegistryPallet::get_backing_collateral(&vault_id).unwrap();
            let user_collateral_before_withdrawal =
                NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap();
            withdraw_nominator_collateral(account_of(USER), &vault_id, default_nomination(&vault_id)).unwrap();
            let vault_backing_collateral_after_withdrawal =
                VaultRegistryPallet::get_backing_collateral(&vault_id).unwrap();
            let user_collateral_after_withdrawal =
                NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap();
            assert_eq!(
                vault_backing_collateral_after_withdrawal,
                vault_backing_collateral_before_withdrawal - default_nomination(&vault_id)
            );
            assert_eq!(
                user_collateral_after_withdrawal,
                user_collateral_before_withdrawal - default_nomination(&vault_id)
            );
        });
    }
}

#[test]
fn integration_test_regular_vaults_are_not_opted_in_to_nomination() {
    test_with_nomination_enabled(|vault_id| {
        let new_vault_id = VaultId {
            account_id: account_of(CAROL),
            ..vault_id
        };
        register_vault(&new_vault_id, Amount::new(1000000, new_vault_id.collateral_currency()));
        assert_eq!(NominationPallet::is_opted_in(&new_vault_id), false);
    })
}

#[test]
fn integration_test_vaults_can_opt_in() {
    test_with_nomination_enabled(|vault_id| {
        assert_nomination_opt_in(&vault_id);
        assert_eq!(NominationPallet::is_opted_in(&vault_id), true);
    });
}

#[test]
fn integration_test_vaults_cannot_opt_in_if_disabled() {
    test_with(|vault_id| {
        assert_noop!(nomination_opt_in(&vault_id), NominationError::VaultNominationDisabled);
    });
}

#[test]
fn integration_test_vaults_can_still_opt_out_if_disabled() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        disable_nomination();
        assert_ok!(nomination_opt_out(&vault_id));
    });
}

#[test]
fn integration_test_cannot_nominate_if_not_opted_in() {
    test_with_nomination_enabled(|vault_id| {
        assert_noop!(
            RuntimeCall::Nomination(NominationCall::deposit_collateral {
                vault_id: vault_id,
                amount: DEFAULT_BACKING_COLLATERAL
            })
            .dispatch(origin_of(account_of(USER))),
            NominationError::VaultNotOptedInToNomination
        );
    });
}

#[test]
fn integration_test_can_nominate_if_opted_in() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        let nominator_collateral = get_nominator_collateral(&vault_id, account_of(USER));
        assert_eq!(nominator_collateral, default_nomination(&vault_id));
        assert_total_nominated_collateral_is(&vault_id, default_nomination(&vault_id));
    });
}

#[test]
fn integration_test_vaults_cannot_withdraw_nominated_collateral() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        assert_noop!(
            withdraw_vault_collateral(
                &vault_id,
                default_backing_collateral(vault_id.collateral_currency()).with_amount(|x| x + 1)
            ),
            NominationError::CannotWithdrawCollateral
        );
    });
}

#[test]
fn integration_test_nominated_collateral_cannot_exceed_nomination_limit() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_ok!(RuntimeCall::Nomination(NominationCall::deposit_collateral {
            vault_id: vault_id.clone(),
            amount: DEFAULT_NOMINATION_LIMIT - 100,
        })
        .dispatch(origin_of(account_of(USER))));
        assert_noop!(
            RuntimeCall::Nomination(NominationCall::deposit_collateral {
                vault_id: vault_id.clone(),
                amount: 101,
            })
            .dispatch(origin_of(account_of(CAROL))),
            NominationError::NominationExceedsLimit
        );
        assert_ok!(RuntimeCall::Nomination(NominationCall::deposit_collateral {
            vault_id: vault_id.clone(),
            amount: 100,
        })
        .dispatch(origin_of(account_of(CAROL))));
    });
}

#[test]
fn integration_test_nominated_collateral_prevents_replace_requests() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        assert_noop!(
            RuntimeCall::Replace(ReplaceCall::request_replace {
                currency_pair: vault_id.currencies.clone(),
                amount: 0,
            })
            .dispatch(origin_of(vault_id.account_id.clone())),
            ReplaceError::VaultHasEnabledNomination
        );
    });
}

#[test]
fn integration_test_vaults_with_zero_nomination_cannot_request_replacement() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        let amount = DEFAULT_VAULT_ISSUED - DEFAULT_VAULT_TO_BE_REDEEMED - DEFAULT_VAULT_TO_BE_REPLACED;
        assert_noop!(
            RuntimeCall::Replace(ReplaceCall::request_replace {
                currency_pair: vault_id.currencies.clone(),
                amount: amount.amount(),
            })
            .dispatch(origin_of(vault_id.account_id.clone())),
            ReplaceError::VaultHasEnabledNomination
        );
    });
}

#[test]
fn integration_test_nomination_increases_issuable_tokens() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        let issuance_capacity_before_nomination =
            VaultRegistryPallet::get_issuable_tokens_from_vault(&vault_id).unwrap();
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        let issuance_capacity_after_nomination =
            VaultRegistryPallet::get_issuable_tokens_from_vault(&vault_id).unwrap();
        assert!(issuance_capacity_after_nomination
            .gt(&issuance_capacity_before_nomination)
            .unwrap());
    });
}

#[test]
fn integration_test_nominator_withdrawal_request_reduces_issuable_tokens() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        let issuance_capacity_before_withdrawal_request =
            VaultRegistryPallet::get_issuable_tokens_from_vault(&vault_id).unwrap();
        assert_ok!(withdraw_nominator_collateral(
            account_of(USER),
            &vault_id,
            default_nomination(&vault_id)
        ));
        let issuance_capacity_after_withdrawal_request =
            VaultRegistryPallet::get_issuable_tokens_from_vault(&vault_id).unwrap();
        assert!(issuance_capacity_after_withdrawal_request
            .lt(&issuance_capacity_before_withdrawal_request)
            .unwrap());
    });
}

#[test]
fn integration_test_nominator_withdrawal_below_collateralization_threshold_fails() {
    test_with_nomination_enabled(|vault_id| {
        assert_ok!(RuntimeCall::Nomination(NominationCall::withdraw_collateral {
            vault_id: vault_id.clone(),
            index: None,
            amount: Some(750000)
        })
        .dispatch(origin_of(account_of(VAULT))));
        assert_nomination_opt_in(&vault_id);
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        set_collateral_exchange_rate(&vault_id, FixedU128::checked_from_integer(3u128).unwrap());
        assert_noop!(
            withdraw_nominator_collateral(account_of(USER), &vault_id, default_nomination(&vault_id)),
            NominationError::CannotWithdrawCollateral
        );
    });
}

#[test]
fn integration_test_nomination_fee_distribution() {
    test_with_nomination_enabled(|_| {});
}

#[test]
fn integration_test_vault_opt_out_must_refund_nomination() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        assert_eq!(
            NominationPallet::get_total_nominated_collateral(&vault_id).unwrap(),
            default_nomination(&vault_id)
        );
        assert_eq!(
            NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap(),
            default_nomination(&vault_id)
        );
        assert_ok!(nomination_opt_out(&vault_id));
        assert_eq!(
            NominationPallet::get_total_nominated_collateral(&vault_id).unwrap(),
            Amount::new(0, vault_id.collateral_currency())
        );
        assert_eq!(
            NominationPallet::get_nominator_collateral(&vault_id, &account_of(USER)).unwrap(),
            Amount::new(0, vault_id.collateral_currency())
        );
        let nonce: u32 = VaultStakingPallet::nonce(&vault_id);
        assert_eq!(nonce, 1);
        assert_eq!(
            VaultStakingPallet::compute_stake_at_index(nonce - 1, &vault_id, &account_of(USER)).unwrap(),
            DEFAULT_NOMINATION as i128
        );
    })
}

#[test]
fn integration_test_banning_a_vault_does_not_force_refund() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        VaultRegistryPallet::ban_vault(&vault_id).unwrap();
        let nonce: u32 = VaultStakingPallet::nonce(&vault_id);
        assert_eq!(nonce, 0);
    })
}

#[test]
fn integration_test_liquidating_a_vault_does_not_force_refund() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));
        VaultRegistryPallet::liquidate_vault(&vault_id).unwrap();
        let nonce: u32 = VaultStakingPallet::nonce(&vault_id);
        assert_eq!(nonce, 0);
    })
}

#[test]
fn integration_test_rewards_are_preserved_on_collateral_withdrawal() {
    test_with_nomination_enabled_and_vault_opted_in(|vault_id| {
        let mut user_data = default_user_state();
        (*user_data.balances.get_mut(&vault_id.collateral_currency()).unwrap()).free =
            default_user_free_balance(vault_id.collateral_currency()) + default_nomination(&vault_id);

        UserData::force_to(USER, user_data);
        assert_nominate_collateral(&vault_id, account_of(USER), default_nomination(&vault_id));

        let (issue_id, _) = issue_utils::request_issue(&vault_id, vault_id.wrapped(400000));
        issue_utils::execute_issue(issue_id);
        FeePallet::distribute_all_vault_rewards(&vault_id).unwrap();
        let reward_before_nomination_withdrawal =
            VaultStakingPallet::compute_reward(vault_id.wrapped_currency(), &vault_id, &account_of(USER)).unwrap();
        let reward_before_nomination_withdrawal2 =
            VaultStakingPallet::compute_reward(vault_id.wrapped_currency(), &vault_id, &account_of(USER)).unwrap();
        assert!(reward_before_nomination_withdrawal > 0);
        assert_eq!(
            reward_before_nomination_withdrawal,
            reward_before_nomination_withdrawal2
        );
        assert_ok!(withdraw_nominator_collateral(
            account_of(USER),
            &vault_id,
            default_nomination(&vault_id)
        ));
        assert_eq!(
            VaultStakingPallet::compute_reward(vault_id.wrapped_currency(), &vault_id, &account_of(USER)).unwrap(),
            reward_before_nomination_withdrawal
        );
    })
}
