#[cfg(test)]
use mocktopus::macros::mockable;

#[cfg_attr(test, mockable)]
pub(crate) mod currency {
    use crate::types::CurrencyId;
    use currency::Amount;

    pub fn get_free_balance<T: crate::Config>(currency_id: CurrencyId<T>, id: &T::AccountId) -> Amount<T> {
        currency::get_free_balance::<T>(currency_id, id)
    }

    pub fn get_reserved_balance<T: crate::Config>(currency_id: CurrencyId<T>, id: &T::AccountId) -> Amount<T> {
        currency::get_reserved_balance::<T>(currency_id, id)
    }
}

#[cfg_attr(test, mockable)]
pub(crate) mod security {
    use frame_system::pallet_prelude::BlockNumberFor;

    pub fn active_block_number<T: crate::Config>() -> BlockNumberFor<T> {
        <security::Pallet<T>>::active_block_number()
    }
}

#[cfg_attr(test, mockable)]
pub(crate) mod staking {
    use crate::{types::BalanceOf, DefaultVaultId};
    use currency::Amount;
    use frame_support::dispatch::{DispatchError, DispatchResult};
    use staking::{RewardsApi, StakingApi};

    pub fn deposit_stake<T: crate::Config>(
        vault_id: &DefaultVaultId<T>,
        nominator_id: &T::AccountId,
        amount: &Amount<T>,
    ) -> DispatchResult {
        T::VaultStaking::deposit_stake(&(None, vault_id.clone()), nominator_id, amount.amount())
    }

    pub fn withdraw_stake<T: crate::Config>(
        vault_id: &DefaultVaultId<T>,
        nominator_id: &T::AccountId,
        maybe_amount: Option<Amount<T>>,
        nonce: Option<<T as frame_system::Config>::Nonce>,
    ) -> Result<Amount<T>, DispatchError> {
        if let Some(amount) = maybe_amount {
            T::VaultStaking::withdraw_stake(&(nonce, vault_id.clone()), nominator_id, amount.amount())?;
            Ok(amount)
        } else {
            let balance = T::VaultStaking::withdraw_all_stake(&(nonce, vault_id.clone()), nominator_id)?;
            Ok(Amount::new(balance, vault_id.collateral_currency()))
        }
    }

    pub fn slash_stake<T: crate::Config>(vault_id: &DefaultVaultId<T>, amount: &Amount<T>) -> DispatchResult {
        T::VaultStaking::slash_stake(vault_id, amount.amount())
    }

    pub fn force_refund<T: crate::Config>(vault_id: &DefaultVaultId<T>) -> Result<Amount<T>, DispatchError> {
        let amount = T::VaultStaking::force_refund(vault_id)?;
        Ok(Amount::<T>::new(amount, vault_id.collateral_currency()))
    }

    pub fn compute_stake<T: crate::Config>(
        vault_id: &DefaultVaultId<T>,
        nominator_id: &T::AccountId,
    ) -> Result<BalanceOf<T>, DispatchError> {
        T::VaultStaking::get_stake(&(None, vault_id.clone()), nominator_id)
    }

    pub fn total_current_stake<T: crate::Config>(vault_id: &DefaultVaultId<T>) -> Result<Amount<T>, DispatchError> {
        let amount = T::VaultStaking::get_total_stake(&(None, vault_id.clone()))?;
        Ok(Amount::<T>::new(amount, vault_id.collateral_currency()))
    }
}

#[cfg_attr(test, mockable)]
pub(crate) mod reward {
    use crate::{CurrencyId, DefaultVaultId};
    use currency::Amount;
    use frame_support::dispatch::DispatchError;
    use reward::RewardsApi;

    pub fn set_stake<T: crate::Config>(vault_id: &DefaultVaultId<T>, amount: &Amount<T>) -> Result<(), DispatchError> {
        T::VaultRewards::set_stake(&vault_id.collateral_currency(), vault_id, amount.amount())
    }

    pub fn total_current_stake<T: crate::Config>(currency_id: CurrencyId<T>) -> Result<Amount<T>, DispatchError> {
        let amount = T::VaultRewards::get_total_stake(&currency_id)?;
        Ok(Amount::<T>::new(amount, currency_id))
    }

    #[cfg(feature = "integration-tests")]
    pub fn get_stake<T: crate::Config>(vault_id: &DefaultVaultId<T>) -> Result<crate::BalanceOf<T>, DispatchError> {
        T::VaultRewards::get_stake(&vault_id.collateral_currency(), vault_id)
    }
}

#[cfg_attr(test, mockable)]
pub(crate) mod capacity {
    use crate::types::CurrencyId;
    use currency::Amount;
    use frame_support::dispatch::DispatchError;
    use staking::RewardsApi;

    pub fn set_stake<T: crate::Config>(currency_id: CurrencyId<T>, amount: &Amount<T>) -> Result<(), DispatchError> {
        T::CapacityRewards::set_stake(&(), &currency_id, amount.amount())
    }
}

#[cfg_attr(test, mockable)]
pub(crate) mod oracle {
    use frame_support::dispatch::DispatchError;
    use oracle::{types::UnsignedFixedPoint, OracleKey};

    pub fn get_price<T: crate::Config>(key: OracleKey) -> Result<UnsignedFixedPoint<T>, DispatchError> {
        <oracle::Pallet<T>>::get_price(key)
    }
}

#[cfg_attr(test, mockable)]
pub(crate) mod fee {
    use crate::DefaultVaultId;
    use fee::types::UnsignedFixedPoint;
    use frame_support::dispatch::DispatchResult;

    pub fn distribute_all_vault_rewards<T: crate::Config>(vault_id: &DefaultVaultId<T>) -> DispatchResult {
        <fee::Pallet<T>>::distribute_all_vault_rewards(vault_id)
    }

    pub fn premium_redeem_reward_rate<T: crate::Config>() -> UnsignedFixedPoint<T> {
        <fee::Pallet<T>>::premium_redeem_reward_rate()
    }

    pub fn get_redeem_fee_value<T: crate::Config>() -> UnsignedFixedPoint<T> {
        <fee::Pallet<T>>::get_redeem_fee_value()
    }
}
