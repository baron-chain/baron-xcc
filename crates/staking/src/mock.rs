use crate as staking;
use crate::{Config, Error};
use frame_support::{
    parameter_types,
    traits::{ConstU32, Everything},
};
use orml_traits::parameter_type_with_key;
pub use primitives::{CurrencyId, CurrencyId::Token, TokenSymbol::*};
use primitives::{VaultCurrencyPair, VaultId};
use sp_arithmetic::FixedI128;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
        Staking: staking::{Pallet, Call, Storage, Event<T>},
        Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
    }
);

pub type AccountId = u64;
pub type SignedFixedPoint = FixedI128;
pub type SignedInner = i128;
pub type Nonce = u64;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = Nonce;
    type Block = Block;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = Token(INTR);
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SignedInner = SignedInner;
    type SignedFixedPoint = SignedFixedPoint;
    type CurrencyId = CurrencyId;
    type GetNativeCurrencyId = GetNativeCurrencyId;
}

pub type Balance = u128;
pub type RawAmount = i128;
parameter_types! {
    pub const MaxLocks: u32 = 50;
}
parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        0
    };
}
impl orml_tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = RawAmount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type CurrencyHooks = ();
    type MaxLocks = MaxLocks;
    type DustRemovalWhitelist = Everything;
    type MaxReserves = ConstU32<0>; // we don't use named reserves
    type ReserveIdentifier = (); // we don't use named reserves
}

pub type TestError = Error<Test>;

pub const VAULT: VaultId<AccountId, CurrencyId> = VaultId {
    account_id: 1,
    currencies: VaultCurrencyPair {
        collateral: Token(DOT),
        wrapped: Token(IBTC),
    },
};
pub const ALICE: VaultId<AccountId, CurrencyId> = VaultId {
    account_id: 2,
    currencies: VaultCurrencyPair {
        collateral: Token(DOT),
        wrapped: Token(IBTC),
    },
};
pub const BOB: VaultId<AccountId, CurrencyId> = VaultId {
    account_id: 3,
    currencies: VaultCurrencyPair {
        collateral: Token(DOT),
        wrapped: Token(IBTC),
    },
};

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

        storage.into()
    }
}

pub fn run_test<T>(test: T)
where
    T: FnOnce(),
{
    ExtBuilder::build().execute_with(|| {
        System::set_block_number(1);
        test();
    });
}
