// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{parameter_types, traits::Contains, PalletId};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, RuntimeDebug,
};

use crate as dex_swap_router;
use crate::Config;
use dex_general::{GenerateLpAssetId, ValidateAsset};
use dex_stable::traits::{StablePoolLpCurrencyIdGenerate, ValidateCurrency};
use orml_traits::parameter_type_with_key;

type Block = frame_system::mocking::MockBlock<Test>;

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;

    pub const BlockHashCount: u64 = 250;
    pub const DexStablePalletId: PalletId = PalletId(*b"dex/stab");
    pub const DexGeneralPalletId: PalletId = PalletId(*b"dex/genr");
    pub const MaxReserves: u32 = 50;
    pub const MaxLocks:u32 = 50;
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
    pub const PoolCurrencyLimit: u32 = 10;
    pub const PoolCurrencySymbolLimit: u32 = 50;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> u128 {
        0
    };
}

pub type AccountId = u128;
pub type TokenSymbol = u8;
pub type PoolId = u32;

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
    fn contains(_a: &AccountId) -> bool {
        true
    }
}

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
    Forbidden(TokenSymbol),
    Token(TokenSymbol),
    StableLP(PoolId),
    LpToken(TokenSymbol, TokenSymbol),
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

    pub fn is_lp_token(&self) -> bool {
        match self {
            Self::Token(_) => true,
            _ => false,
        }
    }
}

impl From<u32> for CurrencyId {
    fn from(value: u32) -> Self {
        // Inner value must fit inside `u8`
        CurrencyId::Token((value % 256).try_into().unwrap())
    }
}

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
pub enum PoolToken {
    Token(TokenSymbol),
    StablePoolLp(PoolId),
}

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
pub enum PoolType {
    P2(PoolToken, PoolToken),
    P3(PoolToken, PoolToken, PoolToken),
    P4(PoolToken, PoolToken, PoolToken, PoolToken),
    P5(PoolToken, PoolToken, PoolToken, PoolToken, PoolToken),
    P6(PoolToken, PoolToken, PoolToken, PoolToken, PoolToken, PoolToken),
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

impl orml_tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
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

pub type Moment = u64;
pub type Balance = u128;

pub const MILLISECS_PER_BLOCK: Moment = 12000;
pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

impl pallet_timestamp::Config for Test {
    type MinimumPeriod = MinimumPeriod;
    type Moment = u64;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

pub struct PoolLpGenerate;
impl StablePoolLpCurrencyIdGenerate<CurrencyId, PoolId> for PoolLpGenerate {
    fn generate_by_pool_id(pool_id: PoolId) -> CurrencyId {
        return CurrencyId::StableLP(pool_id);
    }
}

pub struct VerifyPoolAsset;
impl ValidateCurrency<CurrencyId> for VerifyPoolAsset {
    fn validate_pooled_currency(currencies: &[CurrencyId]) -> bool {
        for currency in currencies.iter() {
            if let CurrencyId::Forbidden(_) = *currency {
                return false;
            }
        }
        true
    }

    fn validate_pool_lp_currency(currency_id: CurrencyId) -> bool {
        if let CurrencyId::Token(_) = currency_id {
            return false;
        }
        if Tokens::total_issuance(currency_id) > 0 {
            return false;
        }
        true
    }
}

impl dex_stable::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CurrencyId = CurrencyId;
    type MultiCurrency = Tokens;
    type PoolId = PoolId;
    type EnsurePoolAsset = VerifyPoolAsset;
    type LpGenerate = PoolLpGenerate;
    type TimeProvider = Timestamp;
    type PoolCurrencyLimit = PoolCurrencyLimit;
    type PoolCurrencySymbolLimit = PoolCurrencySymbolLimit;
    type PalletId = DexStablePalletId;
    type WeightInfo = ();
}

pub struct PairLpIdentity;
impl GenerateLpAssetId<CurrencyId> for PairLpIdentity {
    fn generate_lp_asset_id(asset_0: CurrencyId, asset_1: CurrencyId) -> Option<CurrencyId> {
        CurrencyId::join_lp_token(asset_0, asset_1)
    }
}

pub struct VerifyPairAsset;
impl ValidateAsset<CurrencyId> for VerifyPairAsset {
    fn validate_asset(currency_id: &CurrencyId) -> bool {
        currency_id.is_lp_token()
    }
}

impl dex_general::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MultiCurrency = Tokens;
    type PalletId = DexGeneralPalletId;
    type AssetId = CurrencyId;
    type EnsurePairAsset = VerifyPairAsset;
    type LpGenerate = PairLpIdentity;
    type WeightInfo = ();
    type MaxBootstrapRewards = MaxBootstrapRewards;
    type MaxBootstrapLimits = MaxBootstrapLimits;
}

parameter_types! {
    pub const MaxBootstrapRewards: u32 = 1000;
    pub const MaxBootstrapLimits:u32 = 1000;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type StablePoolId = PoolId;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type GeneralAmm = DexGeneral;
    type StableAmm = DexStable;
    type GeneralWeightInfo = ();
    type StableWeightInfo = ();
    type WeightInfo = ();
}

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

        storage.into()
    }
}

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>} = 0,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,

        Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 10,
        DexGeneral: dex_general::{Pallet, Call, Storage, Event<T>} = 11,
        DexStable: dex_stable::{Pallet, Call, Storage, Event<T>} = 12,
        DexSwapRouter: dex_swap_router::{Pallet, Call, Event<T>} = 13,
    }
);

pub const USER1: u128 = 1;
pub const USER2: u128 = 2;
pub const USER3: u128 = 3;

pub const TOKEN1_SYMBOL: u8 = 1;
pub const TOKEN2_SYMBOL: u8 = 2;
pub const TOKEN3_SYMBOL: u8 = 3;
pub const TOKEN4_SYMBOL: u8 = 4;

pub const TOKEN1_DECIMAL: u32 = 18;
pub const TOKEN2_DECIMAL: u32 = 18;
pub const TOKEN3_DECIMAL: u32 = 6;
pub const TOKEN4_DECIMAL: u32 = 6;

pub const TOKEN1_UNIT: u128 = 1_000_000_000_000_000_000;
pub const TOKEN2_UNIT: u128 = 1_000_000_000_000_000_000;
pub const TOKEN3_UNIT: u128 = 1_000_000;
pub const TOKEN4_UNIT: u128 = 1_000_000;

pub const TOKEN1_ASSET_ID: CurrencyId = CurrencyId::Token(1);
pub const TOKEN2_ASSET_ID: CurrencyId = CurrencyId::Token(2);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (USER1, CurrencyId::Token(TOKEN1_SYMBOL), TOKEN1_UNIT * 1_00_000_000),
            (USER1, CurrencyId::Token(TOKEN2_SYMBOL), TOKEN2_UNIT * 1_00_000_000),
            (USER1, CurrencyId::Token(TOKEN3_SYMBOL), TOKEN3_UNIT * 1_00_000_000),
            (USER1, CurrencyId::Token(TOKEN4_SYMBOL), TOKEN4_UNIT * 1_00_000_000),
            (USER2, CurrencyId::Token(TOKEN1_SYMBOL), TOKEN1_UNIT * 1_00),
            (USER2, CurrencyId::Token(TOKEN2_SYMBOL), TOKEN2_UNIT * 1_00),
            (USER2, CurrencyId::Token(TOKEN3_SYMBOL), TOKEN3_UNIT * 1_00),
            (USER2, CurrencyId::Token(TOKEN4_SYMBOL), TOKEN4_UNIT * 1_00),
            (USER3, CurrencyId::Token(TOKEN1_SYMBOL), TOKEN1_UNIT * 1_00_000_000),
            (USER3, CurrencyId::Token(TOKEN2_SYMBOL), TOKEN2_UNIT * 1_00_000_000),
            (USER3, CurrencyId::Token(TOKEN3_SYMBOL), TOKEN3_UNIT * 1_00_000_000),
            (USER3, CurrencyId::Token(TOKEN4_SYMBOL), TOKEN4_UNIT * 1_00_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}
