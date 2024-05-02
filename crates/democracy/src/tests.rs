//! The crate's tests.

use super::*;
use crate as pallet_democracy;
use frame_support::{
    assert_noop, assert_ok, ord_parameter_types, parameter_types,
    traits::{ConstU32, ConstU64, Contains, EqualPrivilegeOnly, OnInitialize, SortedMembers},
    weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use pallet_balances::Error as BalancesError;
use sp_core::H256;
use sp_runtime::{
    traits::{BadOrigin, BlakeTwo256, IdentityLookup},
    BuildStorage, Perbill,
};

mod cancellation;
mod decoders;
mod fast_tracking;
mod public_proposals;
mod scheduling;
mod voting;

const MAX_PROPOSALS: u32 = 100;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Preimage: pallet_preimage,
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
    }
);

// Test that a filtered call can be dispatched.
pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
    fn contains(call: &RuntimeCall) -> bool {
        !matches!(
            call,
            &RuntimeCall::Balances(pallet_balances::Call::force_set_balance { .. })
        )
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(
            Weight::from_parts(
                frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
                u64::MAX,
        ));
}

impl frame_system::Config for Test {
    type BaseCallFilter = BaseFilter;
    type BlockWeights = BlockWeights;
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type Block = Block;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_preimage::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<u64>;
    type BaseDeposit = ConstU64<0>;
    type ByteDeposit = ConstU64<0>;
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<u64>;
    type MaxScheduledPerBlock = ConstU32<100>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type WeightInfo = ();
    type Preimages = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type MaxLocks = MaxLocks;
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ();
}
parameter_types! {
    pub const LaunchPeriod: u64 = 60 * 60 * 24 * 7; // one week
    pub const VotingPeriod: u64 = 4;
    pub const FastTrackVotingPeriod: u64 = 2;
    pub const MinimumDeposit: u64 = 1;
    pub const EnactmentPeriod: u64 = 2;
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = MAX_PROPOSALS;
    pub static PreimageByteDeposit: u64 = 0;
    pub const TreasuryAccount:u64 = 232323;
}
ord_parameter_types! {
    pub const One: u64 = 1;
    pub const Two: u64 = 2;
    pub const Three: u64 = 3;
    pub const Four: u64 = 4;
    pub const Five: u64 = 5;
    pub const Six: u64 = 6;
}
pub struct OneToFive;
impl SortedMembers<u64> for OneToFive {
    fn sorted_members() -> Vec<u64> {
        vec![1, 2, 3, 4, 5]
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn add(_m: &u64) {}
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = Scheduler;
    type Preimages = Preimage;
    type Currency = pallet_balances::Pallet<Self>;
    type EnactmentPeriod = EnactmentPeriod;
    type VotingPeriod = VotingPeriod;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    type MinimumDeposit = MinimumDeposit;
    type MaxVotes = MaxVotes;
    type MaxProposals = MaxProposals;
    type MaxDeposits = ConstU32<1000>;
    type FastTrackOrigin = EnsureSignedBy<Five, u64>;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = ();
    type UnixTime = Timestamp;
    type LaunchPeriod = LaunchPeriod;
    type TreasuryAccount = TreasuryAccount;
    type TreasuryCurrency = pallet_balances::Pallet<Self>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_democracy::GenesisConfig::<Test>::default()
        .assimilate_storage(&mut t)
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn params_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Democracy::referendum_count(), 0);
        assert_eq!(Balances::free_balance(42), 0);
        assert_eq!(Balances::total_issuance(), 210);
    });
}

fn set_balance_proposal(value: u64) -> BoundedCallOf<Test> {
    let inner = pallet_balances::Call::force_set_balance {
        who: 42,
        new_free: value,
    };
    let outer = RuntimeCall::Balances(inner);
    Preimage::bound(outer).unwrap()
}

#[test]
fn set_balance_proposal_is_correctly_filtered_out() {
    for i in 0..10 {
        let call = Preimage::realize(&set_balance_proposal(i)).unwrap().0;
        assert!(!<Test as frame_system::Config>::BaseCallFilter::contains(&call));
    }
}

fn propose_set_balance(who: u64, value: u64, delay: u64) -> DispatchResult {
    Democracy::propose(RuntimeOrigin::signed(who), set_balance_proposal(value), delay)
}

fn next_block() {
    let week = 1000 * 60 * 60 * 24 * 7;
    Timestamp::set_timestamp(System::block_number() * week / 2);

    System::set_block_number(System::block_number() + 1);
    Scheduler::on_initialize(System::block_number());
    assert!(Democracy::begin_block(System::block_number()).is_ok());
}

fn fast_forward_to(n: u64) {
    while System::block_number() < n {
        next_block();
    }
}

fn begin_referendum() -> ReferendumIndex {
    System::set_block_number(0);
    assert_ok!(propose_set_balance(1, 2, 1));
    fast_forward_to(2);
    0
}

fn aye(who: u64) -> Vote<u64> {
    Vote {
        aye: true,
        balance: Balances::free_balance(&who),
    }
}

fn nay(who: u64) -> Vote<u64> {
    Vote {
        aye: false,
        balance: Balances::free_balance(&who),
    }
}

fn tally(r: ReferendumIndex) -> Tally<u64> {
    Democracy::referendum_status(r).unwrap().tally
}

#[test]
fn should_launch_works() {
    new_test_ext().execute_with(|| {
        NextLaunchTimestamp::<Test>::put(1670835600); // Mon Dec 12 2022 09:00:00 UTC
        let arbitrary_timestamp = 1670864631; // Mon Dec 12 2022 17:03:51 UTC

        let week_boundaries = [
            1671440400, // Mon Dec 19 2022 09:00:00 UTC
            1672045200, // Mon Dec 26 2022 09:00:00 UTC
            1672650000, // Mon Jan 02 2023 09:00:00 UTC
        ];
        // first launch immediately after launch of chain / first runtime upgrade
        assert!(Democracy::should_launch(arbitrary_timestamp));
        // second time it should return false
        assert!(!Democracy::should_launch(arbitrary_timestamp));

        for boundary in week_boundaries {
            // one second before the next week it should still return false
            assert!(!Democracy::should_launch(boundary - 1));

            // first second of next week it should return true exactly once
            assert!(Democracy::should_launch(boundary));
            assert!(!Democracy::should_launch(boundary));
        }
    });
}

#[test]
fn should_launch_skipped_works() {
    new_test_ext().execute_with(|| {
        NextLaunchTimestamp::<Test>::put(1671440400); // Mon Dec 19 2022 09:00:00 GMT

        // skip 3 weeks + 1 day + 1 hour + 5 minutes
        let now = 1673345100; // Tue Jan 10 2023 10:05:00 GMT

        assert!(Democracy::should_launch(now));
        assert_eq!(
            NextLaunchTimestamp::<Test>::get(),
            1673859600 // Mon Jan 16 2023 09:00:00 GMT
        );
    });
}
