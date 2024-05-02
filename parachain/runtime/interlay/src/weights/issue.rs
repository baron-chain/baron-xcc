
//! Autogenerated weights for issue
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-08-07, STEPS: `50`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `interlay-rust-runner-2mz2v-jrrg4`, CPU: `AMD EPYC 7502P 32-Core Processor`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("interlay-dev"), DB CACHE: 1024

// Executed Command:
// target/release/interbtc-parachain
// benchmark
// pallet
// --pallet
// *
// --extrinsic
// *
// --chain
// interlay-dev
// --execution=wasm
// --wasm-execution=compiled
// --steps
// 50
// --repeat
// 10
// --output
// parachain/runtime/interlay/src/weights/
// --template
// .deploy/runtime-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for issue using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> issue::WeightInfo for WeightInfo<T> {

	/// Storage: BTCRelay StartBlockHeight (r:1 w:0)
	/// Proof: BTCRelay StartBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: Oracle Aggregate (r:2 w:0)
	/// Proof: Oracle Aggregate (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: Fee IssueGriefingCollateral (r:1 w:0)
	/// Proof: Fee IssueGriefingCollateral (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Issue IssueBtcDustValue (r:1 w:0)
	/// Proof: Issue IssueBtcDustValue (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: VaultRegistry SecureCollateralThreshold (r:1 w:0)
	/// Proof: VaultRegistry SecureCollateralThreshold (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: VaultStaking Nonce (r:1 w:0)
	/// Proof: VaultStaking Nonce (max_values: None, max_size: Some(74), added: 2549, mode: MaxEncodedLen)
	/// Storage: VaultStaking TotalCurrentStake (r:1 w:0)
	/// Proof: VaultStaking TotalCurrentStake (max_values: None, max_size: Some(106), added: 2581, mode: MaxEncodedLen)
	/// Storage: Fee IssueFee (r:1 w:0)
	/// Proof: Fee IssueFee (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Security Nonce (r:1 w:1)
	/// Proof: Security Nonce (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	/// Storage: System ParentHash (r:1 w:0)
	/// Proof: System ParentHash (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	/// Storage: VaultRegistry VaultBitcoinPublicKey (r:1 w:0)
	/// Proof: VaultRegistry VaultBitcoinPublicKey (max_values: None, max_size: Some(81), added: 2556, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Issue IssuePeriod (r:1 w:0)
	/// Proof: Issue IssuePeriod (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Issue IssueRequests (r:0 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	fn request_issue	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2681`
		//  Estimated: `6028`
		// Minimum execution time: 556_004_000 picoseconds.
		Weight::from_parts(558_989_000, 6028)
			.saturating_add(T::DbWeight::get().reads(18_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: BTCRelay DisableInclusionCheck (r:1 w:0)
	/// Proof: BTCRelay DisableInclusionCheck (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay Chains (r:1 w:0)
	/// Proof: BTCRelay Chains (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BlockHeaders (r:1 w:0)
	/// Proof: BTCRelay BlockHeaders (max_values: None, max_size: Some(200), added: 2675, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableParachainConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableParachainConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `h` is `[2, 10]`.
	/// The range of component `i` is `[1, 10]`.
	/// The range of component `o` is `[1, 10]`.
	/// The range of component `b` is `[770, 2048]`.
	fn execute_issue_exact	(h: u32, i: u32, _o: u32, b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2434`
		//  Estimated: `3737`
		// Minimum execution time: 195_191_000 picoseconds.
		Weight::from_parts(33_956_966, 3737)
			// Standard Error: 810_295
			.saturating_add(Weight::from_parts(10_187_551, 0).saturating_mul(h.into()))
			// Standard Error: 730_159
			.saturating_add(Weight::from_parts(7_188_076, 0).saturating_mul(i.into()))
			// Standard Error: 5_239
			.saturating_add(Weight::from_parts(34_842, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(12_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: BTCRelay DisableInclusionCheck (r:1 w:0)
	/// Proof: BTCRelay DisableInclusionCheck (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay Chains (r:1 w:0)
	/// Proof: BTCRelay Chains (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BlockHeaders (r:1 w:0)
	/// Proof: BTCRelay BlockHeaders (max_values: None, max_size: Some(200), added: 2675, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableParachainConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableParachainConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: VaultRegistry SecureCollateralThreshold (r:1 w:0)
	/// Proof: VaultRegistry SecureCollateralThreshold (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: Oracle Aggregate (r:1 w:0)
	/// Proof: Oracle Aggregate (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: VaultStaking Nonce (r:1 w:0)
	/// Proof: VaultStaking Nonce (max_values: None, max_size: Some(74), added: 2549, mode: MaxEncodedLen)
	/// Storage: VaultStaking TotalCurrentStake (r:1 w:0)
	/// Proof: VaultStaking TotalCurrentStake (max_values: None, max_size: Some(106), added: 2581, mode: MaxEncodedLen)
	/// Storage: Fee IssueFee (r:1 w:0)
	/// Proof: Fee IssueFee (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `h` is `[2, 10]`.
	/// The range of component `i` is `[1, 10]`.
	/// The range of component `o` is `[1, 10]`.
	/// The range of component `b` is `[770, 2048]`.
	fn execute_issue_overpayment	(h: u32, _i: u32, _o: u32, _b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3248`
		//  Estimated: `3737`
		// Minimum execution time: 313_197_000 picoseconds.
		Weight::from_parts(676_783_389, 3737)
			// Standard Error: 967_719
			.saturating_add(Weight::from_parts(1_294_288, 0).saturating_mul(h.into()))
			.saturating_add(T::DbWeight::get().reads(17_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: BTCRelay DisableInclusionCheck (r:1 w:0)
	/// Proof: BTCRelay DisableInclusionCheck (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay Chains (r:1 w:0)
	/// Proof: BTCRelay Chains (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BlockHeaders (r:1 w:0)
	/// Proof: BTCRelay BlockHeaders (max_values: None, max_size: Some(200), added: 2675, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableParachainConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableParachainConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: Fee IssueFee (r:1 w:0)
	/// Proof: Fee IssueFee (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `h` is `[2, 10]`.
	/// The range of component `i` is `[1, 10]`.
	/// The range of component `o` is `[1, 10]`.
	/// The range of component `b` is `[770, 2048]`.
	fn execute_issue_underpayment	(h: u32, _i: u32, o: u32, b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2739`
		//  Estimated: `3737`
		// Minimum execution time: 231_854_000 picoseconds.
		Weight::from_parts(253_010_791, 3737)
			// Standard Error: 189_488
			.saturating_add(Weight::from_parts(3_800_800, 0).saturating_mul(h.into()))
			// Standard Error: 170_748
			.saturating_add(Weight::from_parts(167_716, 0).saturating_mul(o.into()))
			// Standard Error: 1_225
			.saturating_add(Weight::from_parts(6_220, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(13_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: BTCRelay DisableInclusionCheck (r:1 w:0)
	/// Proof: BTCRelay DisableInclusionCheck (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay Chains (r:1 w:0)
	/// Proof: BTCRelay Chains (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BlockHeaders (r:1 w:0)
	/// Proof: BTCRelay BlockHeaders (max_values: None, max_size: Some(200), added: 2675, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableParachainConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableParachainConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `h` is `[2, 10]`.
	/// The range of component `i` is `[1, 10]`.
	/// The range of component `o` is `[1, 10]`.
	/// The range of component `b` is `[770, 2048]`.
	fn execute_expired_issue_exact	(h: u32, i: u32, o: u32, b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3363`
		//  Estimated: `3737`
		// Minimum execution time: 201_924_000 picoseconds.
		Weight::from_parts(139_929_561, 3737)
			// Standard Error: 194_156
			.saturating_add(Weight::from_parts(3_652_125, 0).saturating_mul(h.into()))
			// Standard Error: 174_955
			.saturating_add(Weight::from_parts(1_547_829, 0).saturating_mul(i.into()))
			// Standard Error: 174_955
			.saturating_add(Weight::from_parts(1_815_584, 0).saturating_mul(o.into()))
			// Standard Error: 1_255
			.saturating_add(Weight::from_parts(16_572, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(12_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: BTCRelay DisableInclusionCheck (r:1 w:0)
	/// Proof: BTCRelay DisableInclusionCheck (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay Chains (r:1 w:0)
	/// Proof: BTCRelay Chains (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BlockHeaders (r:1 w:0)
	/// Proof: BTCRelay BlockHeaders (max_values: None, max_size: Some(200), added: 2675, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableParachainConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableParachainConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: VaultRegistry SecureCollateralThreshold (r:1 w:0)
	/// Proof: VaultRegistry SecureCollateralThreshold (max_values: None, max_size: Some(54), added: 2529, mode: MaxEncodedLen)
	/// Storage: Oracle Aggregate (r:1 w:0)
	/// Proof: Oracle Aggregate (max_values: None, max_size: Some(44), added: 2519, mode: MaxEncodedLen)
	/// Storage: VaultStaking Nonce (r:1 w:0)
	/// Proof: VaultStaking Nonce (max_values: None, max_size: Some(74), added: 2549, mode: MaxEncodedLen)
	/// Storage: VaultStaking TotalCurrentStake (r:1 w:0)
	/// Proof: VaultStaking TotalCurrentStake (max_values: None, max_size: Some(106), added: 2581, mode: MaxEncodedLen)
	/// Storage: Fee IssueFee (r:1 w:0)
	/// Proof: Fee IssueFee (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `h` is `[2, 10]`.
	/// The range of component `i` is `[1, 10]`.
	/// The range of component `o` is `[1, 10]`.
	/// The range of component `b` is `[770, 2048]`.
	fn execute_expired_issue_overpayment	(h: u32, i: u32, o: u32, b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3947 + h * (1 ±0) + i * (1 ±0)`
		//  Estimated: `3737`
		// Minimum execution time: 319_510_000 picoseconds.
		Weight::from_parts(297_484_836, 3737)
			// Standard Error: 93_915
			.saturating_add(Weight::from_parts(3_775_028, 0).saturating_mul(h.into()))
			// Standard Error: 84_627
			.saturating_add(Weight::from_parts(1_330_041, 0).saturating_mul(i.into()))
			// Standard Error: 84_627
			.saturating_add(Weight::from_parts(179_165, 0).saturating_mul(o.into()))
			// Standard Error: 607
			.saturating_add(Weight::from_parts(2_628, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(17_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: BTCRelay DisableInclusionCheck (r:1 w:0)
	/// Proof: BTCRelay DisableInclusionCheck (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay Chains (r:1 w:0)
	/// Proof: BTCRelay Chains (max_values: None, max_size: Some(24), added: 2499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BlockHeaders (r:1 w:0)
	/// Proof: BTCRelay BlockHeaders (max_values: None, max_size: Some(200), added: 2675, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableBitcoinConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableBitcoinConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay StableParachainConfirmations (r:1 w:0)
	/// Proof: BTCRelay StableParachainConfirmations (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	/// Storage: Fee IssueFee (r:1 w:0)
	/// Proof: Fee IssueFee (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: Tokens Accounts (r:1 w:1)
	/// Proof: Tokens Accounts (max_values: None, max_size: Some(115), added: 2590, mode: MaxEncodedLen)
	/// Storage: Tokens TotalIssuance (r:1 w:1)
	/// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(35), added: 2510, mode: MaxEncodedLen)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// The range of component `h` is `[2, 10]`.
	/// The range of component `i` is `[1, 10]`.
	/// The range of component `o` is `[1, 10]`.
	/// The range of component `b` is `[770, 2048]`.
	fn execute_expired_issue_underpayment	(h: u32, i: u32, _o: u32, b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3653`
		//  Estimated: `3737`
		// Minimum execution time: 241_423_000 picoseconds.
		Weight::from_parts(228_353_888, 3737)
			// Standard Error: 69_658
			.saturating_add(Weight::from_parts(3_202_457, 0).saturating_mul(h.into()))
			// Standard Error: 62_769
			.saturating_add(Weight::from_parts(689_283, 0).saturating_mul(i.into()))
			// Standard Error: 450
			.saturating_add(Weight::from_parts(3_239, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(13_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: Issue IssueRequests (r:1 w:1)
	/// Proof: Issue IssueRequests (max_values: None, max_size: Some(272), added: 2747, mode: MaxEncodedLen)
	/// Storage: Issue IssuePeriod (r:1 w:0)
	/// Proof: Issue IssuePeriod (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Security ActiveBlockCount (r:1 w:0)
	/// Proof: Security ActiveBlockCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: BTCRelay BestBlockHeight (r:1 w:0)
	/// Proof: BTCRelay BestBlockHeight (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: VaultRegistry Vaults (r:1 w:1)
	/// Proof: VaultRegistry Vaults (max_values: None, max_size: Some(260), added: 2735, mode: MaxEncodedLen)
	fn cancel_issue	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1495`
		//  Estimated: `3737`
		// Minimum execution time: 89_729_000 picoseconds.
		Weight::from_parts(90_551_000, 3737)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: Issue IssuePeriod (r:0 w:1)
	/// Proof: Issue IssuePeriod (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn set_issue_period	() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 17_425_000 picoseconds.
		Weight::from_parts(17_906_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}