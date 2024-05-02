
//! Autogenerated weights for escrow
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

/// Weights for escrow using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> escrow::WeightInfo for WeightInfo<T> {

	/// Storage: Escrow Locked (r:1 w:1)
	/// Proof: Escrow Locked (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Escrow Blocks (r:1 w:0)
	/// Proof: Escrow Blocks (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: Escrow Limits (r:1 w:0)
	/// Proof: Escrow Limits (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	/// Storage: Tokens Locks (r:1 w:1)
	/// Proof: Tokens Locks (max_values: None, max_size: Some(1268), added: 3743, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Escrow SlopeChanges (r:54 w:1)
	/// Proof: Escrow SlopeChanges (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
	/// Storage: Escrow Epoch (r:1 w:1)
	/// Proof: Escrow Epoch (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Escrow PointHistory (r:1 w:52)
	/// Proof: Escrow PointHistory (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointEpoch (r:1 w:1)
	/// Proof: Escrow UserPointEpoch (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: EscrowRewards Stake (r:1 w:1)
	/// Proof: EscrowRewards Stake (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: EscrowRewards TotalStake (r:1 w:1)
	/// Proof: EscrowRewards TotalStake (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardCurrencies (r:1 w:0)
	/// Proof: EscrowRewards RewardCurrencies (max_values: None, max_size: Some(28), added: 2503, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardTally (r:1 w:1)
	/// Proof: EscrowRewards RewardTally (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardPerToken (r:1 w:0)
	/// Proof: EscrowRewards RewardPerToken (max_values: None, max_size: Some(59), added: 2534, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointHistory (r:0 w:1)
	/// Proof: Escrow UserPointHistory (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn create_lock	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1094`
		//  Estimated: `136584`
		// Minimum execution time: 463_989_000 picoseconds.
		Weight::from_parts(466_794_000, 136584)
			.saturating_add(T::DbWeight::get().reads(68_u64))
			.saturating_add(T::DbWeight::get().writes(63_u64))
	}
	/// Storage: Escrow Locked (r:1 w:1)
	/// Proof: Escrow Locked (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Escrow Blocks (r:1 w:0)
	/// Proof: Escrow Blocks (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: Escrow Limits (r:1 w:0)
	/// Proof: Escrow Limits (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	/// Storage: Tokens Locks (r:1 w:1)
	/// Proof: Tokens Locks (max_values: None, max_size: Some(1268), added: 3743, mode: MaxEncodedLen)
	/// Storage: Escrow SlopeChanges (r:1 w:1)
	/// Proof: Escrow SlopeChanges (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
	/// Storage: Escrow Epoch (r:1 w:1)
	/// Proof: Escrow Epoch (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Escrow PointHistory (r:1 w:1)
	/// Proof: Escrow PointHistory (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointEpoch (r:1 w:1)
	/// Proof: Escrow UserPointEpoch (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: EscrowRewards Stake (r:1 w:1)
	/// Proof: EscrowRewards Stake (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: EscrowRewards TotalStake (r:1 w:1)
	/// Proof: EscrowRewards TotalStake (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardCurrencies (r:1 w:0)
	/// Proof: EscrowRewards RewardCurrencies (max_values: None, max_size: Some(28), added: 2503, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardTally (r:1 w:1)
	/// Proof: EscrowRewards RewardTally (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardPerToken (r:1 w:0)
	/// Proof: EscrowRewards RewardPerToken (max_values: None, max_size: Some(59), added: 2534, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointHistory (r:0 w:1)
	/// Proof: Escrow UserPointHistory (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn increase_amount	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1683`
		//  Estimated: `4733`
		// Minimum execution time: 207_245_000 picoseconds.
		Weight::from_parts(208_998_000, 4733)
			.saturating_add(T::DbWeight::get().reads(14_u64))
			.saturating_add(T::DbWeight::get().writes(11_u64))
	}
	/// Storage: Escrow Locked (r:1 w:1)
	/// Proof: Escrow Locked (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Escrow Blocks (r:1 w:0)
	/// Proof: Escrow Blocks (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: Escrow Limits (r:1 w:0)
	/// Proof: Escrow Limits (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	/// Storage: Tokens Locks (r:1 w:1)
	/// Proof: Tokens Locks (max_values: None, max_size: Some(1268), added: 3743, mode: MaxEncodedLen)
	/// Storage: Escrow SlopeChanges (r:97 w:2)
	/// Proof: Escrow SlopeChanges (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
	/// Storage: Escrow Epoch (r:1 w:1)
	/// Proof: Escrow Epoch (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Escrow PointHistory (r:1 w:95)
	/// Proof: Escrow PointHistory (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointEpoch (r:1 w:1)
	/// Proof: Escrow UserPointEpoch (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: EscrowRewards Stake (r:1 w:1)
	/// Proof: EscrowRewards Stake (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: EscrowRewards TotalStake (r:1 w:1)
	/// Proof: EscrowRewards TotalStake (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardCurrencies (r:1 w:0)
	/// Proof: EscrowRewards RewardCurrencies (max_values: None, max_size: Some(28), added: 2503, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardTally (r:1 w:1)
	/// Proof: EscrowRewards RewardTally (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardPerToken (r:1 w:0)
	/// Proof: EscrowRewards RewardPerToken (max_values: None, max_size: Some(59), added: 2534, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointHistory (r:0 w:1)
	/// Proof: Escrow UserPointHistory (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn increase_unlock_height	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1683`
		//  Estimated: `244557`
		// Minimum execution time: 699_982_000 picoseconds.
		Weight::from_parts(706_124_000, 244557)
			.saturating_add(T::DbWeight::get().reads(110_u64))
			.saturating_add(T::DbWeight::get().writes(106_u64))
	}
	/// Storage: Escrow Locked (r:1 w:1)
	/// Proof: Escrow Locked (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: EscrowRewards Stake (r:1 w:1)
	/// Proof: EscrowRewards Stake (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: EscrowRewards TotalStake (r:1 w:1)
	/// Proof: EscrowRewards TotalStake (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardCurrencies (r:1 w:0)
	/// Proof: EscrowRewards RewardCurrencies (max_values: None, max_size: Some(28), added: 2503, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardTally (r:1 w:1)
	/// Proof: EscrowRewards RewardTally (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardPerToken (r:1 w:0)
	/// Proof: EscrowRewards RewardPerToken (max_values: None, max_size: Some(59), added: 2534, mode: MaxEncodedLen)
	/// Storage: Escrow SlopeChanges (r:97 w:0)
	/// Proof: Escrow SlopeChanges (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
	/// Storage: Escrow Epoch (r:1 w:1)
	/// Proof: Escrow Epoch (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Escrow PointHistory (r:1 w:97)
	/// Proof: Escrow PointHistory (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointEpoch (r:1 w:1)
	/// Proof: Escrow UserPointEpoch (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: Tokens Locks (r:1 w:1)
	/// Proof: Tokens Locks (max_values: None, max_size: Some(1268), added: 3743, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointHistory (r:1 w:2)
	/// Proof: Escrow UserPointHistory (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn withdraw	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1902`
		//  Estimated: `244557`
		// Minimum execution time: 681_896_000 picoseconds.
		Weight::from_parts(684_381_000, 244557)
			.saturating_add(T::DbWeight::get().reads(110_u64))
			.saturating_add(T::DbWeight::get().writes(108_u64))
	}
	/// Storage: Escrow Limits (r:0 w:1)
	/// Proof: Escrow Limits (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	fn set_account_limit	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_631_000 picoseconds.
		Weight::from_parts(10_732_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Escrow Blocks (r:0 w:1)
	/// Proof: Escrow Blocks (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	fn set_account_block	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_291_000 picoseconds.
		Weight::from_parts(10_481_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Escrow Locked (r:1 w:1)
	/// Proof: Escrow Locked (max_values: None, max_size: Some(68), added: 2543, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Escrow Blocks (r:1 w:0)
	/// Proof: Escrow Blocks (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
	/// Storage: Escrow Limits (r:1 w:0)
	/// Proof: Escrow Limits (max_values: None, max_size: Some(56), added: 2531, mode: MaxEncodedLen)
	/// Storage: Tokens Locks (r:1 w:1)
	/// Proof: Tokens Locks (max_values: None, max_size: Some(1268), added: 3743, mode: MaxEncodedLen)
	/// Storage: Escrow SlopeChanges (r:96 w:1)
	/// Proof: Escrow SlopeChanges (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
	/// Storage: Escrow Epoch (r:1 w:1)
	/// Proof: Escrow Epoch (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Escrow PointHistory (r:1 w:95)
	/// Proof: Escrow PointHistory (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointEpoch (r:1 w:1)
	/// Proof: Escrow UserPointEpoch (max_values: None, max_size: Some(52), added: 2527, mode: MaxEncodedLen)
	/// Storage: EscrowRewards Stake (r:1 w:1)
	/// Proof: EscrowRewards Stake (max_values: None, max_size: Some(64), added: 2539, mode: MaxEncodedLen)
	/// Storage: EscrowRewards TotalStake (r:1 w:1)
	/// Proof: EscrowRewards TotalStake (max_values: None, max_size: Some(32), added: 2507, mode: MaxEncodedLen)
	/// Storage: EscrowRewards RewardCurrencies (r:1 w:0)
	/// Proof: EscrowRewards RewardCurrencies (max_values: None, max_size: Some(28), added: 2503, mode: MaxEncodedLen)
	/// Storage: Escrow UserPointHistory (r:0 w:1)
	/// Proof: Escrow UserPointHistory (max_values: None, max_size: Some(88), added: 2563, mode: MaxEncodedLen)
	fn update_user_stake	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1490`
		//  Estimated: `242046`
		// Minimum execution time: 661_555_000 picoseconds.
		Weight::from_parts(667_447_000, 242046)
			.saturating_add(T::DbWeight::get().reads(107_u64))
			.saturating_add(T::DbWeight::get().writes(104_u64))
	}
}