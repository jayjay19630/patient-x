//! Mock runtime for identity-registry pallet tests

use crate as pallet_identity_registry;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU32, ConstU64},
};
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
        System: frame_system,
        Timestamp: pallet_timestamp,
        IdentityRegistry: pallet_identity_registry,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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

impl pallet_identity_registry::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type TimeProvider = Timestamp;
    type MaxIdentitiesPerAccount = ConstU32<1>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_timestamp::GenesisConfig::<Test> { now: 0 }
        .assimilate_storage(&mut t)
        .unwrap();

    t.into()
}

// Helper function to advance time
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Timestamp::set_timestamp(System::block_number() * 12000);
    }
}

// Helper function to create a DID
pub fn create_did(suffix: &str) -> sp_runtime::BoundedVec<u8, ConstU32<100>> {
    let did = format!("did:patientx:{}", suffix);
    sp_runtime::BoundedVec::try_from(did.as_bytes().to_vec()).unwrap()
}

// Helper function to create an email hash
pub fn create_email_hash(email: &str) -> H256 {
    use sp_runtime::traits::Hash;
    BlakeTwo256::hash(email.as_bytes())
}

// Helper function to create a name
pub fn create_name(name: &str) -> sp_runtime::BoundedVec<u8, ConstU32<64>> {
    sp_runtime::BoundedVec::try_from(name.as_bytes().to_vec()).unwrap()
}
