//! Runtime API definition for the Reward Module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use frame_support::dispatch::DispatchError;
use oracle_rpc_runtime_api::BalanceWrapper;

sp_api::decl_runtime_apis! {
    pub trait RewardApi<AccountId, VaultId, CurrencyId, Balance, BlockNumber, UnsignedFixedPoint> where
        AccountId: Codec,
        VaultId: Codec,
        CurrencyId: Codec,
        Balance: Codec,
        BlockNumber: Codec,
        UnsignedFixedPoint: Codec,
    {
        /// Calculate the number of escrow rewards accrued
        fn compute_escrow_reward(account_id: AccountId, currency_id: CurrencyId) -> Result<BalanceWrapper<Balance>, DispatchError>;

        /// Calculate the number of farming rewards accrued
        fn compute_farming_reward(account_id: AccountId, pool_currency_id: CurrencyId, reward_currency_id: CurrencyId) -> Result<BalanceWrapper<Balance>, DispatchError>;

        /// Calculate the number of vault rewards accrued
        fn compute_vault_reward(vault_id: VaultId, currency_id: CurrencyId) -> Result<BalanceWrapper<Balance>, DispatchError>;

        /// Estimate staking reward rate for a one year period
        fn estimate_escrow_reward_rate(account_id: AccountId, amount: Option<Balance>, lock_time: Option<BlockNumber>) -> Result<UnsignedFixedPoint, DispatchError>;

        /// Estimate farming rewards for remaining incentives
        fn estimate_farming_reward(account_id: AccountId, pool_currency_id: CurrencyId, reward_currency_id: CurrencyId) -> Result<BalanceWrapper<Balance>, DispatchError>;

        /// Estimate vault reward rate for a one year period
        fn estimate_vault_reward_rate(vault_id: VaultId) -> Result<UnsignedFixedPoint, DispatchError>;
    }
}
