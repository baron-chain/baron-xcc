//! # Annuity Module
//! Distributes block rewards to participants.

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

use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
    transactional,
    weights::Weight,
    PalletId,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::{AccountIdConversion, CheckedDiv, Convert, Saturating};
use sp_std::cmp::min;

pub use pallet::*;

type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};

    /// ## Configuration
    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The annuity module id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type AnnuityPalletId: Get<PalletId>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The native currency for emission.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The block reward provider.
        type BlockRewardProvider: BlockRewardProvider<Self::AccountId, Currency = Self::Currency>;

        /// Convert the block number into a balance.
        type BlockNumberToBalance: Convert<BlockNumberFor<Self>, BalanceOf<Self, I>>;

        /// The emission period for block rewards.
        #[pallet::constant]
        type EmissionPeriod: Get<BlockNumberFor<Self>>;

        /// The total amount of the wrapped asset.
        type TotalWrapped: Get<BalanceOf<Self, I>>;

        /// Weight information for the extrinsics in this module.
        type WeightInfo: WeightInfo;
    }

    // The pallet's events
    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        BlockReward(BalanceOf<T, I>),
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {}

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            if let Err(e) = Self::begin_block(n) {
                sp_runtime::print(e);
            }
            <T as Config<I>>::WeightInfo::on_initialize()
        }
    }

    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn reward_per_block)]
    pub type RewardPerBlock<T: Config<I>, I: 'static = ()> = StorageValue<_, BalanceOf<T, I>, ValueQuery>;

    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn reward_per_wrapped)]
    pub type RewardPerWrapped<T: Config<I>, I: 'static = ()> = StorageValue<_, BalanceOf<T, I>, OptionQuery>;

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    // The pallet's dispatchable functions.
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::withdraw_rewards())]
        #[transactional]
        pub fn withdraw_rewards(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let dest = ensure_signed(origin)?;
            let value = T::BlockRewardProvider::withdraw_reward(&dest)?;
            T::Currency::transfer(&Self::account_id(), &dest, value, ExistenceRequirement::AllowDeath)?;
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_rewards())]
        #[transactional]
        pub fn update_rewards(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::update_reward_per_block();
            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::set_reward_per_wrapped())]
        #[transactional]
        pub fn set_reward_per_wrapped(
            origin: OriginFor<T>,
            reward_per_wrapped: BalanceOf<T, I>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            RewardPerWrapped::<T, I>::put(reward_per_wrapped);
            Ok(().into())
        }
    }
}

// "Internal" functions, callable by code.
impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn account_id() -> T::AccountId {
        T::AnnuityPalletId::get().into_account_truncating()
    }

    pub fn min_reward_per_block() -> BalanceOf<T, I> {
        let reward_per_block = Self::reward_per_block();
        match Self::reward_per_wrapped() {
            Some(reward_per_wrapped) => min(
                reward_per_block,
                reward_per_wrapped.saturating_mul(T::TotalWrapped::get()),
            ),
            None => reward_per_block,
        }
    }

    pub(crate) fn begin_block(_height: BlockNumberFor<T>) -> DispatchResult {
        let reward_per_block = Self::min_reward_per_block();
        Self::deposit_event(Event::<T, I>::BlockReward(reward_per_block));
        T::BlockRewardProvider::distribute_block_reward(&Self::account_id(), reward_per_block)
    }

    pub fn update_reward_per_block() {
        let emission_period = T::BlockNumberToBalance::convert(T::EmissionPeriod::get());
        let total_balance = T::Currency::total_balance(&Self::account_id());
        let reward_per_block = total_balance.checked_div(&emission_period).unwrap_or_default();
        RewardPerBlock::<T, I>::put(reward_per_block);
    }
}

pub trait BlockRewardProvider<AccountId> {
    type Currency: ReservableCurrency<AccountId>;
    #[cfg(any(feature = "runtime-benchmarks", test))]
    fn deposit_stake(who: &AccountId, amount: <Self::Currency as Currency<AccountId>>::Balance) -> DispatchResult;
    fn distribute_block_reward(
        from: &AccountId,
        amount: <Self::Currency as Currency<AccountId>>::Balance,
    ) -> DispatchResult;
    #[cfg(any(feature = "runtime-benchmarks", test))]
    fn can_withdraw_reward() -> bool {
        true
    }
    fn withdraw_reward(who: &AccountId) -> Result<<Self::Currency as Currency<AccountId>>::Balance, DispatchError>;
}
