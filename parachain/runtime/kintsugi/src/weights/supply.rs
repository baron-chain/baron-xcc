
//! Autogenerated weights for supply
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

/// Weights for supply using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> supply::WeightInfo for WeightInfo<T> {

	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: Supply Inflation (r:1 w:0)
	/// Proof: Supply Inflation (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:4 w:4)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: System Account (r:2 w:2)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: Supply LastEmission (r:0 w:1)
	/// Proof: Supply LastEmission (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	fn on_initialize	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `606`
		//  Estimated: `11350`
		// Minimum execution time: 199_900_000 picoseconds.
		Weight::from_parts(201_875_000, 11350)
			.saturating_add(T::DbWeight::get().reads(8_u64))
			.saturating_add(T::DbWeight::get().writes(8_u64))
	}
	/// Storage: Supply Inflation (r:0 w:1)
	/// Proof: Supply Inflation (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	fn set_start_height_and_inflation	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_619_000 picoseconds.
		Weight::from_parts(9_890_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}