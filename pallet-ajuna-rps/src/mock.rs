use super::*;
use crate as pallet_rps;

use frame_support::{
	parameter_types,
	traits::{OnFinalize, OnInitialize},
};

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
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
		RockPaperScissor: pallet_rps::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
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

impl pallet_rps::Config for Test {
	type Event = Event;
}

/// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	//frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	let t = GenesisConfig { system: Default::default() }.build_storage().unwrap();
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
			//Scheduler::on_finalize(System::block_number());
			RockPaperScissor::on_finalize(System::block_number());
		}

		System::set_block_number(System::block_number() + 1);

		// mock on_initialize
		System::on_initialize(System::block_number());
		//Scheduler::on_initialize(System::block_number());
		RockPaperScissor::on_initialize(System::block_number());
	}
}
