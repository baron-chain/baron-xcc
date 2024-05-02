use crate::{
    setup::{assert_eq, *},
    utils::{loans_utils::activate_lending_and_mint, redeem_utils::*},
};
use currency::Amount;
use std::str::FromStr;

fn test_with<R>(execute: impl Fn(VaultId) -> R) {
    let test_with = |collateral_id, wrapped_id, extra_vault_currency| {
        ExtBuilder::build().execute_with(|| {
            let vault_id = PrimitiveVaultId::new(account_of(VAULT), collateral_id, wrapped_id);
            common_setup::<R>(
                wrapped_id,
                extra_vault_currency,
                collateral_id,
                vault_id.clone(),
                FixedU128::one(),
                None,
            );
            execute(vault_id)
        })
    };

    test_with(Token(DOT), Token(KBTC), None);
    test_with(Token(DOT), Token(IBTC), None);
    test_with(Token(DOT), Token(IBTC), Some(Token(KSM)));
    test_with(Token(KSM), Token(IBTC), None);
    test_with(ForeignAsset(1), Token(IBTC), None);
    test_with(LendToken(1), Token(IBTC), None);
}

fn test_setup_for_premium_redeem<R>(execute: impl Fn(VaultId) -> R) {
    let test_with = |collateral_id, wrapped_id, extra_vault_currency: Option<CurrencyId>| {
        ExtBuilder::build().execute_with(|| {
            let secure = FixedU128::checked_from_rational(200, 100).unwrap();
            let premium = FixedU128::checked_from_rational(160, 100).unwrap();
            let liquidation = FixedU128::checked_from_rational(110, 100).unwrap();

            let vault_id = PrimitiveVaultId::new(account_of(VAULT), collateral_id, wrapped_id);
            common_setup::<R>(
                wrapped_id,
                extra_vault_currency,
                collateral_id,
                vault_id.clone(),
                FixedU128::from(2),
                Some((secure, premium, liquidation)),
            );
            execute(vault_id)
        })
    };

    test_with(Token(DOT), Token(KBTC), None);
    test_with(Token(DOT), Token(IBTC), None);
    test_with(Token(DOT), Token(IBTC), Some(Token(KSM)));
    test_with(Token(KSM), Token(IBTC), None);
    test_with(ForeignAsset(1), Token(IBTC), None);
    test_with(LendToken(1), Token(IBTC), None);
}

fn common_setup<R>(
    wrapped_id: CurrencyId,
    extra_vault_currency: Option<CurrencyId>,
    collateral_id: CurrencyId,
    vault_id: VaultId,
    exchange_rate: FixedU128,
    custom_thresholds: Option<(FixedU128, FixedU128, FixedU128)>,
) {
    SecurityPallet::set_active_block_number(1);
    for currency_id in iter_collateral_currencies().filter(|c| !c.is_lend_token()) {
        assert_ok!(OraclePallet::_set_exchange_rate(currency_id, exchange_rate));
    }
    if wrapped_id != DEFAULT_WRAPPED_CURRENCY {
        assert_ok!(OraclePallet::_set_exchange_rate(wrapped_id, FixedU128::one()));
    }

    activate_lending_and_mint(Token(DOT), LendToken(1));

    if let Some((secure, premium, liquidation)) = custom_thresholds {
        set_custom_thresholds(secure, premium, liquidation);
    } else {
        set_default_thresholds();
    }

    LiquidationVaultData::force_to(default_liquidation_vault_state(&vault_id.currencies));
    UserData::force_to(USER, default_user_state());
    CoreVaultData::force_to(&vault_id, default_vault_state(&vault_id));
    // additional vault in order to prevent the edge case where the fee pool does not
    // get additional funds because there are no non-liquidated vaults left
    let carol_vault_id = PrimitiveVaultId::new(account_of(CAROL), collateral_id, wrapped_id);
    CoreVaultData::force_to(&carol_vault_id, default_vault_state(&carol_vault_id));

    if let Some(other_currency) = extra_vault_currency {
        assert_ok!(OraclePallet::_set_exchange_rate(other_currency, FixedU128::one()));
        // check that having other vault with the same account id does not influence tests
        let other_vault_id = vault_id_of(VAULT, other_currency);
        CoreVaultData::force_to(&other_vault_id, default_vault_state(&other_vault_id));
    }
}

/// to-be-replaced & replace_collateral are decreased in request_redeem
fn consume_to_be_replaced(vault: &mut CoreVaultData, amount_btc: Amount<Runtime>) {
    let to_be_replaced_decrease = amount_btc.min(&vault.to_be_replaced).unwrap();
    let released_replace_collateral = griefing(
        (vault.replace_collateral.amount() * to_be_replaced_decrease.amount()) / vault.to_be_replaced.amount(),
    );

    vault.replace_collateral -= released_replace_collateral;
    vault.griefing_collateral -= released_replace_collateral;
    *vault.free_balance.get_mut(&DEFAULT_GRIEFING_CURRENCY).unwrap() += released_replace_collateral;

    vault.to_be_replaced -= to_be_replaced_decrease;
}

mod premium_redeem_tests {
    use super::{assert_eq, *};

    fn setup_vault_below_premium_threshold(vault_id: VaultId) {
        // with 2000 collateral and exchange rate at 2, the vault is at:
        //     - secure threshold (200%) when it has 2000/2/2 = 500 tokens
        //     - premium threshold (160%) when it has 2000/2/1.6 = 625 tokens

        // we award premium redeem only for the amount needed for (issued + to_be_issued - to_be_redeemed)
        // to reach the secure threshold

        // setup the vault such that (issued + to_be_issued - to_be_redeemed) = (450 + 250 - 50) = 650
        // (everything scaled by 1000 to prevent getting dust amount errors)
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                issued: vault_id.wrapped(450_000_000),
                to_be_issued: vault_id.wrapped(250_000_000),
                to_be_redeemed: vault_id.wrapped(50_000_000),
                backing_collateral: vault_id.collateral(2_000_000_000),
                to_be_replaced: vault_id.wrapped(0),
                replace_collateral: griefing(0),
                ..default_vault_state(&vault_id)
            },
        );

        // make sure user has enough tokens to redeem
        let mut user_state = UserData::get(USER);
        (*user_state.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free =
            (*user_state.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free * 1000;
        UserData::force_to(USER, user_state);
    }

    #[test]
    fn integration_test_premium_redeem_with_reward_for_only_part_of_the_request() {
        test_setup_for_premium_redeem(|vault_id| {
            setup_vault_below_premium_threshold(vault_id.clone());

            assert!(!VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());

            let compute_collateral = VaultRegistryPallet::compute_collateral(&vault_id).unwrap().amount();
            assert_eq!(compute_collateral, 2_000_000_000);

            let initial_state = ParachainState::get(&vault_id);

            let redeem_id = setup_redeem(vault_id.wrapped(400_000_000), USER, &vault_id);
            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

            assert!(!VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(!VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());

            dry_run(|| {
                // further redeems will have no rewards, even though the premium redeem
                // has not executed yet
                let redeem_id = setup_redeem(vault_id.wrapped(2_000_000), USER, &vault_id);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                assert_eq!(redeem.premium, 0);
            });

            execute_redeem(redeem_id);

            assert_eq!(
                ParachainState::get(&vault_id),
                initial_state.with_changes(|user, vault, _, fee_pool| {
                    // premium transferred to user
                    // we should get rewarded only for 15.3846153846 *10^6 tokens (that's when we reach nearer to secure
                    // threshold)
                    let expected_premium = vault_id.collateral(15_384_615);
                    vault.backing_collateral -= expected_premium;
                    (*user.balances.get_mut(&vault_id.collateral_currency()).unwrap()).free += expected_premium;

                    // bitcoin balance update as usual
                    (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -=
                        redeem.amount_btc() + redeem.fee() + redeem.transfer_fee_btc();
                    vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();
                    *fee_pool.rewards_for(&vault_id) += redeem.fee();
                })
            );

            // We already checked that redeems have no more rewards after requesting the
            // premium redeem. Here we do a sanity check that it's still the case after
            // execution
            let redeem_id = setup_redeem(vault_id.wrapped(2_000_000), USER, &vault_id);
            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
            assert_eq!(redeem.premium, 0);
        });
    }

    #[test]
    fn integration_test_try_get_premium_vaults_which_is_sufficiently_collateralized_then_under_collateralized() {
        test_setup_for_premium_redeem(|vault_id| {
            assert_noop!(
                RedeemPallet::get_premium_redeem_vaults(),
                VaultRegistryError::NoVaultUnderThePremiumRedeemThreshold
            );

            // put vault under premium redeem threshold
            setup_vault_below_premium_threshold(vault_id.clone());
            assert_eq!(RedeemPallet::get_premium_redeem_vaults().unwrap().len(), 1);
        });
    }

    #[test]
    fn integration_test_redeem_max_premium_redeemable_token() {
        test_setup_for_premium_redeem(|vault_id| {
            setup_vault_below_premium_threshold(vault_id.clone());

            let global_secure = VaultRegistryPallet::get_global_secure_threshold(&vault_id.currencies).unwrap(); // 200%

            // secure > premium > liquidation threshold
            // at start the vault is above the custom&global secure threshold, but due to the to_be_issued
            // tokens it is already eligible for premium redeem
            assert!(!VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(!VaultRegistryPallet::is_vault_below_certain_threshold(&vault_id, global_secure).unwrap());
            assert!(VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());

            // Change vault secure threshold,
            // now custom secure > global secure > premium > liquidation threshold
            let vault_custom_secure_threshold = UnsignedFixedPoint::checked_from_rational(300, 100);
            assert_ok!(
                RuntimeCall::VaultRegistry(VaultRegistryCall::set_custom_secure_threshold {
                    currency_pair: vault_id.currencies.clone(),
                    custom_threshold: vault_custom_secure_threshold,
                })
                .dispatch(origin_of(vault_id.account_id.clone()))
            );

            // vault should be below premium & secure threshold, while above global secure threshold
            assert!(VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(!VaultRegistryPallet::is_vault_below_certain_threshold(&vault_id, global_secure).unwrap());
            assert!(VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());

            let max_premium_for_vault = VaultRegistryPallet::get_vault_max_premium_redeem(&vault_id).unwrap();
            // get premium redeem vaults
            let premium_redeem_vaults = RedeemPallet::get_premium_redeem_vaults().unwrap()[0].clone();
            // non-zero amount of tokens that are elible for premium redeem
            assert!(!premium_redeem_vaults.1.is_zero());

            // request redeem tokens given by RPC
            let redeem_id_1 = setup_redeem(premium_redeem_vaults.1, USER, &vault_id);

            let redeem_1 = RedeemPallet::get_open_redeem_request_from_id(&redeem_id_1).unwrap();
            // premium should be equal to max premium, but allow rounding error in this check.
            assert!(
                redeem_1.premium >= max_premium_for_vault.amount() - 1
                    && redeem_1.premium <= max_premium_for_vault.amount() + 1
            );
            assert!(!redeem_1.premium.is_zero());

            // max premium for vault should be zero
            let max_premium_for_vault = VaultRegistryPallet::get_vault_max_premium_redeem(&vault_id).unwrap();
            assert!(max_premium_for_vault.amount().is_zero());

            // redeeming the max premium amount put backs vault above premium threshold
            // vault should be below secure threshold, while above global secure & premium threshold
            assert!(VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(!VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());
            assert!(!VaultRegistryPallet::is_vault_below_certain_threshold(&vault_id, global_secure).unwrap());

            execute_redeem(redeem_id_1);
            // We should be almost exactly at the secure threshold (there should only be minor
            // rounding errors)
            let vault = CoreVaultData::vault(vault_id.clone());
            let future_tokens = vault.to_be_issued + vault.issued - vault.to_be_redeemed;
            let collateral = vault.backing_collateral;
            let future_ratio = collateral
                .ratio(&future_tokens.convert_to(vault_id.collateral_currency()).unwrap())
                .unwrap();
            // actual collateralization rate: 2.000004822104648639. Allow small rounding changes
            assert!(future_ratio - global_secure < FixedU128::from_float(0.00001));

            let redeem_id_2 = setup_redeem(vault_id.wrapped(800_00), USER, &vault_id);
            let redeem_2 = RedeemPallet::get_open_redeem_request_from_id(&redeem_id_2).unwrap();
            // no premium is given out for new redeems
            assert!(redeem_2.premium.is_zero());
        });
    }
    #[test]
    fn integration_test_premium_redeem_with_reward_for_full_request() {
        test_setup_for_premium_redeem(|vault_id| {
            setup_vault_below_premium_threshold(vault_id.clone());

            assert!(!VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());

            let redeem_id = setup_redeem(vault_id.wrapped(100_000_000), USER, &vault_id);

            assert!(!VaultRegistryPallet::is_vault_below_secure_threshold(&vault_id).unwrap());
            assert!(!VaultRegistryPallet::will_be_below_premium_threshold(&vault_id).unwrap());

            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

            // we should get rewarded for the full amount, since we did not reach secure threshold
            let expected_premium = FeePallet::get_premium_redeem_fee(
                &vault_id
                    .wrapped(redeem.amount_btc)
                    .convert_to(vault_id.collateral_currency())
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(vault_id.collateral(redeem.premium), expected_premium);
        });
    }
}

mod spec_based_tests {
    use primitives::VaultCurrencyPair;

    use super::{assert_eq, *};

    mod request_redeem {
        use frame_support::assert_ok;
        use sp_runtime::FixedU128;

        use super::{assert_eq, *};

        fn calculate_vault_capacity(vault_id: &VaultId) -> Amount<Runtime> {
            let redeemable_tokens = DEFAULT_VAULT_ISSUED - DEFAULT_VAULT_TO_BE_REDEEMED;
            let redeemable_tokens = vault_id.wrapped(redeemable_tokens.amount());

            // we are able to redeem `redeemable_tokens`. However, when requesting a redeem,
            // the fee is subtracted for this amount. As such, a user is able to request more
            // than `redeemable_tokens`. A first approximation of the limit is redeemable_tokens+fee,
            // however, this slightly underestimates it. Since the actual fee rate is not exposed,
            // use an iterative process to find the maximum redeem request amount.
            let mut ret = redeemable_tokens + FeePallet::get_redeem_fee(&redeemable_tokens).unwrap();

            loop {
                let actually_redeemed_tokens = ret - FeePallet::get_redeem_fee(&ret).unwrap();
                if actually_redeemed_tokens > redeemable_tokens {
                    return ret.with_amount(|x| x - 1);
                }
                ret = ret.with_amount(|x| x + 1);
            }
        }

        #[test]
        fn integration_test_request_redeem_at_capacity_succeeds() {
            // PRECONDITION: The vault’s `issuedTokens` MUST be at least `vault.toBeRedeemedTokens +
            // burnedTokens`
            test_with(|vault_id| {
                let amount = calculate_vault_capacity(&vault_id);
                assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
                    amount_wrapped: amount.amount(),
                    btc_address: BtcAddress::random(),
                    vault_id: vault_id.clone()
                })
                .dispatch(origin_of(account_of(USER))));

                let redeem_id = assert_redeem_request_event();
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

                assert_eq!(amount, redeem.fee() + redeem.amount_btc() + redeem.transfer_fee_btc());
                assert_eq!(redeem.vault, vault_id.clone());

                assert_eq!(
                    ParachainState::get(&vault_id),
                    ParachainState::get_default(&vault_id).with_changes(|user, vault, _, _| {
                        vault.to_be_redeemed += redeem.amount_btc() + redeem.transfer_fee_btc();
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -=
                            redeem.amount_btc() + redeem.transfer_fee_btc() + redeem.fee();
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked +=
                            redeem.amount_btc() + redeem.transfer_fee_btc() + redeem.fee();
                        consume_to_be_replaced(vault, redeem.amount_btc());
                    })
                );
            });
        }

        #[test]
        fn integration_test_request_redeem_above_capacity_fails() {
            // PRECONDITION: The vault’s `issuedTokens` MUST be at least `vault.toBeRedeemedTokens +
            // burnedTokens`
            test_with(|vault_id| {
                let amount = calculate_vault_capacity(&vault_id).amount() + 1;
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::request_redeem {
                        amount_wrapped: amount,
                        btc_address: BtcAddress::random(),
                        vault_id: vault_id.clone()
                    })
                    .dispatch(origin_of(account_of(USER))),
                    VaultRegistryError::InsufficientTokensCommitted
                );
            });
        }

        #[test]
        fn integration_test_redeem_cannot_request_from_liquidated_vault() {
            // PRECONDITION: The selected vault MUST NOT be liquidated.
            test_with(|vault_id| {
                liquidate_vault(&vault_id);
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::request_redeem {
                        amount_wrapped: 1500,
                        btc_address: BtcAddress::random(),
                        vault_id: vault_id.clone(),
                    })
                    .dispatch(origin_of(account_of(ALICE))),
                    VaultRegistryError::VaultLiquidated,
                );
            });
        }

        #[test]
        fn integration_test_redeem_redeemer_free_tokens() {
            // PRECONDITION: The redeemer MUST have at least `amountWrapped` free tokens.
            test_with(|vault_id| {
                let free_tokens_to_redeem = vault_id.wrapped(1500);
                let mut good_state = default_user_state();
                (*good_state.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free = free_tokens_to_redeem;
                UserData::force_to(ALICE, good_state);
                assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
                    amount_wrapped: free_tokens_to_redeem.amount(),
                    btc_address: BtcAddress::random(),
                    vault_id: vault_id.clone(),
                })
                .dispatch(origin_of(account_of(ALICE))));

                let mut bad_state = default_user_state();
                (*bad_state.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free =
                    free_tokens_to_redeem.with_amount(|x| x - 1);

                UserData::force_to(ALICE, bad_state);
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::request_redeem {
                        amount_wrapped: free_tokens_to_redeem.amount(),
                        btc_address: BtcAddress::random(),
                        vault_id: vault_id.clone(),
                    })
                    .dispatch(origin_of(account_of(ALICE))),
                    RedeemError::AmountExceedsUserBalance,
                );
            });
        }

        #[test]
        fn integration_test_redeem_vault_capacity_sufficient() {
            // PRECONDITION: The vault’s `issuedTokens` MUST be at least `vault.toBeRedeemedTokens +
            // burnedTokens`.
            // POSTCONDITIONS:
            //  - The vault’s `toBeRedeemedTokens` MUST increase by `burnedTokens`.
            //  - `amountWrapped` of the redeemer’s tokens MUST be locked by this transaction.
            //  - If the vault’s collateralization rate is above the PremiumRedeemThreshold, then `redeem.premium()`
            //    MUST be 0
            test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let vault_to_be_redeemed = vault_id.wrapped(1500);
                let user_to_redeem = vault_id.wrapped(1500);
                set_redeem_state(vault_to_be_redeemed, user_to_redeem, USER, &vault_id);
                let redeem_fee = FeePallet::get_redeem_fee(&user_to_redeem).unwrap();
                let burned_tokens = user_to_redeem - redeem_fee;

                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        backing_collateral: default_vault_backing_collateral(currency_id),
                        ..CoreVaultData::vault(vault_id.clone())
                    },
                );
                let parachain_state_before_request = ParachainState::get(&vault_id);
                let redeem_id = setup_redeem(user_to_redeem, USER, &vault_id);
                let actual_redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                assert_eq!(actual_redeem, default_redeem_request(user_to_redeem, &vault_id, USER));
                assert_eq!(
                    ParachainState::get(&vault_id),
                    parachain_state_before_request.with_changes(|user, vault, _liquidation_vault, _fee_pool| {
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked += user_to_redeem;
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= user_to_redeem;
                        vault.to_be_redeemed += burned_tokens;
                    })
                );
            });
        }

        #[test]
        fn integration_test_redeem_with_premium() {
            // PRECONDITION: The vault’s `issuedTokens` MUST be at least `vault.toBeRedeemedTokens +
            // burnedTokens`.
            // POSTCONDITIONS:
            //  - The vault’s `toBeRedeemedTokens` MUST increase by `burnedTokens`.
            //  - `amountWrapped` of the redeemer’s tokens MUST be locked by this transaction.
            //  - If the vault’s collateralization rate is below the PremiumRedeemThreshold, then `redeem.premium()`
            //    MUST be
            // PremiumRedeemFee multiplied by the worth of `redeem.amountBtc`
            test_with(|vault_id| {
                let vault_to_be_redeemed = vault_id.wrapped(1500);
                let user_to_redeem = vault_id.wrapped(1500);
                set_redeem_state(vault_to_be_redeemed, user_to_redeem, USER, &vault_id);
                setup_redeem(user_to_redeem, USER, &vault_id);
                let redeem_id = assert_redeem_request_event();
                let actual_redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                assert_eq!(actual_redeem, premium_redeem_request(user_to_redeem, &vault_id, USER));
            });
        }

        #[test]
        fn integration_test_redeem_vault_capacity_insufficient() {
            // PRECONDITION: The vault’s `issuedTokens` MUST be at least `vault.toBeRedeemedTokens +
            // burnedTokens`.
            test_with(|vault_id| {
                let vault_to_be_redeemed = vault_id.wrapped(1500);
                let user_to_redeem = vault_id.wrapped(1500);
                set_redeem_state(vault_to_be_redeemed, user_to_redeem, USER, &vault_id);
                let core_vault = CoreVaultData::vault(vault_id.clone());
                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        issued: core_vault.issued.with_amount(|x| x - 1),
                        ..core_vault
                    },
                );
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::request_redeem {
                        amount_wrapped: user_to_redeem.amount(),
                        btc_address: BtcAddress::random(),
                        vault_id: vault_id.clone(),
                    })
                    .dispatch(origin_of(account_of(ALICE))),
                    VaultRegistryError::InsufficientTokensCommitted
                );
            });
        }

        #[test]
        fn integration_test_redeem_dust_value() {
            // PRECONDITION: `burnedTokens` minus the inclusion fee MUST be above the RedeemBtcDustValue,
            // where the inclusion fee is the multiplication of RedeemTransactionSize and the fee rate estimate
            // reported by the oracle.

            test_with(|vault_id| {
                // The formula for finding the threshold `to_redeem` for the dust amount error is
                // `(redeem_dust_value + inclusion_fee) / (1 - redeem_fee_rate)`
                let redeem_dust_value = RedeemPallet::get_dust_value(vault_id.wrapped_currency());
                let inclusion_fee = RedeemPallet::get_current_inclusion_fee(vault_id.wrapped_currency()).unwrap();
                let redeem_fee_rate = FeePallet::redeem_fee();
                let denominator = FixedU128::one() - redeem_fee_rate;
                let numerator = FixedU128::from_inner((redeem_dust_value + inclusion_fee).amount());
                let to_redeem = vault_id.wrapped((numerator / denominator).into_inner());
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::request_redeem {
                        amount_wrapped: to_redeem.amount() - 1,
                        btc_address: BtcAddress::random(),
                        vault_id: vault_id.clone(),
                    })
                    .dispatch(origin_of(account_of(ALICE))),
                    RedeemError::AmountBelowDustAmount
                );
                assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
                    amount_wrapped: to_redeem.amount(),
                    btc_address: BtcAddress::random(),
                    vault_id: vault_id.clone(),
                })
                .dispatch(origin_of(account_of(ALICE))));
            });
        }

        #[test]
        fn integration_test_liquidating_one_collateral_currency_does_not_impact_other_currencies() {
            test_with(|vault_id| {
                let amount_btc = vault_id.wrapped(10000);

                let different_collateral = match vault_id.currencies.collateral {
                    Token(KSM) => Token(DOT),
                    _ => Token(KSM),
                };
                assert_ok!(OraclePallet::_set_exchange_rate(different_collateral, FixedU128::one()));

                let different_collateral_vault_id = PrimitiveVaultId::new(
                    vault_id.account_id.clone(),
                    different_collateral.clone(),
                    vault_id.currencies.wrapped.clone(),
                );
                CoreVaultData::force_to(
                    &different_collateral_vault_id,
                    default_vault_state(&different_collateral_vault_id),
                );

                liquidate_vault(&vault_id);
                assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
                    amount_wrapped: amount_btc.amount(),
                    btc_address: BtcAddress::random(),
                    vault_id: different_collateral_vault_id.clone(),
                })
                .dispatch(origin_of(account_of(ALICE))));
            });
        }
    }

    mod liquidation_redeem {
        use super::{assert_eq, *};
        #[test]
        fn integration_test_liquidation_redeem() {
            // PRECONDITION: The redeemer MUST have at least `amountWrapped` free tokens.
            // POSTCONDITION: `amountWrapped` tokens MUST be burned from the user.
            test_with(|vault_id| {
                let free_tokens_to_redeem = vault_id.wrapped(1500);
                set_redeem_state(vault_id.wrapped(0), free_tokens_to_redeem, USER, &vault_id);
                liquidate_vault(&vault_id);
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
                        currencies: vault_id.currencies.clone(),
                        amount_wrapped: free_tokens_to_redeem.amount() + 1,
                    })
                    .dispatch(origin_of(account_of(ALICE))),
                    RedeemError::AmountExceedsUserBalance,
                );
                let user_tokens_before_redeem =
                    currency::get_free_balance::<Runtime>(vault_id.wrapped_currency(), &account_of(USER));
                let tokens_to_liquidation_redeem = free_tokens_to_redeem.with_amount(|x| x - 10);
                assert_ok!(RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
                    currencies: vault_id.currencies.clone(),
                    amount_wrapped: free_tokens_to_redeem.amount() - 10,
                })
                .dispatch(origin_of(account_of(ALICE))));
                let user_tokens_after_redeem =
                    currency::get_free_balance::<Runtime>(vault_id.wrapped_currency(), &account_of(USER));

                assert_eq!(
                    user_tokens_before_redeem - tokens_to_liquidation_redeem,
                    user_tokens_after_redeem
                )
            });
        }
    }

    mod execute_redeem {
        use redeem::RedeemRequestStatus;

        use super::{assert_eq, *};
        #[test]
        fn integration_test_redeem_wrapped_execute() {
            // PRECONDITIONS:
            // - A pending `RedeemRequest` MUST exist with an id equal to `redeemId`.
            // - The `rawTx` MUST decode to a valid transaction that transfers exactly the amount specified in the
            // `RedeemRequest` struct. It MUST be a transaction to the correct address, and provide the expected
            // OP_RETURN, based on the `RedeemRequest`.
            // - The `merkleProof` MUST contain a valid proof of of `rawTX`.
            // - The bitcoin payment MUST have been submitted to the relay chain, and MUST have sufficient
            //   confirmations.
            // POSTCONDITIONS:
            // - The user's `lockedTokens` MUST decrease by `redeemRequest.amountBtc + redeemRequest.transferFeeBtc`.
            // - The vault’s `toBeRedeemedTokens` MUST decrease by `redeemRequest.amountBtc +
            //   redeemRequest.transferFeeBtc`.
            // - The vault’s `issuedTokens` MUST decrease by `redeemRequest.amountBtc + redeemRequest.transferFeeBtc`.
            // - `redeemRequest.fee` MUST be unlocked and transferred from the redeemer’s account to the fee pool.
            // - `redeemRequest.status` MUST be set to `Completed`.
            test_with(|vault_id| {
                let issued_tokens = vault_id.wrapped(10_000);

                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::execute_redeem {
                        redeem_id: H256::random(),
                        unchecked_transaction: dummy_tx()
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    RedeemError::RedeemIdNotFound
                );
                let redeem_id = setup_redeem(issued_tokens, USER, &vault_id);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                let user_btc_address = BtcAddress::P2PKH(H160([2; 20]));

                // The `rawTx` MUST decode to a valid transaction that transfers exactly the amount specified in the
                // `RedeemRequest` struct.
                let mut current_block = assert_redeem_error(
                    redeem_id,
                    user_btc_address,
                    redeem.amount_btc().with_amount(|x| x - 1),
                    redeem_id,
                    1,
                    BTCRelayError::InvalidPaymentAmount,
                );
                current_block = assert_redeem_error(
                    redeem_id,
                    user_btc_address,
                    redeem.amount_btc().with_amount(|x| x + 1),
                    redeem_id,
                    current_block,
                    BTCRelayError::InvalidPaymentAmount,
                );

                // The `rawTx` MUST decode to a valid transaction, to the correct address
                current_block = assert_redeem_error(
                    redeem_id,
                    BtcAddress::P2PKH(H160([3; 20])),
                    redeem.amount_btc(),
                    redeem_id,
                    current_block,
                    BTCRelayError::InvalidPayment,
                );

                // The bitcoin payment MUST have been submitted to the relay chain, and MUST have sufficient
                // confirmations.
                assert_redeem_error(
                    redeem_id,
                    user_btc_address,
                    redeem.amount_btc(),
                    redeem_id,
                    0,
                    BTCRelayError::ParachainConfirmations,
                );

                // The `rawTx` MUST decode to a valid transaction and provide the expected OP_RETURN
                assert_redeem_error(
                    redeem_id,
                    user_btc_address,
                    redeem.amount_btc(),
                    H256::random(),
                    current_block,
                    BTCRelayError::InvalidPayment,
                );

                // The `merkleProof` MUST contain a valid proof of of `rawTX`
                let (_tx_id, _tx_block_height, mut transaction) = generate_transaction_and_mine(
                    Default::default(),
                    vec![],
                    vec![(user_btc_address, redeem.amount_btc())],
                    vec![redeem_id],
                );
                let invalid_merkle_proof = hex::decode("00000020b0b3d77b97015b519553423c96642b33ca534c50ecefd133640000000000000029a0a725684aeca24af83e3ba0a3e3ee56adfdf032d19e5acba6d0a262e1580ca354915fd4c8001ac42a7b3a1000000005df41db041b26536b5b7fd7aeea4ea6bdb64f7039e4a566b1fa138a07ed2d3705932955c94ee4755abec003054128b10e0fbcf8dedbbc6236e23286843f1f82a018dc7f5f6fba31aa618fab4acad7df5a5046b6383595798758d30d68c731a14043a50d7cb8560d771fad70c5e52f6d7df26df13ca457655afca2cbab2e3b135c0383525b28fca31296c809641205962eb353fb88a9f3602e98a93b1e9ffd469b023d00").unwrap();
                transaction.user_tx_proof.merkle_proof = MerkleProof::parse(&invalid_merkle_proof).unwrap();
                transaction.user_tx_proof.merkle_proof = MerkleProof::parse(&invalid_merkle_proof).unwrap();
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::execute_redeem {
                        redeem_id: redeem_id,
                        unchecked_transaction: transaction
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    BTCRelayError::InvalidTxid
                );
                let parachain_state_before_execution = ParachainState::get(&vault_id);
                execute_redeem(redeem_id);

                // `redeemRequest.amountBtc - redeemRequest.transferFeeBtc` of the tokens in the redeemer’s account MUST
                // be burned. `redeemRequest.fee` MUST be unlocked and transferred from the redeemer’s
                // account to the fee pool.
                assert_eq!(
                    ParachainState::get(&vault_id),
                    parachain_state_before_execution.with_changes(|user, vault, _, fee_pool| {
                        vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();
                        vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();

                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -=
                            redeem.amount_btc() + redeem.transfer_fee_btc() + redeem.fee();

                        *fee_pool.rewards_for(&vault_id) += redeem.fee();
                    })
                );
                // `redeemRequest.status` MUST be set to `Completed`.
                let completed_redeem = RedeemPallet::get_open_or_completed_redeem_request_from_id(&redeem_id).unwrap();
                assert_eq!(completed_redeem.status, RedeemRequestStatus::Completed);
            });
        }
    }

    mod cancel_redeem {
        use redeem::RedeemRequestStatus;

        use super::{assert_eq, *};

        fn set_redeem_period(period: u32) {
            assert_ok!(RuntimeCall::Redeem(RedeemCall::set_redeem_period { period }).dispatch(root()));
        }

        fn request_redeem(vault_id: &VaultId) -> H256 {
            assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
                amount_wrapped: 4_000,
                btc_address: BtcAddress::random(),
                vault_id: vault_id.clone()
            })
            .dispatch(origin_of(account_of(USER))));
            // get the redeem id
            assert_redeem_request_event()
        }

        fn execute_redeem(redeem_id: H256) -> DispatchResultWithPostInfo {
            ExecuteRedeemBuilder::new(redeem_id).execute()
        }

        fn cancel_redeem(redeem_id: H256) -> DispatchResultWithPostInfo {
            RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                redeem_id: redeem_id,
                reimburse: true,
            })
            .dispatch(origin_of(account_of(USER)))
        }

        #[test]
        fn integration_test_redeem_expiry_only_parachain_blocks_expired() {
            test_with(|vault_id| {
                set_redeem_period(1000);
                let redeem_id = request_redeem(&vault_id);
                mine_blocks(1);
                SecurityPallet::set_active_block_number(10000);

                assert_noop!(cancel_redeem(H256::random()), RedeemError::RedeemIdNotFound);
                // request still uses period = 200, so cancel fails and execute succeeds
                assert_noop!(cancel_redeem(redeem_id), RedeemError::TimeNotExpired);
                assert_ok!(execute_redeem(redeem_id));
            });
        }

        #[test]
        fn integration_test_redeem_expiry_only_bitcoin_blocks_expired() {
            test_with(|vault_id| {
                set_redeem_period(1000);
                let redeem_id = request_redeem(&vault_id);
                SecurityPallet::set_active_block_number(100);
                mine_blocks(20);

                // request still uses period = 200, so cancel fails and execute succeeds
                assert_noop!(cancel_redeem(redeem_id), RedeemError::TimeNotExpired);
                assert_ok!(execute_redeem(redeem_id));
            });
        }

        #[test]
        fn integration_test_redeem_expiry_no_period_change_pre_expiry() {
            test_with(|vault_id| {
                set_redeem_period(1000);
                let redeem_id = request_redeem(&vault_id);
                SecurityPallet::set_active_block_number(750);
                mine_blocks(1);

                assert_noop!(cancel_redeem(redeem_id), RedeemError::TimeNotExpired);
                assert_ok!(execute_redeem(redeem_id));
            });
        }

        #[test]
        fn integration_test_redeem_expiry_no_period_change_post_expiry() {
            // PRECONDITION: The request MUST be expired.

            // can still execute after expiry
            test_with(|vault_id| {
                set_redeem_period(1000);
                let redeem_id = request_redeem(&vault_id);
                mine_blocks(100);
                SecurityPallet::set_active_block_number(1100);
                assert_ok!(execute_redeem(redeem_id));
            });

            // .. but user can also cancel. Whoever is first wins
            test_with(|vault_id| {
                set_redeem_period(1000);
                let redeem_id = request_redeem(&vault_id);
                mine_blocks(100);
                SecurityPallet::set_active_block_number(1100);
                assert_ok!(cancel_redeem(redeem_id));
            });
        }

        #[test]
        fn integration_test_redeem_expiry_with_period_decrease() {
            // PRECONDITION: The request MUST be expired.
            test_with(|vault_id| {
                set_redeem_period(2000);
                let redeem_id = request_redeem(&vault_id);
                SecurityPallet::set_active_block_number(1100);
                mine_blocks(12);
                set_redeem_period(1000);

                // request still uses period = 200, so cancel fails and execute succeeds
                assert_noop!(cancel_redeem(redeem_id), RedeemError::TimeNotExpired);
                assert_ok!(execute_redeem(redeem_id));
            });
        }

        #[test]
        fn integration_test_redeem_expiry_with_period_increase() {
            test_with(|vault_id| {
                set_redeem_period(100);
                let redeem_id = request_redeem(&vault_id);
                SecurityPallet::set_active_block_number(110);
                mine_blocks(12);
                set_redeem_period(200);

                // request uses period = 200, so execute succeeds and cancel fails
                assert_noop!(cancel_redeem(redeem_id), RedeemError::TimeNotExpired);
                assert_ok!(execute_redeem(redeem_id));
            });
        }

        #[test]
        fn integration_test_redeem_can_only_be_cancelled_by_redeemer() {
            // PRECONDITION: The function call MUST be signed by redeemRequest.redeemer,
            // i.e. this function can only be called by the account who made the redeem request.
            test_with(|vault_id| {
                set_redeem_period(1000);
                let redeem_id = request_redeem(&vault_id);
                mine_blocks(12);
                SecurityPallet::set_active_block_number(1100);
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                        redeem_id: redeem_id,
                        reimburse: true
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    RedeemError::UnauthorizedRedeemer
                );
            });
        }

        #[test]
        fn integration_test_redeem_wrapped_cancel_reimburse_sufficient_collateral_for_wrapped() {
            // POSTCONDITIONS:
            // - If the vault is not liquidated, the following collateral changes are made:
            //     - If `reimburse` is true, the user SHOULD be reimbursed the worth of `amountIncludingParachainFee`
            //   in collateral. The transfer MUST be saturating, i.e. if the amount is not available, it should transfer
            // whatever amount is available.
            //     - A punishment fee MUST be tranferred from the vault’s backing collateral to the redeemer:
            //       `PunishmentFee`.
            //   The transfer MUST be saturating, i.e. if the amount is not available, it should transfer whatever
            // amount is available.
            // - `redeem.fee()` MUST be transferred from the vault to the fee pool if non-zero.
            // - If after the loss of collateral the vault remains above the `SecureCollateralThreshold`:
            //     - `amountIncludingParachainFee` of the user’s tokens MUST be unlocked and transferred to the vault.
            //     - The `redeem.status` is set to `Reimbursed(true)`, where the true indicates that the vault has
            //       received the tokens.
            // - The vault MUST be banned.
            test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let amount_btc = vault_id.wrapped(10_000);

                let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                let parachain_state_before_cancellation = ParachainState::get(&vault_id);
                let amount_without_fee_collateral = redeem.amount_without_fee_as_collateral(currency_id);

                let punishment_fee = FeePallet::get_punishment_fee(&amount_without_fee_collateral).unwrap();
                assert!(punishment_fee.amount() > 0);

                // alice cancels redeem request and chooses to reimburse
                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: true
                })
                .dispatch(origin_of(account_of(USER))));

                assert_eq!(
                    ParachainState::get(&vault_id),
                    parachain_state_before_cancellation.with_changes(|user, vault, _, fee_pool| {
                        // vault gets slashed for 110% to user
                        vault.backing_collateral -= amount_without_fee_collateral + punishment_fee;
                        *vault.free_balance.get_mut(&vault_id.wrapped_currency()).unwrap() +=
                            redeem.amount_btc() + redeem.transfer_fee_btc();
                        vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();

                        (*user.balances.get_mut(&currency_id).unwrap()).free +=
                            amount_without_fee_collateral + punishment_fee;
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -= amount_btc;

                        *fee_pool.rewards_for(&vault_id) += redeem.fee();
                    })
                );
                check_redeem_status(USER, RedeemRequestStatus::Reimbursed(true));
                assert_noop!(
                    VaultRegistryPallet::ensure_not_banned(&vault_id),
                    VaultRegistryError::VaultBanned
                );
            });
        }

        #[test]
        fn integration_test_redeem_wrapped_cancel_reimburse_insufficient_collateral_for_wrapped() {
            // POSTCONDITIONS:
            // - If the vault is not liquidated, the following collateral changes are made:
            //     - If `reimburse` is true, the user SHOULD be reimbursed the worth of `amountIncludingParachainFee`
            //   in collateral. The transfer MUST be saturating, i.e. if the amount is not available, it should transfer
            // whatever amount is available.
            //     - A punishment fee MUST be tranferred from the vault’s backing collateral to the redeemer:
            //       `PunishmentFee`.
            //   The transfer MUST be saturating, i.e. if the amount is not available, it should transfer whatever
            // amount is available.
            // - `redeem.fee()` MUST be transferred from the vault to the fee pool if non-zero.
            // - If after the loss of collateral the vault is below the `SecureCollateralThreshold`:
            //     - `amountIncludingParachainFee` of the user’s tokens are burned.
            //     - `decreaseTokens` MUST be called, supplying the vault, the user, and `amountIncludingParachainFee`
            //       as arguments.
            //     - The `redeem.status` is set to `Reimbursed(false)`, where the `false` indicates that the vault has
            //       not yet received the tokens.
            // - The vault MUST be banned.
            test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let amount_btc = vault_id.wrapped(10_000);

                // set collateral to the minimum amount required, such that the vault can not afford to both
                // reimburse and keep collateral his current tokens
                let required_collateral =
                    VaultRegistryPallet::get_required_collateral_for_wrapped(&DEFAULT_VAULT_ISSUED, currency_id)
                        .unwrap();
                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        backing_collateral: required_collateral,
                        ..CoreVaultData::vault(vault_id.clone())
                    },
                );

                let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                let parachain_state_before_cancellation = ParachainState::get(&vault_id);
                let amount_without_fee_as_collateral = redeem.amount_without_fee_as_collateral(currency_id);

                let punishment_fee = FeePallet::get_punishment_fee(&amount_without_fee_as_collateral).unwrap();
                assert!(punishment_fee.amount() > 0);

                // alice cancels redeem request and chooses to reimburse
                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: true
                })
                .dispatch(origin_of(account_of(USER))));

                assert_eq!(
                    ParachainState::get(&vault_id),
                    parachain_state_before_cancellation.with_changes(|user, vault, _, fee_pool| {
                        // vault gets slashed for 110% to user
                        vault.backing_collateral -= amount_without_fee_as_collateral + punishment_fee;
                        // vault free tokens does not change, and issued tokens is reduced
                        vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();
                        vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();

                        (*user.balances.get_mut(&currency_id).unwrap()).free +=
                            amount_without_fee_as_collateral + punishment_fee;
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -= amount_btc;

                        *fee_pool.rewards_for(&vault_id) += redeem.fee();
                    })
                );

                SecurityPallet::set_active_block_number(100000000);
                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        backing_collateral: required_collateral + amount_btc.convert_to(currency_id).unwrap() * 2,
                        ..CoreVaultData::vault(vault_id.clone())
                    },
                );
                check_redeem_status(USER, RedeemRequestStatus::Reimbursed(false));
            });
        }

        #[test]
        fn integration_test_redeem_wrapped_cancel_no_reimburse() {
            // POSTCONDITIONS:
            // - If the vault is not liquidated, the following collateral changes are made:
            //     - A punishment fee MUST be tranferred from the vault’s backing collateral to the redeemer:
            //       `PunishmentFee`.
            //   The transfer MUST be saturating, i.e. if the amount is not available, it should transfer whatever
            // amount is available.
            // - If `reimburse` is false:
            //     - All the user’s tokens that were locked in `requestRedeem` MUST be unlocked, i.e. an amount of
            // `redeem.amountBtc + redeem.fee() + redeem.transferFeeBtc`.
            //     - The vault’s `toBeRedeemedTokens` MUST decrease by `amountIncludingParachainFee`.
            // - The vault MUST be banned.
            test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let amount_btc = vault_id.wrapped(10_000);

                let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
                let parachain_state_before_cancellation = ParachainState::get(&vault_id);
                let amount_without_fee_collateral = redeem.amount_without_fee_as_collateral(currency_id);

                let punishment_fee = FeePallet::get_punishment_fee(&amount_without_fee_collateral).unwrap();
                assert!(punishment_fee.amount() > 0);

                // alice cancels redeem request and chooses not to reimburse
                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: false
                })
                .dispatch(origin_of(account_of(USER))));

                assert_eq!(
                    ParachainState::get(&vault_id),
                    parachain_state_before_cancellation.with_changes(|user, vault, _, _| {
                        // vault is slashed a punishment fee of 10%
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -=
                            redeem.amount_btc() + redeem.transfer_fee_btc() + redeem.fee();
                        (*user.balances.get_mut(&currency_id).unwrap()).free += punishment_fee;
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free +=
                            redeem.amount_btc() + redeem.transfer_fee_btc() + redeem.fee();

                        vault.backing_collateral -= punishment_fee;
                        vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                    })
                );
                assert_noop!(
                    VaultRegistryPallet::ensure_not_banned(&vault_id),
                    VaultRegistryError::VaultBanned
                );
            });
        }

        #[test]
        fn integration_test_redeem_wrapped_cancel_liquidated_no_reimburse() {
            // POSTCONDITIONS:
            // - If the vault is liquidated:
            //    - If ``reimburse`` is false, an amount of ``confiscatedCollateral`` MUST be transferred from the vault
            //      to the redeemer.
            test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let issued_tokens = vault_id.wrapped(10_000);
                let collateral_vault = Amount::new(1_000_000, currency_id);
                let redeem_id = setup_cancelable_redeem(USER, &vault_id, issued_tokens);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

                // setup vault state such that 1/4th of its collateral is freed after successful redeem
                let consumed_issued_tokens = redeem.amount_btc() + redeem.transfer_fee_btc();
                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        issued: consumed_issued_tokens * 4,
                        to_be_issued: vault_id.wrapped(0),
                        to_be_redeemed: consumed_issued_tokens * 4,
                        backing_collateral: collateral_vault,
                        to_be_replaced: vault_id.wrapped(0),
                        replace_collateral: griefing(0),
                        ..default_vault_state(&vault_id)
                    },
                );

                liquidate_vault(&vault_id);

                let post_liquidation_state = ParachainState::get(&vault_id);

                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: false
                })
                .dispatch(origin_of(account_of(USER))));

                // NOTE: changes are relative the the post liquidation state
                assert_eq!(
                    ParachainState::get(&vault_id),
                    post_liquidation_state.with_changes(|user, vault, liquidation_vault, _fee_pool| {
                        let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);

                        // to-be-redeemed decreased, forwarding to liquidation vault
                        vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                        liquidation_vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();

                        // the collateral that remained with the vault to back this redeem is now transferred to the
                        // liquidation vault
                        let collateral_for_this_redeem = collateral_vault / 4;
                        vault.liquidated_collateral -= collateral_for_this_redeem;
                        liquidation_vault.collateral += collateral_for_this_redeem;

                        // user's tokens get unlocked
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -=
                            redeem.amount_btc() + redeem.fee() + redeem.transfer_fee_btc();
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free +=
                            redeem.amount_btc() + redeem.fee() + redeem.transfer_fee_btc();

                        // Note that no punishment is taken from vault, because it's already liquidated
                    })
                );
            });
        }

        #[test]
        fn integration_test_redeem_wrapped_cancel_liquidated_reimburse() {
            // POSTCONDITIONS:
            // - If the vault is liquidated:
            //    - If ``reimburse`` is true:
            //       - an amount of ``confiscatedCollateral`` MUST be transferred from the vault to the redeemer.
            //       - `redeem.fee()` MUST be transferred from the vault to the fee pool if non-zero.
            test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let issued_tokens = vault_id.wrapped(10_000);
                let collateral_vault = Amount::new(1_000_000, currency_id);
                let redeem_id = setup_cancelable_redeem(USER, &vault_id, issued_tokens);
                let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

                // setup vault state such that 1/4th of its collateral is freed after successful redeem
                let consumed_issued_tokens = redeem.amount_btc() + redeem.transfer_fee_btc();
                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        issued: consumed_issued_tokens * 4,
                        to_be_issued: vault_id.wrapped(0),
                        to_be_redeemed: consumed_issued_tokens * 4,
                        backing_collateral: collateral_vault,
                        to_be_replaced: vault_id.wrapped(0),
                        replace_collateral: griefing(0),
                        ..default_vault_state(&vault_id)
                    },
                );

                liquidate_vault(&vault_id);

                let post_liquidation_state = ParachainState::get(&vault_id);

                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: true
                })
                .dispatch(origin_of(account_of(USER))));

                // NOTE: changes are relative to the post liquidation state
                assert_eq!(
                    ParachainState::get(&vault_id),
                    post_liquidation_state.with_changes(|user, vault, liquidation_vault, fee_pool| {
                        let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);

                        // to-be-redeemed decreased, forwarding to liquidation vault
                        vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                        liquidation_vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                        liquidation_vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();

                        *fee_pool.rewards_for(&vault_id) += redeem.fee();

                        // the collateral that remained with the vault to back this redeem is now transferred to the
                        // user
                        let collateral_for_this_redeem = collateral_vault / 4;
                        vault.liquidated_collateral -= collateral_for_this_redeem;

                        // user's tokens get unlocked
                        (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -=
                            redeem.amount_btc() + redeem.fee() + redeem.transfer_fee_btc();
                        (*user.balances.get_mut(&currency_id).unwrap()).free += collateral_for_this_redeem;

                        // Note that no punishment is taken from vault, because it's already liquidated
                    })
                );
            });
        }
    }

    mod mint_tokens_for_reimbursed_redeem {
        use super::{assert_eq, *};

        #[test]
        fn integration_test_mint_tokens_for_reimbursed_redeem_equivalence_to_succesful_cancel() {
            // PRECONDITIONS:
            // - A pending `RedeemRequest` MUST exist with an id equal to `redeemId`.
            // - The vault MUST have sufficient collateral to remain above the `SecureCollateralThreshold` after
            // issuing `redeem.amountBtc + redeem.transferFeeBtc` tokens.
            // - The function call MUST be signed by `redeem.vault`, i.e. this function can only be called by the the
            //   vault.
            // POSTCONDITION: `redeem.amountBtc + redeem.transferFeeBtc` tokens MUST be minted to the vault.

            // scenario 1: sufficient collateral
            let result1 = test_with(|vault_id| {
                let redeem_id = setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id.clone());
                get_additional_collateral(&vault_id);
                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: true
                })
                .dispatch(origin_of(account_of(USER))));
                ParachainState::get(&vault_id)
            });
            // scenario 2: insufficient collateral
            let result2 = test_with(|vault_id| {
                let currency_id = vault_id.collateral_currency();
                let redeem_id = setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id.clone());
                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: true
                })
                .dispatch(origin_of(account_of(USER))));
                get_additional_collateral(&vault_id);
                SecurityPallet::set_active_block_number(100000000);

                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                        currency_pair: vault_id.currencies.clone(),
                        redeem_id: H256::random()
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    RedeemError::RedeemIdNotFound
                );
                let tmp = CoreVaultData::vault(vault_id.clone());
                CoreVaultData::force_to(
                    &vault_id,
                    CoreVaultData {
                        backing_collateral: Amount::new(0, currency_id),
                        ..CoreVaultData::vault(vault_id.clone())
                    },
                );
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                        currency_pair: vault_id.currencies.clone(),
                        redeem_id: redeem_id
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    VaultRegistryError::ExceedingVaultLimit
                );
                CoreVaultData::force_to(&vault_id, tmp);
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                        currency_pair: VaultCurrencyPair {
                            collateral: vault_id.currencies.collateral,
                            wrapped: if vault_id.currencies.wrapped == Token(DOT) {
                                Token(IBTC)
                            } else {
                                Token(DOT)
                            },
                        },
                        redeem_id: redeem_id
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    RedeemError::UnauthorizedVault
                );
                assert_ok!(RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                    currency_pair: vault_id.currencies.clone(),
                    redeem_id: redeem_id
                })
                .dispatch(origin_of(account_of(VAULT))));
                ParachainState::get(&vault_id)
            });
            // the states should be identical
            assert_eq!(result1, result2);
        }

        #[test]
        fn integration_test_mint_tokens_for_reimbursed_redeem_wrong_status() {
            // PRECONDITION: `redeem.status` MUST be `Reimbursed(false)`.
            // POSTCONDITION: redeem.amountBtc + redeem.transferFeeBtc tokens MUST be minted to the vault.

            // scenario 1: sufficient collateral
            test_with(|vault_id| {
                let redeem_id = setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id.clone());
                assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: false
                })
                .dispatch(origin_of(account_of(USER))));
                get_additional_collateral(&vault_id);
                SecurityPallet::set_active_block_number(100000000);
                assert_noop!(
                    RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                        currency_pair: vault_id.currencies.clone(),
                        redeem_id: redeem_id
                    })
                    .dispatch(origin_of(account_of(VAULT))),
                    RedeemError::RedeemCancelled
                );
            });
        }
    }
}

mod execute_redeem_payment_limits {
    use super::{assert_eq, *};

    #[test]
    fn integration_test_redeem_polka_btc_execute_underpayment_fails() {
        test_with(|vault_id| {
            let redeem_id = setup_redeem(vault_id.wrapped(10_000), USER, &vault_id);
            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

            assert_noop!(
                ExecuteRedeemBuilder::new(redeem_id)
                    .with_amount(redeem.amount_btc().with_amount(|x| x - 1))
                    .execute(),
                BTCRelayError::InvalidPaymentAmount
            );
        });
    }

    #[test]
    fn integration_test_redeem_polka_btc_execute_with_exact_amount_succeeds() {
        test_with(|vault_id| {
            let redeem_id = setup_redeem(vault_id.wrapped(10_000), USER, &vault_id);
            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

            ExecuteRedeemBuilder::new(redeem_id)
                .with_amount(redeem.amount_btc())
                .assert_execute();
        });
    }

    #[test]
    fn integration_test_redeem_polka_btc_execute_overpayment_fails() {
        test_with(|vault_id| {
            let redeem_id = setup_redeem(vault_id.wrapped(10_000), USER, &vault_id);
            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

            assert_noop!(
                ExecuteRedeemBuilder::new(redeem_id)
                    .with_amount(redeem.amount_btc().with_amount(|x| x + 1))
                    .execute(),
                BTCRelayError::InvalidPaymentAmount
            );
        });
    }
}

#[test]
fn integration_test_redeem_execute_succeeds() {
    test_with(|vault_id| {
        let issued_tokens = vault_id.wrapped(10_000);

        let redeem_id = setup_redeem(issued_tokens, USER, &vault_id);
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        execute_redeem(redeem_id);

        assert_eq!(
            ParachainState::get(&vault_id),
            ParachainState::get_default(&vault_id).with_changes(|user, vault, _, fee_pool| {
                vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= issued_tokens;
                *fee_pool.rewards_for(&vault_id) += redeem.fee();
                consume_to_be_replaced(vault, redeem.amount_btc() + redeem.transfer_fee_btc());
            })
        );
    });
}

#[test]
fn integration_test_execute_redeem_on_banned_vault_succeeds() {
    test_with(|vault_id| {
        let amount_btc = vault_id.wrapped(10000);
        let redeem_id_1 = setup_cancelable_redeem(USER, &vault_id, amount_btc);
        let redeem_id_2 = setup_redeem(amount_btc, USER, &vault_id);

        // cancel first
        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id_1,
            reimburse: true
        })
        .dispatch(origin_of(account_of(USER))));

        // should now be banned
        assert_noop!(
            VaultRegistryPallet::ensure_not_banned(&vault_id),
            VaultRegistryError::VaultBanned
        );

        // should still be able to execute despite being banned
        assert_ok!(ExecuteRedeemBuilder::new(redeem_id_2).execute());
    })
}

#[test]
fn integration_test_premium_redeem_wrapped_execute() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let issued_tokens = vault_id.wrapped(10_000);

        let user_btc_address = BtcAddress::P2PKH(H160([2; 20]));

        // make vault undercollateralized. Note that we place it under the liquidation threshold
        // as well, but as long as we don't call liquidate that's ok
        set_collateral_exchange_rate(&vault_id, FixedU128::from(100));

        // alice requests to redeem issued_tokens from Bob
        assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
            amount_wrapped: issued_tokens.amount(),
            btc_address: user_btc_address,
            vault_id: vault_id.clone()
        })
        .dispatch(origin_of(account_of(USER))));

        // assert that request happened and extract the id
        let redeem_id = assert_redeem_request_event();
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        // send the btc from the vault to the user
        let (_tx_id, _tx_block_height, transaction) = generate_transaction_and_mine(
            Default::default(),
            vec![],
            vec![(user_btc_address, redeem.amount_btc())],
            vec![redeem_id],
        );

        SecurityPallet::set_active_block_number(1 + CONFIRMATIONS);

        assert_ok!(RuntimeCall::Redeem(RedeemCall::execute_redeem {
            redeem_id,
            unchecked_transaction: transaction
        })
        .dispatch(origin_of(account_of(VAULT))));

        assert_eq!(
            ParachainState::get(&vault_id),
            ParachainState::get_default(&vault_id).with_changes(|user, vault, _, fee_pool| {
                // fee moves from user to fee_pool
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= redeem.fee();
                *fee_pool.rewards_for(&vault_id) += redeem.fee();
                // amount_btc is burned from user and decreased on vault
                let burned_amount = redeem.amount_btc() + redeem.transfer_fee_btc();
                vault.issued -= burned_amount;
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= burned_amount;
                // premium is moved from vault to user
                vault.backing_collateral -= redeem.premium().unwrap();
                (*user.balances.get_mut(&currency_id).unwrap()).free += redeem.premium().unwrap();
                consume_to_be_replaced(vault, burned_amount);
            })
        );

        let premium: Amount<Runtime> = redeem.premium().unwrap();
        assert!(!premium.is_zero()); // sanity check that our test is useful
    });
}

#[test]
fn integration_test_multiple_redeems_multiple_op_returns() {
    test_with(|vault_id| {
        let issued_tokens = vault_id.wrapped(10_000);
        let user_1_btc_address = BtcAddress::P2PKH(H160([2; 20]));
        let user_2_btc_address = BtcAddress::P2PKH(H160([3; 20]));

        assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
            amount_wrapped: issued_tokens.amount(),
            btc_address: user_1_btc_address,
            vault_id: vault_id.clone()
        })
        .dispatch(origin_of(account_of(ALICE))));

        // assert that request happened and extract the id
        let redeem_1_id = assert_redeem_request_event();
        let redeem_1 = RedeemPallet::get_open_redeem_request_from_id(&redeem_1_id).unwrap();

        assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
            amount_wrapped: issued_tokens.amount(),
            btc_address: user_2_btc_address,
            vault_id: vault_id.clone()
        })
        .dispatch(origin_of(account_of(CAROL))));

        // assert that request happened and extract the id
        let redeem_2_id = assert_redeem_request_event();
        let redeem_2 = RedeemPallet::get_open_redeem_request_from_id(&redeem_2_id).unwrap();

        // try to fulfill both redeem requests in a single transaction
        let (_tx_id, _tx_block_height, transaction) = generate_transaction_and_mine(
            Default::default(),
            vec![],
            vec![
                (user_1_btc_address, redeem_1.amount_btc()),
                (user_2_btc_address, redeem_2.amount_btc()),
            ],
            vec![redeem_1_id, redeem_2_id],
        );

        SecurityPallet::set_active_block_number(1 + CONFIRMATIONS);

        assert_err!(
            RuntimeCall::Redeem(RedeemCall::execute_redeem {
                redeem_id: redeem_1_id,
                unchecked_transaction: transaction.clone()
            })
            .dispatch(origin_of(account_of(VAULT))),
            BTCRelayError::InvalidOpReturnTransaction
        );

        assert_err!(
            RuntimeCall::Redeem(RedeemCall::execute_redeem {
                redeem_id: redeem_2_id,
                unchecked_transaction: transaction
            })
            .dispatch(origin_of(account_of(VAULT))),
            BTCRelayError::InvalidOpReturnTransaction
        );
    });
}

#[test]
fn integration_test_single_redeem_multiple_op_returns() {
    test_with(|vault_id| {
        let issued_tokens = vault_id.wrapped(10_000);
        let user_btc_address = BtcAddress::P2PKH(H160([2; 20]));

        assert_ok!(RuntimeCall::Redeem(RedeemCall::request_redeem {
            amount_wrapped: issued_tokens.amount(),
            btc_address: user_btc_address,
            vault_id: vault_id.clone()
        })
        .dispatch(origin_of(account_of(ALICE))));

        // assert that request happened and extract the id
        let redeem_id = assert_redeem_request_event();
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        let (_tx_id, _tx_block_height, transaction) = generate_transaction_and_mine(
            Default::default(),
            vec![],
            vec![(user_btc_address, redeem.amount_btc())],
            vec![
                redeem_id,
                H256::from_str(&"278e2f901256e2a7bab9071cea41da7b392c157aa50e70cae90f5e2a50c49e8d").unwrap(),
            ],
        );

        SecurityPallet::set_active_block_number(1 + CONFIRMATIONS);

        assert_err!(
            RuntimeCall::Redeem(RedeemCall::execute_redeem {
                redeem_id,
                unchecked_transaction: transaction
            })
            .dispatch(origin_of(account_of(VAULT))),
            BTCRelayError::InvalidOpReturnTransaction
        );
    });
}

#[test]
fn integration_test_redeem_wrapped_liquidation_redeem() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let issued = vault_id.wrapped(400);
        let to_be_issued = vault_id.wrapped(100);
        let to_be_redeemed = vault_id.wrapped(50);
        let liquidation_redeem_amount = vault_id.wrapped(325);

        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                issued,
                to_be_issued,
                to_be_redeemed,
                backing_collateral: Amount::new(10_000, currency_id),
                ..CoreVaultData::get_default(&vault_id)
            },
        );

        // create tokens for the vault and user
        liquidate_vault(&vault_id);

        let post_liquidation_state = ParachainState::get(&vault_id);

        assert_noop!(
            RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
                currencies: vault_id.currencies.clone(),
                amount_wrapped: 351
            })
            .dispatch(origin_of(account_of(USER))),
            VaultRegistryError::InsufficientTokensCommitted
        );

        assert_ok!(RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
            currencies: vault_id.currencies.clone(),
            amount_wrapped: liquidation_redeem_amount.amount()
        })
        .dispatch(origin_of(account_of(USER))));

        // NOTE: changes are relative the the post liquidation state
        assert_eq!(
            ParachainState::get(&vault_id),
            post_liquidation_state.with_changes(|user, _vault, liquidation_vault, _fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);
                let reward = liquidation_vault.collateral.with_amount(|x| {
                    (x * liquidation_redeem_amount.amount())
                        / (liquidation_vault.issued + liquidation_vault.to_be_issued - liquidation_vault.to_be_redeemed)
                            .amount()
                });

                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= liquidation_redeem_amount;
                (*user.balances.get_mut(&currency_id).unwrap()).free += reward;

                liquidation_vault.issued -= liquidation_redeem_amount;
                liquidation_vault.collateral -= reward;
            })
        );
    });
}

#[test]
fn integration_test_redeem_wrapped_cancel_reimburse_sufficient_collateral_for_wrapped() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let amount_btc = vault_id.wrapped(10_000);

        let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
        let amount_without_fee_collateral = redeem.amount_without_fee_as_collateral(currency_id);

        let punishment_fee = FeePallet::get_punishment_fee(&amount_without_fee_collateral).unwrap();
        assert!(punishment_fee.amount() > 0);

        // alice cancels redeem request and chooses to reimburse
        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id,
            reimburse: true
        })
        .dispatch(origin_of(account_of(USER))));

        assert_eq!(
            ParachainState::get(&vault_id),
            ParachainState::get_default(&vault_id).with_changes(|user, vault, _, fee_pool| {
                // vault gets slashed for 110% to user
                vault.backing_collateral -= amount_without_fee_collateral + punishment_fee;
                *vault.free_balance.get_mut(&vault_id.wrapped_currency()).unwrap() +=
                    redeem.amount_btc() + redeem.transfer_fee_btc();

                (*user.balances.get_mut(&currency_id).unwrap()).free += amount_without_fee_collateral + punishment_fee;
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= amount_btc;

                *fee_pool.rewards_for(&vault_id) += redeem.fee();

                consume_to_be_replaced(vault, redeem.amount_btc() + redeem.transfer_fee_btc());
            })
        );
    });
}

#[test]
fn integration_test_redeem_wrapped_cancel_reimburse_insufficient_collateral_for_wrapped() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let amount_btc = vault_id.wrapped(10_000);

        // set collateral to the minimum amount required, such that the vault can not afford to both
        // reimburse and keep collateral his current tokens
        let required_collateral =
            VaultRegistryPallet::get_required_collateral_for_wrapped(&DEFAULT_VAULT_ISSUED, currency_id).unwrap();
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                backing_collateral: required_collateral,
                ..CoreVaultData::vault(vault_id.clone())
            },
        );
        let initial_state = ParachainState::get(&vault_id);

        let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();
        let amount_without_fee_as_collateral = redeem.amount_without_fee_as_collateral(currency_id);

        let punishment_fee = FeePallet::get_punishment_fee(&amount_without_fee_as_collateral).unwrap();
        assert!(punishment_fee.amount() > 0);

        // alice cancels redeem request and chooses to reimburse
        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id,
            reimburse: true
        })
        .dispatch(origin_of(account_of(USER))));

        assert_eq!(
            ParachainState::get(&vault_id),
            initial_state.with_changes(|user, vault, _, fee_pool| {
                // vault gets slashed for 110% to user
                vault.backing_collateral -= amount_without_fee_as_collateral + punishment_fee;
                // vault free tokens does not change, and issued tokens is reduced
                vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();

                (*user.balances.get_mut(&currency_id).unwrap()).free +=
                    amount_without_fee_as_collateral + punishment_fee;
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= amount_btc;

                *fee_pool.rewards_for(&vault_id) += redeem.fee();

                consume_to_be_replaced(vault, redeem.amount_btc() + redeem.transfer_fee_btc());
            })
        );

        SecurityPallet::set_active_block_number(100000000);
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                backing_collateral: required_collateral + amount_btc.convert_to(currency_id).unwrap() * 2,
                ..CoreVaultData::vault(vault_id.clone())
            },
        );
        let pre_minting_state = ParachainState::get(&vault_id);

        assert_ok!(RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
            currency_pair: vault_id.currencies.clone(),
            redeem_id: redeem_id
        })
        .dispatch(origin_of(account_of(VAULT))));
        assert_eq!(
            ParachainState::get(&vault_id),
            pre_minting_state.with_changes(|_user, vault, _, _fee_pool| {
                vault.issued += redeem.amount_btc() + redeem.transfer_fee_btc();
                *vault.free_balance.get_mut(&vault_id.wrapped_currency()).unwrap() +=
                    redeem.amount_btc() + redeem.transfer_fee_btc();
            })
        );
    });
}

#[test]
fn integration_test_redeem_wrapped_cancel_no_reimburse() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let amount_btc = vault_id.wrapped(10_000);

        let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        let punishment_fee =
            FeePallet::get_punishment_fee(&redeem.amount_without_fee_as_collateral(currency_id)).unwrap();
        assert!(punishment_fee.amount() > 0);

        // alice cancels redeem request and chooses not to reimburse
        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id,
            reimburse: false
        })
        .dispatch(origin_of(account_of(USER))));

        assert_eq!(
            ParachainState::get(&vault_id),
            ParachainState::get_default(&vault_id).with_changes(|user, vault, _, _| {
                // vault is slashed a punishment fee of 10%

                (*user.balances.get_mut(&currency_id).unwrap()).free += punishment_fee;

                vault.backing_collateral -= punishment_fee;

                consume_to_be_replaced(vault, redeem.amount_btc() + redeem.transfer_fee_btc());
            })
        );
    });
}

#[test]
fn integration_test_liquidation_redeem_with_cancel_redeem() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let issued_tokens = vault_id.wrapped(10_000);
        let collateral_vault = Amount::new(1_000_000, currency_id);
        VaultRegistryPallet::collateral_integrity_check();
        let redeem_id = setup_cancelable_redeem(USER, &vault_id, issued_tokens);
        VaultRegistryPallet::collateral_integrity_check();
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        // setup vault state such that 1/4th of its collateral is freed after successful redeem
        let consumed_issued_tokens = redeem.amount_btc() + redeem.transfer_fee_btc();
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                issued: consumed_issued_tokens * 2,
                to_be_issued: vault_id.wrapped(0),
                to_be_redeemed: consumed_issued_tokens,
                backing_collateral: collateral_vault,
                to_be_replaced: vault_id.wrapped(0),
                replace_collateral: griefing(0),
                ..default_vault_state(&vault_id)
            },
        );

        // make sure user has plenty of kbtc
        TokensPallet::set_balance(root(), account_of(USER), Token(KBTC), 10000000000, 10000000000).unwrap();

        liquidate_vault(&vault_id);

        let post_liquidation_state = ParachainState::get(&vault_id);

        assert_ok!(RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
            currencies: vault_id.currencies.clone(),
            amount_wrapped: consumed_issued_tokens.amount()
        })
        .dispatch(origin_of(account_of(USER))));
        assert_noop!(
            RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
                currencies: vault_id.currencies.clone(),
                amount_wrapped: 1
            })
            .dispatch(origin_of(account_of(USER))),
            VaultRegistryError::InsufficientTokensCommitted
        );

        let pre_cancellation_state = ParachainState::get(&vault_id);
        assert_eq!(
            pre_cancellation_state,
            post_liquidation_state.with_changes(|user, _vault, liquidation_vault, _fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);
                liquidation_vault.issued -= consumed_issued_tokens;
                liquidation_vault.collateral -= collateral_vault / 2;

                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= consumed_issued_tokens;
                (*user.balances.get_mut(&vault_id.collateral_currency()).unwrap()).free += collateral_vault / 2;
            })
        );

        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id,
            reimburse: false
        })
        .dispatch(origin_of(account_of(USER))));

        let post_cancellation_state = ParachainState::get(&vault_id);
        assert_eq!(
            post_cancellation_state,
            pre_cancellation_state.with_changes(|user, vault, liquidation_vault, _fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);
                liquidation_vault.to_be_redeemed -= consumed_issued_tokens;
                liquidation_vault.collateral += collateral_vault / 2;

                vault.to_be_redeemed -= consumed_issued_tokens;
                vault.liquidated_collateral -= collateral_vault / 2;

                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -= issued_tokens;
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free += issued_tokens;
            })
        );

        assert_ok!(RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
            currencies: vault_id.currencies.clone(),
            amount_wrapped: consumed_issued_tokens.amount()
        })
        .dispatch(origin_of(account_of(USER))));

        // expect same change as the previous liquidation_redeem
        assert_eq!(
            ParachainState::get(&vault_id),
            post_cancellation_state.with_changes(|user, _vault, liquidation_vault, _fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);
                liquidation_vault.issued -= consumed_issued_tokens;
                liquidation_vault.collateral -= collateral_vault / 2;

                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= consumed_issued_tokens;
                (*user.balances.get_mut(&vault_id.collateral_currency()).unwrap()).free += collateral_vault / 2;
            })
        );
    })
}

#[test]
fn integration_test_redeem_wrapped_cancel_liquidated_no_reimburse() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let issued_tokens = vault_id.wrapped(10_000);
        let collateral_vault = Amount::new(1_000_000, currency_id);
        VaultRegistryPallet::collateral_integrity_check();
        let redeem_id = setup_cancelable_redeem(USER, &vault_id, issued_tokens);
        VaultRegistryPallet::collateral_integrity_check();
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        // setup vault state such that 1/4th of its collateral is freed after successful redeem
        let consumed_issued_tokens = redeem.amount_btc() + redeem.transfer_fee_btc();
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                issued: consumed_issued_tokens * 4,
                to_be_issued: vault_id.wrapped(0),
                to_be_redeemed: consumed_issued_tokens * 4,
                backing_collateral: collateral_vault,
                to_be_replaced: vault_id.wrapped(0),
                replace_collateral: griefing(0),
                ..default_vault_state(&vault_id)
            },
        );

        liquidate_vault(&vault_id);

        let post_liquidation_state = ParachainState::get(&vault_id);

        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id,
            reimburse: false
        })
        .dispatch(origin_of(account_of(USER))));

        // NOTE: changes are relative the the post liquidation state
        assert_eq!(
            ParachainState::get(&vault_id),
            post_liquidation_state.with_changes(|user, vault, liquidation_vault, _fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);

                // to-be-redeemed decreased, forwarding to liquidation vault
                vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                liquidation_vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();

                // the collateral that remained with the vault to back this redeem is now transferred to the
                // liquidation vault
                let collateral_for_this_redeem = collateral_vault / 4;
                vault.liquidated_collateral -= collateral_for_this_redeem;
                liquidation_vault.collateral += collateral_for_this_redeem;

                // user's tokens get unlocked
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -=
                    redeem.amount_btc() + redeem.fee() + redeem.transfer_fee_btc();
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free +=
                    redeem.amount_btc() + redeem.fee() + redeem.transfer_fee_btc();

                // Note that no punishment is taken from vault, because it's already liquidated
            })
        );
    });
}

#[test]
fn integration_test_redeem_wrapped_cancel_liquidated_reimburse() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let issued_tokens = vault_id.wrapped(10_000);
        let collateral_vault = Amount::new(1_000_000, currency_id);
        let redeem_id = setup_cancelable_redeem(USER, &vault_id, issued_tokens);
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        // setup vault state such that 1/4th of its collateral is freed after successful redeem
        let consumed_issued_tokens = redeem.amount_btc() + redeem.transfer_fee_btc();
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                issued: consumed_issued_tokens * 4,
                to_be_issued: vault_id.wrapped(0),
                to_be_redeemed: consumed_issued_tokens * 4,
                backing_collateral: collateral_vault,
                to_be_replaced: vault_id.wrapped(0),
                replace_collateral: griefing(0),
                ..default_vault_state(&vault_id)
            },
        );

        liquidate_vault(&vault_id);

        let post_liquidation_state = ParachainState::get(&vault_id);

        assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
            redeem_id: redeem_id,
            reimburse: true
        })
        .dispatch(origin_of(account_of(USER))));

        // NOTE: changes are relative the the post liquidation state
        assert_eq!(
            ParachainState::get(&vault_id),
            post_liquidation_state.with_changes(|user, vault, liquidation_vault, fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);

                // to-be-redeemed decreased, forwarding to liquidation vault
                vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                liquidation_vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                // decrease issued tokens on the liquidation vault by the same amount, s.t. the
                // effective exchange rate (i.e. the one accounting for to_be_redeemed tokens)
                // of the liquidation vault does not change.
                liquidation_vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();

                // tokens are given to the vault, minus a fee that is given to the fee pool
                *fee_pool.rewards_for(&vault_id) += redeem.fee();

                // the collateral that remained with the vault to back this redeem is transferred to the user
                let collateral_for_this_redeem = collateral_vault / 4;
                vault.liquidated_collateral -= collateral_for_this_redeem;
                (*user.balances.get_mut(&currency_id).unwrap()).free += collateral_for_this_redeem;

                // user's tokens get burned
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -= issued_tokens;

                // Note that no punishment is taken from vault, because it's already liquidated
            })
        );
    });
}

#[test]
fn integration_test_redeem_wrapped_execute_liquidated() {
    test_with(|vault_id| {
        let currency_id = vault_id.collateral_currency();
        let issued_tokens = vault_id.wrapped(10_000);
        let collateral_vault = Amount::new(1_000_000, currency_id);

        let redeem_id = setup_redeem(issued_tokens, USER, &vault_id);
        let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

        // setup vault state such that 1/4th of its collateral is freed after successful redeem
        let consumed_issued_tokens = redeem.amount_btc() + redeem.transfer_fee_btc();
        CoreVaultData::force_to(
            &vault_id,
            CoreVaultData {
                issued: consumed_issued_tokens * 4,
                to_be_issued: vault_id.wrapped(0),
                to_be_redeemed: consumed_issued_tokens * 4,
                backing_collateral: collateral_vault,
                to_be_replaced: vault_id.wrapped(0),
                replace_collateral: griefing(0),
                ..default_vault_state(&vault_id)
            },
        );

        liquidate_vault(&vault_id);

        let post_liquidation_state = ParachainState::get(&vault_id);

        execute_redeem(redeem_id);

        // NOTE: changes are relative the the post liquidation state
        assert_eq!(
            ParachainState::get(&vault_id),
            post_liquidation_state.with_changes(|user, vault, liquidation_vault, fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);

                // fee given to fee pool
                *fee_pool.rewards_for(&vault_id) += redeem.fee();

                // wrapped burned from user
                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).locked -= issued_tokens;

                // to-be-redeemed & issued decreased, forwarding to liquidation vault
                vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                liquidation_vault.to_be_redeemed -= redeem.amount_btc() + redeem.transfer_fee_btc();
                liquidation_vault.issued -= redeem.amount_btc() + redeem.transfer_fee_btc();

                // collateral released
                let released_collateral = vault.liquidated_collateral / 4;
                vault.liquidated_collateral -= released_collateral;
                *vault.free_balance.get_mut(&currency_id).unwrap() += released_collateral;
            })
        );
    })
}

fn get_additional_collateral(vault_id: &VaultId) {
    assert_ok!(VaultRegistryPallet::transfer_funds(
        CurrencySource::FreeBalance(account_of(FAUCET)),
        CurrencySource::Collateral(vault_id.clone()),
        &Amount::new(100_000_000_000, vault_id.collateral_currency()),
    ));
}

fn setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id: VaultId) -> H256 {
    let currency_id = vault_id.collateral_currency();
    let amount_btc = vault_id.wrapped(10_000);

    // set collateral to the minimum amount required, such that the vault can not afford to both
    // reimburse and keep collateral his current tokens
    let required_collateral =
        VaultRegistryPallet::get_required_collateral_for_wrapped(&DEFAULT_VAULT_ISSUED, currency_id).unwrap();
    CoreVaultData::force_to(
        &vault_id,
        CoreVaultData {
            backing_collateral: required_collateral,
            ..CoreVaultData::vault(vault_id.clone())
        },
    );
    let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);
    let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

    let punishment_fee = FeePallet::get_punishment_fee(&redeem.amount_without_fee_as_collateral(currency_id)).unwrap();
    assert!(punishment_fee.amount() > 0);

    redeem_id
}

mod mint_tokens_for_reimbursed_redeem_equivalence_test {
    use super::{assert_eq, *};

    #[test]
    fn integration_test_mint_tokens_for_reimbursed_redeem_equivalence_to_succesful_cancel() {
        // scenario 1: sufficient collateral
        let result1 = test_with(|vault_id| {
            let redeem_id = setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id.clone());
            get_additional_collateral(&vault_id);
            assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                redeem_id: redeem_id,
                reimburse: true
            })
            .dispatch(origin_of(account_of(USER))));
            ParachainState::get(&vault_id)
        });
        // scenario 2: insufficient collateral
        let result2 = test_with(|vault_id| {
            let redeem_id = setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id.clone());
            assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                redeem_id: redeem_id,
                reimburse: true
            })
            .dispatch(origin_of(account_of(USER))));
            get_additional_collateral(&vault_id);
            SecurityPallet::set_active_block_number(100000000);
            assert_ok!(RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                currency_pair: vault_id.currencies.clone(),
                redeem_id: redeem_id
            })
            .dispatch(origin_of(account_of(VAULT))));
            ParachainState::get(&vault_id)
        });
        // scenario 3: insufficient collateral but only due to custom threshold
        let result3 = test_with(|vault_id| {
            let redeem_id = setup_cancelable_redeem_with_insufficient_collateral_for_reimburse(vault_id.clone());
            // add a bit of collateral, but not too much
            assert_ok!(VaultRegistryPallet::transfer_funds(
                CurrencySource::FreeBalance(account_of(FAUCET)),
                CurrencySource::Collateral(vault_id.clone()),
                &Amount::new(100_000, vault_id.collateral_currency()),
            ));

            assert_ok!(
                RuntimeCall::VaultRegistry(VaultRegistryCall::set_custom_secure_threshold {
                    currency_pair: vault_id.currencies.clone(),
                    custom_threshold: UnsignedFixedPoint::checked_from_rational(200, 1),
                })
                .dispatch(origin_of(vault_id.account_id.clone()))
            );
            assert_ok!(RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                redeem_id: redeem_id,
                reimburse: true
            })
            .dispatch(origin_of(account_of(USER))));
            get_additional_collateral(&vault_id);
            SecurityPallet::set_active_block_number(100000000);
            assert_ok!(RuntimeCall::Redeem(RedeemCall::mint_tokens_for_reimbursed_redeem {
                currency_pair: vault_id.currencies.clone(),
                redeem_id: redeem_id
            })
            .dispatch(origin_of(account_of(VAULT))));
            ParachainState::get(&vault_id)
        });
        // the states should be identical
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }
}

#[test]
fn liquidation_redeem_should_be_possible_with_zero_collateral() {
    test_with(|vault_id| {
        let redeem_amount = Amount::new(1000, vault_id.wrapped_currency());

        let mut liquidation_vault_data = LiquidationVaultData::get();
        let liquidation_vault = liquidation_vault_data
            .liquidation_vaults
            .get_mut(&vault_id.currencies)
            .unwrap();
        (*liquidation_vault).collateral = Amount::new(0, vault_id.collateral_currency());
        (*liquidation_vault).issued = redeem_amount;
        (*liquidation_vault).to_be_issued = Amount::new(0, vault_id.wrapped_currency());
        (*liquidation_vault).to_be_redeemed = Amount::new(0, vault_id.wrapped_currency());

        LiquidationVaultData::force_to(liquidation_vault_data);

        let post_liquidation_state = ParachainState::get(&vault_id);

        assert_ok!(RuntimeCall::Redeem(RedeemCall::liquidation_redeem {
            currencies: vault_id.currencies.clone(),
            amount_wrapped: redeem_amount.amount(),
        })
        .dispatch(origin_of(account_of(USER))));

        assert_eq!(
            ParachainState::get(&vault_id),
            post_liquidation_state.with_changes(|user, _vault, liquidation_vault, _fee_pool| {
                let liquidation_vault = liquidation_vault.with_currency(&vault_id.currencies);
                liquidation_vault.issued -= redeem_amount;

                (*user.balances.get_mut(&vault_id.wrapped_currency()).unwrap()).free -= redeem_amount;
            })
        );
    })
}

mod self_redeem {
    use super::{assert_eq, *};

    #[test]
    fn integration_test_self_redeem_with_partial_amount_succeeds() {
        test_with(|vault_id| {
            let issued_tokens = vault_id.wrapped(10_000);

            assert_ok!(RuntimeCall::Redeem(RedeemCall::self_redeem {
                currency_pair: vault_id.currencies.clone(),
                amount_wrapped: issued_tokens.amount()
            })
            .dispatch(origin_of(vault_id.account_id.clone())));

            let (fee, consumed_issued_tokens) = assert_self_redeem_event();

            assert_eq!(
                ParachainState::get(&vault_id),
                ParachainState::get_default(&vault_id).with_changes(|_, vault, _, fee_pool| {
                    vault.issued -= consumed_issued_tokens;
                    (*vault.free_balance.get_mut(&vault_id.wrapped_currency()).unwrap()) -= issued_tokens;
                    *fee_pool.rewards_for(&vault_id) += fee;
                    consume_to_be_replaced(vault, consumed_issued_tokens);
                })
            );
        });
    }
    #[test]
    fn integration_test_self_redeem_with_full_amount_succeeds() {
        test_with(|vault_id| {
            let issued_tokens = default_vault_state(&vault_id).issued - default_vault_state(&vault_id).to_be_redeemed;

            assert_ok!(RuntimeCall::Redeem(RedeemCall::self_redeem {
                currency_pair: vault_id.currencies.clone(),
                amount_wrapped: issued_tokens.amount()
            })
            .dispatch(origin_of(vault_id.account_id.clone())));

            assert_self_redeem_event();

            assert_eq!(
                ParachainState::get(&vault_id),
                ParachainState::get_default(&vault_id).with_changes(|_, vault, _, _| {
                    vault.issued -= issued_tokens;
                    (*vault.free_balance.get_mut(&vault_id.wrapped_currency()).unwrap()) -= issued_tokens;
                    consume_to_be_replaced(vault, issued_tokens);
                })
            );
        });
    }

    #[test]
    fn integration_test_self_redeem_with_higher_issued_tokens_fails() {
        test_with(|vault_id| {
            let issue_amount = 999999999;

            assert_ok!(RuntimeCall::Tokens(TokensCall::set_balance {
                who: vault_id.account_id.clone(),
                currency_id: vault_id.wrapped_currency(),
                new_free: issue_amount,
                new_reserved: 0,
            })
            .dispatch(root()));

            assert_err!(
                RuntimeCall::Redeem(RedeemCall::self_redeem {
                    currency_pair: vault_id.currencies.clone(),
                    amount_wrapped: issue_amount
                })
                .dispatch(origin_of(vault_id.account_id.clone())),
                VaultRegistryError::InsufficientTokensCommitted
            );
        })
    }

    #[test]
    fn integration_test_self_redeem_with_higher_than_free_balance_fails() {
        test_with(|vault_id| {
            let issue_amount = 1000;

            assert_ok!(RuntimeCall::Tokens(TokensCall::set_balance {
                who: vault_id.account_id.clone(),
                currency_id: vault_id.wrapped_currency(),
                new_free: issue_amount - 1,
                new_reserved: 0,
            })
            .dispatch(root()));

            assert_err!(
                RuntimeCall::Redeem(RedeemCall::self_redeem {
                    currency_pair: vault_id.currencies.clone(),
                    amount_wrapped: issue_amount
                })
                .dispatch(origin_of(vault_id.account_id.clone())),
                RedeemError::AmountExceedsUserBalance
            );
        })
    }

    #[test]
    fn integration_test_self_redeem_with_zero_amount_fails() {
        test_with(|vault_id| {
            assert_err!(
                RuntimeCall::Redeem(RedeemCall::self_redeem {
                    currency_pair: vault_id.currencies.clone(),
                    amount_wrapped: 0
                })
                .dispatch(origin_of(vault_id.account_id.clone())),
                RedeemError::AmountBelowDustAmount
            );
        })
    }
}

mod oracle_down {
    use super::{assert_eq, *};

    #[test]
    fn no_oracle_request_redeem_fails() {
        test_with(|vault_id| {
            OraclePallet::expire_all();

            assert_noop!(
                RuntimeCall::Redeem(RedeemCall::request_redeem {
                    amount_wrapped: 10_000,
                    btc_address: USER_BTC_ADDRESS,
                    vault_id: vault_id.clone()
                })
                .dispatch(origin_of(account_of(USER))),
                OracleError::MissingExchangeRate
            );
        });
    }

    #[test]
    fn no_oracle_execute_redeem_succeeds() {
        test_with(|vault_id| {
            let redeem_id = setup_redeem(vault_id.wrapped(10_000), USER, &vault_id);
            let redeem = RedeemPallet::get_open_redeem_request_from_id(&redeem_id).unwrap();

            OraclePallet::expire_all();

            ExecuteRedeemBuilder::new(redeem_id)
                .with_amount(redeem.amount_btc())
                .assert_execute();
        });
    }

    #[test]
    fn no_oracle_cancel_redeem_reimburse_fails() {
        test_with(|vault_id| {
            let amount_btc = vault_id.wrapped(10000);
            let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);

            OraclePallet::expire_all();

            assert_noop!(
                RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: true
                })
                .dispatch(origin_of(account_of(USER))),
                OracleError::MissingExchangeRate
            );
        });
    }

    #[test]
    fn no_oracle_cancel_redeem_retry_fails() {
        test_with(|vault_id| {
            let amount_btc = vault_id.wrapped(10000);
            let redeem_id = setup_cancelable_redeem(USER, &vault_id, amount_btc);

            OraclePallet::expire_all();

            assert_noop!(
                RuntimeCall::Redeem(RedeemCall::cancel_redeem {
                    redeem_id: redeem_id,
                    reimburse: false
                })
                .dispatch(origin_of(account_of(USER))),
                OracleError::MissingExchangeRate
            );
        });
    }
}
