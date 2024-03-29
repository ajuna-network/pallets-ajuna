use super::*;
use crate as pallet_rpsonline;

use sp_core::H256;

use frame_support::{
	parameter_types,
	traits::{EqualPrivilegeOnly, OnFinalize, OnInitialize},
	weights::Weight,
};

use frame_support_test::TestRandomness;

use frame_system::EnsureRoot;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Config, Storage, Event<T>},
		MatchMaker: pallet_matchmaker::{Pallet, Call, Storage, Event<T>},
		RPSOnline: pallet_rpsonline::{Pallet, Call, Config<T>, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1_000_000);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}
impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
}

parameter_types! {
	pub const AmountPlayers: u8 = 2;
	pub const AmountBrackets: u8 = 3;
}

/// Used for matchmaking in pallets/connectfour.
impl pallet_matchmaker::Config for Test {
	type Event = Event;
	type AmountPlayers = AmountPlayers;
	type AmountBrackets = AmountBrackets;
}

impl pallet_rpsonline::Config for Test {
	type Event = Event;
	type Randomness = TestRandomness<Self>;
	type Proposal = Call;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MatchMaker = MatchMaker;
}

/// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	//frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	let t = GenesisConfig {
		system: Default::default(),
		scheduler: Default::default(),
		rps_online: Default::default(),
	}
	.build_storage()
	.unwrap();
	t.into()
}

pub fn run_next_block() {
	run_to_block(System::block_number() + 1);
}

/// Run until a particular block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			// mock on_finalize
			System::on_finalize(System::block_number());
			Scheduler::on_finalize(System::block_number());
			RPSOnline::on_finalize(System::block_number());
		}

		System::set_block_number(System::block_number() + 1);

		// mock on_initialize
		System::on_initialize(System::block_number());
		Scheduler::on_initialize(System::block_number());
		RPSOnline::on_initialize(System::block_number());
	}
}
