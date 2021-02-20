use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;
use pallet_balances::{self as balances, Reasons};

parameter_types! {
    pub const ExistentialDeposit: u64 = 1; // Should be greather than zero
}

impl pallet_balances::Trait for Test {
    type Balance = u64;
//    type Event = MetaEvent;
	type Event = ();
	type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
//    type AccountStore = ();
    type WeightInfo = ();
    type MaxLocks = ();
}

mod faucet {
    // Re-export contents of the root. This basically
    // needs to give a name for the current crate.
    // This hack is required for `impl_outer_event!`.
    pub use super::super::*;
}
impl_outer_event! {
    pub enum MetaEvent for Test {
        system<T>, balances<T>, faucet<T>,
    }
}

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl Trait for Test {
	type Event = ();
	type Currency = Balances;
}

type Balances = pallet_balances::Module<Test>;

pub type TemplateModule = Module<Test>;

pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
