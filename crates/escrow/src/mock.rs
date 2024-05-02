use crate as escrow;
use crate::{Config, Convert, Error};
use frame_support::{
    parameter_types,
    traits::{ConstU32, Everything},
};
pub use primitives::{CurrencyId, CurrencyId::Token, TokenSymbol::*};
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
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Rewards: reward::{Pallet, Call, Storage, Event<T>},
        Escrow: escrow::{Pallet, Call, Storage, Event<T>},
    }
);

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type Nonce = u64;
pub type SignedFixedPoint = FixedI128;

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
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ();
}

impl reward::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SignedFixedPoint = SignedFixedPoint;
    type PoolId = ();
    type StakeId = AccountId;
    type CurrencyId = CurrencyId;
    type MaxRewardCurrencies = ConstU32<1>;
}

parameter_types! {
    pub const Span: BlockNumber = 10;
    pub const MaxPeriod: BlockNumber = 100;
}

pub struct BlockNumberToBalance;

impl Convert<BlockNumber, Balance> for BlockNumberToBalance {
    fn convert(a: BlockNumber) -> Balance {
        a.into()
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type BlockNumberToBalance = BlockNumberToBalance;
    type Currency = Balances;
    type Span = Span;
    type MaxPeriod = MaxPeriod;
    type EscrowRewards = Rewards;
    type WeightInfo = ();
}

pub type TestError = Error<Test>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

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
        System::set_block_number(0);
        test();
    });
}
