// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

//! Test utilities

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use frame_support::{parameter_types, traits::Contains, PalletId};
use orml_traits::parameter_type_with_key;
use sp_core::{ConstU32, H256};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, RuntimeDebug,
};

use crate as pallet_dex_general;
pub use crate::{AssetBalance, Config, GenerateLpAssetId, Pallet, ValidateAsset};

type Block = frame_system::mocking::MockBlock<Test>;

#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    Eq,
    PartialEq,
    Copy,
    Clone,
    RuntimeDebug,
    PartialOrd,
    MaxEncodedLen,
    Ord,
    TypeInfo,
)]
pub enum CurrencyId {
    Token(u8),
    LpToken(u8, u8),
}

impl CurrencyId {
    pub fn join_lp_token(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
        let lp_token_0 = match currency_id_0 {
            CurrencyId::Token(x) => x,
            _ => return None,
        };
        let lp_token_1 = match currency_id_1 {
            CurrencyId::Token(y) => y,
            _ => return None,
        };
        Some(CurrencyId::LpToken(lp_token_0, lp_token_1))
    }
}

pub struct EnsurePairAssetImpl;

impl ValidateAsset<CurrencyId> for EnsurePairAssetImpl {
    fn validate_asset(currency_id: &CurrencyId) -> bool {
        match currency_id {
            CurrencyId::Token(_) => true,
            _ => false,
        }
    }
}

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
        DexGeneral: pallet_dex_general::{Pallet, Call, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;

    pub const BlockHashCount: u64 = 250;
    pub const DexGeneralPalletId: PalletId = PalletId(*b"dex/genr");
    pub const MaxReserves: u32 = 50;
    pub const MaxLocks:u32 = 50;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type Block = Block;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type PalletInfo = PalletInfo;
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> u128 {
        0
    };
}

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
    fn contains(_a: &AccountId) -> bool {
        true
    }
}

impl orml_tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type Amount = i128;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type DustRemovalWhitelist = MockDustRemovalWhitelist;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type CurrencyHooks = ();
}

pub struct PairLpIdentity;
impl GenerateLpAssetId<CurrencyId> for PairLpIdentity {
    fn generate_lp_asset_id(asset_0: CurrencyId, asset_1: CurrencyId) -> Option<CurrencyId> {
        CurrencyId::join_lp_token(asset_0, asset_1)
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MultiCurrency = Tokens;
    type PalletId = DexGeneralPalletId;
    type AssetId = CurrencyId;
    type EnsurePairAsset = EnsurePairAssetImpl;
    type LpGenerate = PairLpIdentity;
    type WeightInfo = ();
    type MaxBootstrapRewards = ConstU32<1000>;
    type MaxBootstrapLimits = ConstU32<1000>;
}

pub type DexPallet = Pallet<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

    pallet_dex_general::GenesisConfig::<Test> {
        fee_receiver: None,
        fee_point: 5,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

type AccountId = u128;
