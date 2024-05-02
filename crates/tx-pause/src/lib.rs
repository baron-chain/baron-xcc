// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Transaction Pause Pallet
//!
//! The Transaction Pause pallet provides a dynamic call filter that can be controlled with
//! extrinsics. This pallet may be used to disable dispatch of specific calls within a runtime.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The Transaction Pause pallet provides functions for:
//!
//! - Setting a dynamic list of [`FullNameOf`] items that are matched against to filter these calls.
//! - Setting [`Config::WhitelistCallNames`] that cannot be paused by this pallet.
//! - Repatriating a reserved balance to a beneficiary account that exists.
//! - Transferring a balance between accounts (when not reserved).
//! - Slashing an account balance.
//! - Account creation and removal.
//! - Managing total issuance.
//! - Setting and managing locks.
//!
//! Can also be used as call-filter by the runtime together with the SafeMode
//!
//! TODO expand an link to safe mode in docs.
//!
//! ### Terminology
//!
//! - **Pause**: The ability to dispatch of a specific call becomes disabled.
//! - **Unpause**: The ability to dispatch of a specific call becomes enabled, if it was paused.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `pause` - Pause a pallet or optionally a specific call within a pallet.
//! - `unpause` - Unpause a pallet or optionally a specific call within a pallet.
//!
//! ## Usage
//!
//! The following examples show how to use the Transaction Pause pallet in your custom pallet.
//! TODO check doc links
//! ### Example within a runtime's [`pallet-frame-system::BaseCallFilter`] configuration:
//!
//! ```ignore
//! impl frame_system::Config for Runtime {
//! 	…
//! 	type BaseCallFilter = InsideBoth<DefaultFilter, InsideBoth<TxPause, SafeMode>>;
//! 	…
//! }
//! ```
//!
//! ## Genesis config
//!
//! The Transaction Pause pallet may be configured to pause calls on genesis with it's
//! [`GenesisConfig`].
//!
//! ## Assumptions
//!
//! * TODO

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

use frame_support::{
    dispatch::GetDispatchInfo,
    pallet_prelude::*,
    traits::{CallMetadata, Contains, GetCallMetadata, IsSubType, IsType},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::Dispatchable, DispatchResult};
use sp_std::{convert::TryInto, prelude::*};

pub use pallet::*;
pub use weights::*;

/// The stringy name of a pallet from [`GetCallMetadata`] for [`Config::RuntimeCall`] variants.
pub type PalletNameOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;
/// The stringy name of a call (within a pallet) from [`GetCallMetadata`] for
/// [`Config::RuntimeCall`] variants.
pub type CallNameOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;
/// A sully specified pallet ([`PalletNameOf`]) and optional call ([`CallNameOf`])
/// to partially or fully specify an item a variant of a  [`Config::RuntimeCall`].
pub type FullNameOf<T> = (PalletNameOf<T>, Option<CallNameOf<T>>);

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
            + GetCallMetadata
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The only origin that can pause calls.
        type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The only origin that can un-pause calls.
        type UnpauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Contains all calls that cannot be paused.
        ///
        /// The `TxMode` pallet cannot pause it's own calls, and does not need to be explicitly
        /// added here.
        type WhitelistCallNames: Contains<FullNameOf<Self>>;

        /// Maximum length for pallet and call SCALE encoded string names.
        ///
        /// Too long names will not be truncated but handled like
        /// [`Self::PauseTooLongNames`] specifies.
        #[pallet::constant]
        type MaxNameLen: Get<u32>;

        /// Specifies if functions and pallets with too long names should be treated as paused.
        ///
        /// Setting this to `true` ensures that all calls that
        /// are callable, are also pause-able.
        /// Otherwise there could be a situation where a call
        /// is callable but not pause-able, which would could be exploited.
        #[pallet::constant]
        type PauseTooLongNames: Get<bool>;

        // Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The call is (already or still) paused.
        IsPaused,

        /// The call is (already or still) unpaused.
        IsUnpaused,

        /// The call is listed as safe and cannot be paused.
        IsUnpausable,

        // The pallet or call does not exist in the runtime.
        NotFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// This pallet, or a specific call is now paused. \[pallet_name, Option<call_name>\]
        SomethingPaused { full_name: FullNameOf<T> },
        /// This pallet, or a specific call is now unpaused. \[pallet_name, Option<call_name>\]
        SomethingUnpaused { full_name: FullNameOf<T> },
    }

    /// The set of calls that are explicitly paused.
    #[pallet::storage]
    #[pallet::getter(fn paused_calls)]
    pub type PausedCalls<T: Config> = StorageMap<_, Blake2_128Concat, FullNameOf<T>, (), OptionQuery>;

    /// Configure the initial state of this pallet in the genesis block.
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The initially paused calls.
        pub paused: Vec<FullNameOf<T>>,
        pub _phantom: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (pallet_name, maybe_call_name) in &self.paused {
                PausedCalls::<T>::insert((pallet_name, maybe_call_name), ());
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Pause a call.
        ///
        /// Can only be called by [`Config::PauseOrigin`].
        /// Emits an [`Event::SomethingPaused`] event on success.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::pause())]
        pub fn pause(origin: OriginFor<T>, full_name: FullNameOf<T>) -> DispatchResult {
            T::PauseOrigin::ensure_origin(origin)?;

            Self::ensure_can_pause(&full_name)?;
            PausedCalls::<T>::insert(&full_name, ());
            Self::deposit_event(Event::SomethingPaused { full_name });

            Ok(())
        }

        /// Un-pause a call.
        ///
        /// Can only be called by [`Config::UnpauseOrigin`].
        /// Emits an [`Event::SomethingUnpaused`] event on success.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unpause())]
        pub fn unpause(origin: OriginFor<T>, full_name: FullNameOf<T>) -> DispatchResult {
            T::UnpauseOrigin::ensure_origin(origin)?;

            Self::ensure_can_unpause(&full_name)?;
            PausedCalls::<T>::remove(&full_name);
            Self::deposit_event(Event::SomethingUnpaused { full_name });
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Return whether this pallet or call is paused.
    pub fn is_paused_unbound(pallet_name: Vec<u8>, call_name: Vec<u8>) -> bool {
        let pallet_name = PalletNameOf::<T>::try_from(pallet_name);
        let call_name = CallNameOf::<T>::try_from(call_name);

        match (pallet_name, call_name) {
            (Ok(pallet_name), Ok(call_name)) => {
                Self::is_paused(&&<FullNameOf<T>>::from((pallet_name.clone(), Some(call_name.clone()))))
            }
            _ => T::PauseTooLongNames::get(),
        }
    }

    /// Return whether this pallet or call is paused
    pub fn is_paused(full_name: &FullNameOf<T>) -> bool {
        let (pallet_name, maybe_call_name) = full_name;
        // SAFETY: Everything that is whitelisted cannot be paused,
        // including calls within paused pallets.
        if T::WhitelistCallNames::contains(&<FullNameOf<T>>::from((pallet_name.clone(), maybe_call_name.clone()))) {
            return false;
        };
        // Check is pallet is paused.
        if <PausedCalls<T>>::contains_key(<FullNameOf<T>>::from((pallet_name.clone(), None))) {
            return true;
        };
        // Check if call in a pallet is paused
        <PausedCalls<T>>::contains_key(full_name)
    }

    /// Ensure that this pallet or call can be paused.
    pub fn ensure_can_pause(full_name: &FullNameOf<T>) -> Result<(), Error<T>> {
        // The `TxPause` pallet can never be paused.
        if full_name.0.as_ref() == <Self as PalletInfoAccess>::name().as_bytes().to_vec() {
            return Err(Error::<T>::IsUnpausable);
        }
        if Self::is_paused(&full_name) {
            return Err(Error::<T>::IsPaused);
        }
        if T::WhitelistCallNames::contains(&full_name) {
            return Err(Error::<T>::IsUnpausable);
        }
        Ok(())
    }

    /// Ensure that this call can be un-paused.
    pub fn ensure_can_unpause(full_name: &FullNameOf<T>) -> Result<(), Error<T>> {
        if Self::is_paused(&full_name) {
            // SAFETY: Everything that is paused, can be un-paused.
            Ok(())
        } else {
            Err(Error::IsUnpaused)
        }
    }
}

impl<T: pallet::Config> Contains<<T as frame_system::Config>::RuntimeCall> for Pallet<T>
where
    <T as frame_system::Config>::RuntimeCall: GetCallMetadata,
{
    /// Return whether the call is allowed to be dispatched.
    fn contains(call: &<T as frame_system::Config>::RuntimeCall) -> bool {
        let CallMetadata {
            pallet_name,
            function_name,
        } = call.get_call_metadata();
        !Pallet::<T>::is_paused_unbound(pallet_name.into(), function_name.into())
    }
}
