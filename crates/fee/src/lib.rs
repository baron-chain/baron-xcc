//! # Fee Module
//! Based on the [specification](https://spec.interlay.io/spec/fee.html).

#![deny(warnings)]
#![cfg_attr(test, feature(proc_macro_hygiene))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod default_weights;
pub use default_weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;

#[cfg(test)]
extern crate mocktopus;

#[cfg(test)]
use mocktopus::macros::mockable;

use codec::{Decode, Encode, EncodeLike};
use currency::{Amount, CurrencyId, OnSweep, Rounding};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    storage,
    traits::Get,
    transactional, PalletId,
};
use frame_system::ensure_signed;
pub use pallet::*;
use primitives::VaultId;
use reward::RewardsApi;
use sp_arithmetic::{traits::*, FixedPointNumber};
use sp_runtime::{traits::AccountIdConversion, TransactionOutcome};
use sp_std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
};
use staking::StakingApi;
use types::{BalanceOf, DefaultVaultCurrencyPair, DefaultVaultId, UnsignedFixedPoint, Version};

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use traits::NominationApi;

    /// ## Configuration
    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config + security::Config + currency::Config {
        /// The fee module id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type FeePalletId: Get<PalletId>;

        /// Weight information for the extrinsics in this module.
        type WeightInfo: WeightInfo;

        /// Signed fixed point type.
        type SignedFixedPoint: FixedPointNumber<Inner = <Self as Config>::SignedInner> + Encode + EncodeLike + Decode;

        /// The `Inner` type of the `SignedFixedPoint`.
        type SignedInner: Debug
            + CheckedDiv
            + TryFrom<BalanceOf<Self>>
            + TryInto<BalanceOf<Self>>
            + MaybeSerializeDeserialize;

        /// Capacity reward pool.
        type CapacityRewards: RewardsApi<(), CurrencyId<Self>, BalanceOf<Self>, CurrencyId = CurrencyId<Self>>;

        /// Vault reward pool.
        type VaultRewards: RewardsApi<
            CurrencyId<Self>,
            DefaultVaultId<Self>,
            BalanceOf<Self>,
            CurrencyId = CurrencyId<Self>,
        >;

        /// Vault staking pool.
        type VaultStaking: StakingApi<DefaultVaultId<Self>, Self::Nonce, BalanceOf<Self>>
            + RewardsApi<
                (Option<Self::Nonce>, DefaultVaultId<Self>),
                Self::AccountId,
                BalanceOf<Self>,
                CurrencyId = CurrencyId<Self>,
            >;

        /// Handler to transfer undistributed rewards.
        type OnSweep: OnSweep<Self::AccountId, Amount<Self>>;

        /// Maximum expected value to set the storage fields to.
        #[pallet::constant]
        type MaxExpectedValue: Get<UnsignedFixedPoint<Self>>;

        /// Api of the nomination pallet; used to make sure that commission can only be set if opted in.
        type NominationApi: NominationApi<DefaultVaultId<Self>, Amount<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Unable to convert value.
        TryIntoIntError,
        /// Value exceeds the expected upper bound for storage fields in this pallet.
        AboveMaxExpectedValue,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    /// # Issue

    /// Fee share that users need to pay to issue tokens.
    #[pallet::storage]
    #[pallet::getter(fn issue_fee)]
    pub type IssueFee<T: Config> = StorageValue<_, UnsignedFixedPoint<T>, ValueQuery>;

    /// Default griefing collateral (e.g. DOT/KSM) as a percentage of the locked
    /// collateral of a Vault a user has to lock to issue tokens.
    #[pallet::storage]
    #[pallet::getter(fn issue_griefing_collateral)]
    pub type IssueGriefingCollateral<T: Config> = StorageValue<_, UnsignedFixedPoint<T>, ValueQuery>;

    /// # Redeem

    /// Fee share that users need to pay to redeem tokens.
    #[pallet::storage]
    #[pallet::getter(fn redeem_fee)]
    pub type RedeemFee<T: Config> = StorageValue<_, UnsignedFixedPoint<T>, ValueQuery>;

    /// # Vault Registry

    /// If users execute a redeem with a Vault flagged for premium redeem,
    /// they can earn a collateral premium, slashed from the Vault.
    #[pallet::storage]
    #[pallet::getter(fn premium_redeem_fee)]
    pub type PremiumRedeemFee<T: Config> = StorageValue<_, UnsignedFixedPoint<T>, ValueQuery>;

    /// Fee that a Vault has to pay if it fails to execute redeem or replace requests
    /// (for redeem, on top of the slashed value of the request). The fee is
    /// paid in collateral based on the token amount at the current exchange rate.
    #[pallet::storage]
    #[pallet::getter(fn punishment_fee)]
    pub type PunishmentFee<T: Config> = StorageValue<_, UnsignedFixedPoint<T>, ValueQuery>;

    /// # Replace

    /// Default griefing collateral (e.g. DOT/KSM) as a percentage of the to-be-locked collateral
    /// of the new Vault. This collateral will be slashed and allocated to the replacing Vault
    /// if the to-be-replaced Vault does not transfer BTC on time.
    #[pallet::storage]
    #[pallet::getter(fn replace_griefing_collateral)]
    pub type ReplaceGriefingCollateral<T: Config> = StorageValue<_, UnsignedFixedPoint<T>, ValueQuery>;

    #[pallet::type_value]
    pub(super) fn DefaultForStorageVersion() -> Version {
        Version::V0
    }

    /// Build storage at V1 (requires default 0).
    #[pallet::storage]
    #[pallet::getter(fn storage_version)]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery, DefaultForStorageVersion>;

    /// The fraction up rewards going straight to the vault operator. The rest goes to the vault's pool.
    #[pallet::storage]
    pub(super) type Commission<T: Config> =
        StorageMap<_, Blake2_128Concat, DefaultVaultId<T>, UnsignedFixedPoint<T>, OptionQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub issue_fee: UnsignedFixedPoint<T>,
        pub issue_griefing_collateral: UnsignedFixedPoint<T>,
        pub redeem_fee: UnsignedFixedPoint<T>,
        pub premium_redeem_fee: UnsignedFixedPoint<T>,
        pub punishment_fee: UnsignedFixedPoint<T>,
        pub replace_griefing_collateral: UnsignedFixedPoint<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            IssueFee::<T>::put(self.issue_fee);
            IssueGriefingCollateral::<T>::put(self.issue_griefing_collateral);
            RedeemFee::<T>::put(self.redeem_fee);
            PremiumRedeemFee::<T>::put(self.premium_redeem_fee);
            PunishmentFee::<T>::put(self.punishment_fee);
            ReplaceGriefingCollateral::<T>::put(self.replace_griefing_collateral);
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // The pallet's dispatchable functions.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Withdraw all rewards from the `origin` account in the `vault_id` staking pool.
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_rewards())]
        #[transactional]
        pub fn withdraw_rewards(
            origin: OriginFor<T>,
            vault_id: DefaultVaultId<T>,
            index: Option<T::Nonce>,
        ) -> DispatchResultWithPostInfo {
            let nominator_id = ensure_signed(origin)?;
            for currency_id in [vault_id.wrapped_currency(), T::GetNativeCurrencyId::get()] {
                Self::withdraw_vault_rewards(&vault_id, &nominator_id, index, currency_id)?;
            }
            Ok(().into())
        }

        /// Changes the issue fee percentage (only executable by the Root account)
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        /// * `fee` - the new fee
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::set_issue_fee())]
        #[transactional]
        pub fn set_issue_fee(origin: OriginFor<T>, fee: UnsignedFixedPoint<T>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(fee <= Self::get_max_expected_value(), Error::<T>::AboveMaxExpectedValue);
            IssueFee::<T>::put(fee);
            Ok(().into())
        }

        /// Changes the issue griefing collateral percentage (only executable by the Root account)
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        /// * `griefing_collateral` - the new griefing collateral
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_issue_griefing_collateral())]
        #[transactional]
        pub fn set_issue_griefing_collateral(
            origin: OriginFor<T>,
            griefing_collateral: UnsignedFixedPoint<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                griefing_collateral <= Self::get_max_expected_value(),
                Error::<T>::AboveMaxExpectedValue
            );
            IssueGriefingCollateral::<T>::put(griefing_collateral);
            Ok(().into())
        }

        /// Changes the redeem fee percentage (only executable by the Root account)
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        /// * `fee` - the new fee
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::set_redeem_fee())]
        #[transactional]
        pub fn set_redeem_fee(origin: OriginFor<T>, fee: UnsignedFixedPoint<T>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(fee <= Self::get_max_expected_value(), Error::<T>::AboveMaxExpectedValue);
            RedeemFee::<T>::put(fee);
            Ok(().into())
        }

        /// Changes the premium redeem fee percentage (only executable by the Root account)
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        /// * `fee` - the new fee
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_premium_redeem_fee())]
        #[transactional]
        pub fn set_premium_redeem_fee(origin: OriginFor<T>, fee: UnsignedFixedPoint<T>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(fee <= Self::get_max_expected_value(), Error::<T>::AboveMaxExpectedValue);
            PremiumRedeemFee::<T>::put(fee);
            Ok(().into())
        }

        /// Changes the punishment fee percentage (only executable by the Root account)
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        /// * `fee` - the new fee
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::set_punishment_fee())]
        #[transactional]
        pub fn set_punishment_fee(origin: OriginFor<T>, fee: UnsignedFixedPoint<T>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(fee <= Self::get_max_expected_value(), Error::<T>::AboveMaxExpectedValue);
            PunishmentFee::<T>::put(fee);
            Ok(().into())
        }

        /// Changes the replace griefing collateral percentage (only executable by the Root account)
        ///
        /// # Arguments
        ///
        /// * `origin` - signing account
        /// * `griefing_collateral` - the new griefing collateral
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::set_replace_griefing_collateral())]
        #[transactional]
        pub fn set_replace_griefing_collateral(
            origin: OriginFor<T>,
            griefing_collateral: UnsignedFixedPoint<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                griefing_collateral <= Self::get_max_expected_value(),
                Error::<T>::AboveMaxExpectedValue
            );
            ReplaceGriefingCollateral::<T>::put(griefing_collateral);
            Ok(().into())
        }

        /// todo: proper weight
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::set_commission())]
        #[transactional]
        pub fn set_commission(
            origin: OriginFor<T>,
            currencies: DefaultVaultCurrencyPair<T>,
            commission: UnsignedFixedPoint<T>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            let vault_id = VaultId::from_pair(account_id, currencies);
            T::NominationApi::ensure_opted_in_to_nomination(&vault_id)?;
            ensure!(
                commission <= UnsignedFixedPoint::<T>::one(),
                Error::<T>::AboveMaxExpectedValue
            );
            Commission::<T>::insert(vault_id, commission);
            Ok(().into())
        }
    }
}

// "Internal" functions, callable by code.
#[cfg_attr(test, mockable)]
impl<T: Config> Pallet<T> {
    fn get_commission_rate(vault_id: &DefaultVaultId<T>) -> UnsignedFixedPoint<T> {
        Commission::<T>::get(vault_id).unwrap_or(<UnsignedFixedPoint<T>>::zero())
    }
    /// The account ID of the fee pool.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn fee_pool_account_id() -> T::AccountId {
        <T as Config>::FeePalletId::get().into_account_truncating()
    }

    pub fn get_max_expected_value() -> UnsignedFixedPoint<T> {
        <T as Config>::MaxExpectedValue::get()
    }

    // Public functions exposed to other pallets

    /// Distribute rewards to participants.
    ///
    /// # Arguments
    ///
    /// * `amount` - amount of rewards
    pub fn distribute_rewards(amount: &Amount<T>) -> DispatchResult {
        // distribute vault rewards and return leftover
        let remaining = Self::distribute(amount)?;
        if !remaining.is_zero() {
            // sweep the remaining rewards to the treasury if non-zero
            T::OnSweep::on_sweep(&Self::fee_pool_account_id(), remaining)?;
        }
        Ok(())
    }

    /// Calculate the required issue fee in tokens.
    ///
    /// # Arguments
    ///
    /// * `amount` - issue amount in tokens
    pub fn get_issue_fee(amount: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        amount.checked_rounded_mul(&<IssueFee<T>>::get(), Rounding::NearestPrefUp)
    }

    /// Calculate the required issue griefing collateral.
    ///
    /// # Arguments
    ///
    /// * `amount` - issue amount in collateral (at current exchange rate)
    pub fn get_issue_griefing_collateral(amount: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        amount.checked_rounded_mul(&<IssueGriefingCollateral<T>>::get(), Rounding::NearestPrefUp)
    }

    /// Calculate the required redeem fee in tokens. Upon execution, the
    /// rewards should be forwarded to the fee pool instead of being burned.
    ///
    /// # Arguments
    ///
    /// * `amount` - redeem amount in tokens
    pub fn get_redeem_fee(amount: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        amount.checked_rounded_mul(&<RedeemFee<T>>::get(), Rounding::NearestPrefUp)
    }

    /// Calculate the premium redeem fee in collateral for a user to get if redeeming
    /// with a Vault below the premium redeem threshold.
    ///
    /// # Arguments
    ///
    /// * `amount` - amount in collateral (at current exchange rate)
    pub fn get_premium_redeem_fee(amount: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        amount.checked_rounded_mul(&<PremiumRedeemFee<T>>::get(), Rounding::Down)
    }

    /// Get the premium redeem reward rate.
    ///
    /// # Returns
    /// Returns the premium redeem reward rate.
    pub fn premium_redeem_reward_rate() -> UnsignedFixedPoint<T> {
        <PremiumRedeemFee<T>>::get()
    }

    /// Get the fee share that users need to pay to redeem tokens.
    ///
    /// # Returns
    /// Returns the redeem fee.
    pub fn get_redeem_fee_value() -> UnsignedFixedPoint<T> {
        <RedeemFee<T>>::get()
    }
    /// Calculate punishment fee for a Vault that fails to execute a redeem
    /// request before the expiry.
    ///
    /// # Arguments
    ///
    /// * `amount` - amount in collateral (at current exchange rate)
    pub fn get_punishment_fee(amount: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        amount.checked_rounded_mul(&<PunishmentFee<T>>::get(), Rounding::NearestPrefUp)
    }

    /// Calculate the required replace griefing collateral.
    ///
    /// # Arguments
    ///
    /// * `amount` - replace amount in collateral (at current exchange rate)
    pub fn get_replace_griefing_collateral(amount: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        amount.checked_rounded_mul(&<ReplaceGriefingCollateral<T>>::get(), Rounding::NearestPrefUp)
    }

    pub fn compute_vault_rewards(
        vault_id: &DefaultVaultId<T>,
        nominator_id: &T::AccountId,
        currency_id: CurrencyId<T>,
    ) -> Result<Amount<T>, DispatchError> {
        // use a closure so we can use the `?` operator
        let get_rewards = || -> Result<Amount<T>, DispatchError> {
            let balance_before = currency::get_free_balance::<T>(currency_id, nominator_id);
            Self::withdraw_vault_rewards(vault_id, nominator_id, None, currency_id)?;
            let balance_after = currency::get_free_balance::<T>(currency_id, nominator_id);
            let reward = balance_after.saturating_sub(&balance_before)?;
            Ok(reward)
        };

        // don't commit storage changes
        storage::with_transaction(|| TransactionOutcome::Rollback(get_rewards()))
    }

    pub fn distribute_all_vault_rewards(vault_id: &DefaultVaultId<T>) -> DispatchResult {
        for currency_id in [vault_id.wrapped_currency(), T::GetNativeCurrencyId::get()] {
            Self::distribute_vault_rewards(&vault_id, currency_id)?;
        }
        Ok(())
    }

    // Private functions internal to this pallet

    /// Withdraw rewards from a pool and transfer to `account_id`.
    fn withdraw_vault_rewards(
        vault_id: &DefaultVaultId<T>,
        nominator_id: &T::AccountId,
        index: Option<T::Nonce>,
        currency_id: CurrencyId<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Self::distribute_vault_rewards(&vault_id, currency_id)?;

        let rewards = T::VaultStaking::withdraw_reward(&(index, vault_id.clone()), nominator_id, currency_id)?
            .try_into()
            .map_err(|_| Error::<T>::TryIntoIntError)?;
        let amount = Amount::<T>::new(rewards, currency_id);
        amount.transfer(&Self::fee_pool_account_id(), nominator_id)?;

        Ok(rewards)
    }

    fn distribute(reward: &Amount<T>) -> Result<Amount<T>, DispatchError> {
        Ok(
            if let Err(_) = T::CapacityRewards::distribute_reward(&(), reward.currency(), reward.amount()) {
                reward.clone()
            } else {
                Amount::<T>::zero(reward.currency())
            },
        )
    }

    pub fn distribute_vault_rewards(vault_id: &DefaultVaultId<T>, currency_id: CurrencyId<T>) -> DispatchResult {
        let collateral_id = vault_id.collateral_currency();

        // push rewards based on collateral capacity
        let reward = T::CapacityRewards::withdraw_reward(&(), &collateral_id, currency_id)?;
        T::VaultRewards::distribute_reward(&collateral_id, currency_id, reward)?;

        // push rewards based on vault's contribution to that capacity
        let reward = T::VaultRewards::withdraw_reward(&collateral_id, vault_id, currency_id)?;

        let full_amount = Amount::<T>::new(reward, currency_id);

        let commission_rate = Self::get_commission_rate(vault_id);
        let commission = full_amount.checked_mul(&commission_rate)?;
        commission.transfer(&Self::fee_pool_account_id(), &vault_id.account_id)?;

        let remainder = full_amount.checked_sub(&commission)?;

        T::VaultStaking::distribute_reward(&(None, vault_id.clone()), currency_id, remainder.amount())?;

        Ok(())
    }
}
