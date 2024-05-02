use crate as fee;
use crate::{Config, Error};
use frame_support::{
    parameter_types,
    traits::{ConstU32, Everything},
    PalletId,
};
use mocktopus::mocking::clear_mocks;
use orml_traits::parameter_type_with_key;
use primitives::VaultId;
pub use primitives::{CurrencyId, CurrencyId::Token, TokenSymbol::*};
use sp_arithmetic::{FixedI128, FixedU128};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup, Zero},
    BuildStorage, DispatchError, FixedPointNumber,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},

        // Tokens & Balances
        Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>},

        CapacityRewards: reward::<Instance1>::{Pallet, Call, Storage, Event<T>},
        VaultRewards: reward::<Instance2>::{Pallet, Call, Storage, Event<T>},
        VaultStaking: staking::{Pallet, Storage, Event<T>},

        // Operational
        Security: security::{Pallet, Call, Storage, Event<T>},
        Fee: fee::{Pallet, Call, Config<T>, Storage},
    }
);

pub type AccountId = u64;
pub type Balance = u128;
pub type RawAmount = i128;
pub type Moment = u64;
pub type Nonce = u64;
pub type SignedFixedPoint = FixedI128;
pub type SignedInner = i128;
pub type UnsignedFixedPoint = FixedU128;

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
    pub const GetRelayChainCurrencyId: CurrencyId = Token(DOT);
    pub const GetWrappedCurrencyId: CurrencyId = Token(IBTC);
    pub const MaxLocks: u32 = 50;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Zero::zero()
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

type CapacityRewardsInstance = reward::Instance1;

impl reward::Config<CapacityRewardsInstance> for Test {
    type RuntimeEvent = RuntimeEvent;
    type SignedFixedPoint = SignedFixedPoint;
    type PoolId = ();
    type StakeId = CurrencyId;
    type CurrencyId = CurrencyId;
    type MaxRewardCurrencies = ConstU32<10>;
}

type VaultRewardsInstance = reward::Instance2;

impl reward::Config<VaultRewardsInstance> for Test {
    type RuntimeEvent = RuntimeEvent;
    type SignedFixedPoint = SignedFixedPoint;
    type PoolId = CurrencyId;
    type StakeId = VaultId<AccountId, CurrencyId>;
    type CurrencyId = CurrencyId;
    type MaxRewardCurrencies = ConstU32<10>;
}

impl staking::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SignedFixedPoint = SignedFixedPoint;
    type SignedInner = SignedInner;
    type CurrencyId = CurrencyId;
    type GetNativeCurrencyId = GetNativeCurrencyId;
}

parameter_types! {
    pub const MinimumPeriod: Moment = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl security::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub struct CurrencyConvert;
impl currency::CurrencyConversion<currency::Amount<Test>, CurrencyId> for CurrencyConvert {
    fn convert(
        _amount: &currency::Amount<Test>,
        _to: CurrencyId,
    ) -> Result<currency::Amount<Test>, sp_runtime::DispatchError> {
        unimplemented!()
    }
}

impl currency::Config for Test {
    type SignedInner = SignedInner;
    type SignedFixedPoint = SignedFixedPoint;
    type UnsignedFixedPoint = UnsignedFixedPoint;
    type Balance = Balance;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type GetRelayChainCurrencyId = GetRelayChainCurrencyId;
    type GetWrappedCurrencyId = GetWrappedCurrencyId;
    type CurrencyConversion = CurrencyConvert;
}

parameter_types! {
    pub const FeePalletId: PalletId = PalletId(*b"mod/fees");
    pub const MaxExpectedValue: UnsignedFixedPoint = UnsignedFixedPoint::from_inner(<UnsignedFixedPoint as FixedPointNumber>::DIV);
}

pub struct MockNomination;

impl traits::NominationApi<VaultId<AccountId, CurrencyId>, currency::Amount<Test>> for MockNomination {
    fn deposit_vault_collateral(
        _vault_id: &VaultId<AccountId, CurrencyId>,
        _amount: &currency::Amount<Test>,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
    fn ensure_opted_in_to_nomination(_vault_id: &VaultId<AccountId, CurrencyId>) -> Result<(), DispatchError> {
        Ok(())
    }

    #[cfg(any(feature = "runtime-benchmarks", test))]
    fn opt_in_to_nomination(_vault_id: &VaultId<AccountId, CurrencyId>) {}
}

impl Config for Test {
    type FeePalletId = FeePalletId;
    type WeightInfo = ();
    type SignedFixedPoint = SignedFixedPoint;
    type SignedInner = SignedInner;
    type CapacityRewards = CapacityRewards;
    type VaultRewards = VaultRewards;
    type VaultStaking = VaultStaking;
    type OnSweep = ();
    type MaxExpectedValue = MaxExpectedValue;
    type NominationApi = MockNomination;
}

#[allow(dead_code)]
pub type TestError = Error<Test>;

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
    clear_mocks();
    ExtBuilder::build().execute_with(|| {
        System::set_block_number(1);
        Security::set_active_block_number(1);
        test();
    });
}
