
//! Autogenerated weights for farming
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-08-07, STEPS: `50`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `interlay-rust-runner-2mz2v-kcxvd`, CPU: `AMD EPYC 7502P 32-Core Processor`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kintsugi-dev"), DB CACHE: 1024

// Executed Command:
// target/release/interbtc-parachain
// benchmark
// pallet
// --pallet
// *
// --extrinsic
// *
// --chain
// kintsugi-dev
// --execution=wasm
// --wasm-execution=compiled
// --steps
// 50
// --repeat
// 10
// --output
// parachain/runtime/kintsugi/src/weights/
// --template
// .deploy/runtime-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for farming using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> farming::WeightInfo for WeightInfo<T> {

	/// Storage: Farming RewardSchedules (r:5 w:0)
	/// Proof: Farming RewardSchedules (max_values: None, max_size: Some(74), added: 2549, mode: MaxEncodedLen)
	/// Storage: FarmingRewards TotalStake (r:2 w:0)
	/// Proof: FarmingRewards TotalStake (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
	/// The range of component `c` is `[1, 4]`.
	fn on_initialize	(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208 + c * (41 ±0)`
		//  Estimated: `3539 + c * (2549 ±0)`
		// Minimum execution time: 28_858_000 picoseconds.
		Weight::from_parts(18_114_846, 3539)
			// Standard Error: 91_877
			.saturating_add(Weight::from_parts(12_423_979, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 2549).saturating_mul(c.into()))
	}
	/// Storage: Tokens Accounts (r:2 w:2)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Farming RewardSchedules (r:1 w:1)
	/// Proof: Farming RewardSchedules (max_values: None, max_size: Some(74), added: 2549, mode: MaxEncodedLen)
	fn update_reward_schedule	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `847`
		//  Estimated: `6170`
		// Minimum execution time: 97_555_000 picoseconds.
		Weight::from_parts(97_986_000, 6170)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: Tokens Accounts (r:2 w:2)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:0)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Farming RewardSchedules (r:0 w:1)
	/// Proof: Farming RewardSchedules (max_values: None, max_size: Some(74), added: 2549, mode: MaxEncodedLen)
	fn remove_reward_schedule	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `945`
		//  Estimated: `6170`
		// Minimum execution time: 76_313_000 picoseconds.
		Weight::from_parts(76_793_000, 6170)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: FarmingRewards RewardCurrencies (r:1 w:0)
	/// Proof: FarmingRewards RewardCurrencies (max_values: None, max_size: Some(138), added: 2613, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: FarmingRewards Stake (r:1 w:1)
	/// Proof: FarmingRewards Stake (max_values: None, max_size: Some(75), added: 2550, mode: MaxEncodedLen)
	/// Storage: FarmingRewards TotalStake (r:1 w:1)
	/// Proof: FarmingRewards TotalStake (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
	/// Storage: FarmingRewards RewardTally (r:4 w:4)
	/// Proof: FarmingRewards RewardTally (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: FarmingRewards RewardPerToken (r:4 w:0)
	/// Proof: FarmingRewards RewardPerToken (max_values: None, max_size: Some(70), added: 2545, mode: MaxEncodedLen)
	/// The range of component `c` is `[1, 4]`.
	fn deposit	(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `900 + c * (70 ±0)`
		//  Estimated: `3603 + c * (2577 ±0)`
		// Minimum execution time: 92_445_000 picoseconds.
		Weight::from_parts(81_740_057, 3603)
			// Standard Error: 37_348
			.saturating_add(Weight::from_parts(12_635_191, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 2577).saturating_mul(c.into()))
	}
	/// Storage: FarmingRewards RewardCurrencies (r:1 w:0)
	/// Proof: FarmingRewards RewardCurrencies (max_values: None, max_size: Some(138), added: 2613, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: FarmingRewards Stake (r:1 w:1)
	/// Proof: FarmingRewards Stake (max_values: None, max_size: Some(75), added: 2550, mode: MaxEncodedLen)
	/// Storage: FarmingRewards TotalStake (r:1 w:1)
	/// Proof: FarmingRewards TotalStake (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
	/// Storage: FarmingRewards RewardTally (r:4 w:4)
	/// Proof: FarmingRewards RewardTally (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: FarmingRewards RewardPerToken (r:4 w:0)
	/// Proof: FarmingRewards RewardPerToken (max_values: None, max_size: Some(70), added: 2545, mode: MaxEncodedLen)
	/// The range of component `c` is `[1, 4]`.
	fn withdraw	(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `900 + c * (70 ±0)`
		//  Estimated: `3603 + c * (2577 ±0)`
		// Minimum execution time: 85_862_000 picoseconds.
		Weight::from_parts(74_571_773, 3603)
			// Standard Error: 47_353
			.saturating_add(Weight::from_parts(12_753_635, 0).saturating_mul(c.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(c.into())))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(c.into())))
			.saturating_add(Weight::from_parts(0, 2577).saturating_mul(c.into()))
	}
	/// Storage: FarmingRewards Stake (r:1 w:0)
	/// Proof: FarmingRewards Stake (max_values: None, max_size: Some(75), added: 2550, mode: MaxEncodedLen)
	/// Storage: FarmingRewards RewardPerToken (r:1 w:0)
	/// Proof: FarmingRewards RewardPerToken (max_values: None, max_size: Some(70), added: 2545, mode: MaxEncodedLen)
	/// Storage: FarmingRewards RewardTally (r:1 w:1)
	/// Proof: FarmingRewards RewardTally (max_values: None, max_size: Some(102), added: 2577, mode: MaxEncodedLen)
	/// Storage: FarmingRewards TotalRewards (r:1 w:1)
	/// Proof: FarmingRewards TotalRewards (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:2 w:2)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:0)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn claim	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1318`
		//  Estimated: `6170`
		// Minimum execution time: 127_536_000 picoseconds.
		Weight::from_parts(130_081_000, 6170)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}