
//! Autogenerated weights for pallet_utility
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

/// Weights for pallet_utility using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {

	/// The range of component `c` is `[0, 1000]`.
	fn batch	(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 14_058_000 picoseconds.
		Weight::from_parts(14_189_000, 0)
			// Standard Error: 4_520
			.saturating_add(Weight::from_parts(10_937_825, 0).saturating_mul(c.into()))
	}
	fn as_derivative	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 11_062_000 picoseconds.
		Weight::from_parts(11_383_000, 0)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all	(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 14_068_000 picoseconds.
		Weight::from_parts(20_266_041, 0)
			// Standard Error: 6_485
			.saturating_add(Weight::from_parts(11_505_085, 0).saturating_mul(c.into()))
	}
	fn dispatch_as	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 19_228_000 picoseconds.
		Weight::from_parts(20_481_000, 0)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch	(c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 14_228_000 picoseconds.
		Weight::from_parts(6_535_190, 0)
			// Standard Error: 79_731
			.saturating_add(Weight::from_parts(11_047_964, 0).saturating_mul(c.into()))
	}
}