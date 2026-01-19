# Substrate Framework Development

## Table of Contents
- [Substrate Overview](#substrate-overview)
- [Core Concepts](#core-concepts)
- [Pallet Development](#pallet-development)
- [Runtime Development](#runtime-development)
- [Storage Patterns](#storage-patterns)
- [Events and Errors](#events-and-errors)
- [Transaction Weights](#transaction-weights)
- [Testing](#testing)

## Substrate Overview

Substrate is a modular blockchain framework for building custom blockchains. It's the foundation for Polkadot and many parachains.

### Key Features
- **Modular architecture**: Compose blockchain from pallets (modules)
- **Forkless upgrades**: Update runtime without hard forks
- **Flexible consensus**: BABE, GRANDPA, PoW, PoS, or custom
- **Built-in networking**: libp2p integration
- **WebAssembly runtime**: Platform-independent execution

### Architecture Layers
```
┌─────────────────────────────┐
│   Client (Native Binary)    │  ← Networking, consensus, RPC
├─────────────────────────────┤
│   Runtime (WASM)            │  ← Business logic, state transitions
│   ├── Pallet A              │
│   ├── Pallet B              │
│   └── Pallet C              │
└─────────────────────────────┘
```

## Core Concepts

### Pallets
Pallets are modular components that encapsulate specific blockchain functionality.

```rust
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId>;
        
        #[pallet::constant]
        type MaxNameLength: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SomethingStored { value: u32, who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        StorageOverflow,
        ValueTooLarge,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            Something::<T>::put(something);
            Self::deposit_event(Event::SomethingStored { value: something, who });
            
            Ok(())
        }
    }
}
```

### Runtime Configuration
The runtime is the core blockchain logic compiled to WebAssembly.

```rust
// runtime/src/lib.rs
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64},
    weights::Weight,
};
use sp_runtime::{generic, traits::{BlakeTwo256, IdentifyAccount, Verify}};

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type BlockNumber = u32;
pub type Index = u32;
pub type Hash = sp_core::H256;
pub type Balance = u128;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(
            Weight::from_parts(2_000_000_000_000, u64::MAX)
        );
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = BlockLength;
    type DbWeight = RocksDbWeight;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU32<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        
        // Custom pallets
        MyPallet: pallet_my_pallet,
    }
);
```

## Pallet Development

### Complete Pallet Example
```rust
#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Currency type for handling balances
        type Currency: Currency<Self::AccountId>;
        
        /// Maximum length of a name
        #[pallet::constant]
        type MaxNameLength: Get<u32>;
        
        /// Minimum deposit required
        #[pallet::constant]
        type MinDeposit: Get<BalanceOf<Self>>;
    }

    pub type BalanceOf<T> = 
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn profiles)]
    pub type Profiles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u8, T::MaxNameLength>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn balances_locked)]
    pub type BalancesLocked<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProfileCreated { who: T::AccountId },
        ProfileUpdated { who: T::AccountId },
        ProfileRemoved { who: T::AccountId },
        DepositLocked { who: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProfileAlreadyExists,
        ProfileNotFound,
        NameTooLong,
        InsufficientBalance,
        DepositTooLow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            // Called at beginning of each block
            Weight::from_parts(10_000, 0)
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            // Called at end of each block
        }

        fn offchain_worker(_n: BlockNumberFor<T>) {
            // Off-chain worker logic
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        #[pallet::call_index(0)]
        pub fn create_profile(
            origin: OriginFor<T>,
            name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(!Profiles::<T>::contains_key(&who), Error::<T>::ProfileAlreadyExists);
            
            let bounded_name: BoundedVec<_, _> = name.try_into()
                .map_err(|_| Error::<T>::NameTooLong)?;
            
            let deposit = T::MinDeposit::get();
            T::Currency::reserve(&who, deposit)
                .map_err(|_| Error::<T>::InsufficientBalance)?;
            
            Profiles::<T>::insert(&who, bounded_name);
            BalancesLocked::<T>::insert(&who, deposit);
            
            Self::deposit_event(Event::ProfileCreated { who });
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        #[pallet::call_index(1)]
        pub fn update_profile(
            origin: OriginFor<T>,
            name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);
            
            let bounded_name: BoundedVec<_, _> = name.try_into()
                .map_err(|_| Error::<T>::NameTooLong)?;
            
            Profiles::<T>::insert(&who, bounded_name);
            Self::deposit_event(Event::ProfileUpdated { who });
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(2).ref_time())]
        #[pallet::call_index(2)]
        pub fn remove_profile(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);
            
            let deposit = BalancesLocked::<T>::take(&who);
            T::Currency::unreserve(&who, deposit);
            
            Profiles::<T>::remove(&who);
            Self::deposit_event(Event::ProfileRemoved { who });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // Helper functions (not dispatchable)
        pub fn do_something_internal(who: &T::AccountId) -> Result<(), Error<T>> {
            // Internal logic
            Ok(())
        }
    }
}
```

## Runtime Development

### Genesis Configuration
```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub initial_profiles: Vec<(T::AccountId, Vec<u8>)>,
}

#[cfg(feature = "std")]
impl<T: Config> Default for GenesisConfig<T> {
    fn default() -> Self {
        Self { initial_profiles: Default::default() }
    }
}

#[pallet::genesis_build]
impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
    fn build(&self) {
        for (account, name) in &self.initial_profiles {
            let bounded_name: BoundedVec<_, _> = name.clone().try_into()
                .expect("Name too long in genesis");
            Profiles::<T>::insert(account, bounded_name);
        }
    }
}
```

## Storage Patterns

### Storage Types
```rust
// Single value
#[pallet::storage]
pub type TotalCount<T> = StorageValue<_, u64, ValueQuery>;

// Map: key -> value
#[pallet::storage]
pub type Owners<T: Config> = StorageMap<
    _,
    Blake2_128Concat,  // Hash algorithm
    T::AccountId,      // Key
    u64,               // Value
    ValueQuery,        // Returns default on missing key
>;

// Double map: (key1, key2) -> value
#[pallet::storage]
pub type Approvals<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,      // First key
    Blake2_128Concat,
    T::AccountId,      // Second key
    u64,               // Value
    ValueQuery,
>;

// N-Map: flexible number of keys
#[pallet::storage]
pub type MultiIndex<T: Config> = StorageNMap<
    _,
    (
        NMapKey<Blake2_128Concat, T::AccountId>,
        NMapKey<Blake2_128Concat, u32>,
        NMapKey<Twox64Concat, u32>,
    ),
    Vec<u8>,
    OptionQuery,
>;
```

### Best Practices
```rust
// ✅ Good: Use bounded collections
use frame_support::BoundedVec;

#[pallet::storage]
pub type Data<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u8, T::MaxDataSize>,
>;

// ❌ Bad: Unbounded vector (vulnerable to attacks)
#[pallet::storage]
pub type Data<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Vec<u8>,  // Never use unbounded types in storage!
>;

// ✅ Good: Iterate with limits
let count = Owners::<T>::iter()
    .take(MAX_ITERATIONS)
    .count();

// ❌ Bad: Unbounded iteration
let count = Owners::<T>::iter().count();  // Could run forever!
```

## Events and Errors

### Events Best Practices
```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    // ✅ Include relevant context
    TokenMinted { 
        owner: T::AccountId, 
        token_id: u64, 
        metadata: BoundedVec<u8, T::MaxMetadataLen>,
    },
    
    // ✅ Use past tense
    ProfileUpdated { who: T::AccountId },
    
    // ❌ Avoid: Too generic
    Success,
    
    // ❌ Avoid: Missing context
    Updated,
}
```

### Error Handling
```rust
#[pallet::error]
pub enum Error<T> {
    // ✅ Descriptive names
    InsufficientBalance,
    ProfileAlreadyExists,
    TokenNotFound,
    UnauthorizedAccess,
    
    // ✅ Document when not obvious
    /// The name exceeds the maximum allowed length
    NameTooLong,
    
    // ❌ Avoid: Generic errors
    Error,
    Failed,
}

// Using errors in calls
#[pallet::call]
impl<T: Config> Pallet<T> {
    pub fn transfer(
        origin: OriginFor<T>,
        to: T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let from = ensure_signed(origin)?;
        
        let balance = Balances::<T>::get(&from);
        ensure!(balance >= amount, Error::<T>::InsufficientBalance);
        
        // Continue with transfer...
        Ok(())
    }
}
```

## Transaction Weights

### Weight Calculation
```rust
use frame_support::weights::Weight;

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(
        // Compute: 10,000 base + DB reads/writes
        Weight::from_parts(
            10_000 + T::DbWeight::get().reads_writes(2, 1).ref_time(),
            0
        )
    )]
    pub fn do_something(origin: OriginFor<T>) -> DispatchResult {
        let _who = ensure_signed(origin)?;
        
        // 2 reads
        let value1 = Storage1::<T>::get();
        let value2 = Storage2::<T>::get();
        
        // 1 write
        Storage3::<T>::put(value1 + value2);
        
        Ok(())
    }
}
```

### Benchmarking
```rust
// benchmarking.rs
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    create_profile {
        let caller: T::AccountId = whitelisted_caller();
        let name = vec![1u8; T::MaxNameLength::get() as usize];
    }: _(RawOrigin::Signed(caller.clone()), name)
    verify {
        assert!(Profiles::<T>::contains_key(&caller));
    }

    update_profile {
        let caller: T::AccountId = whitelisted_caller();
        let name = vec![1u8; 10];
        Pallet::<T>::create_profile(RawOrigin::Signed(caller.clone()).into(), name.clone())?;
        
        let new_name = vec![2u8; 20];
    }: _(RawOrigin::Signed(caller.clone()), new_name)
    verify {
        let profile = Profiles::<T>::get(&caller).unwrap();
        assert_eq!(profile[0], 2u8);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;
    use frame_support::{assert_ok, assert_noop};

    #[test]
    fn create_profile_works() {
        new_test_ext().execute_with(|| {
            let account = 1;
            let name = vec![1, 2, 3];
            
            assert_ok!(MyPallet::create_profile(
                RuntimeOrigin::signed(account),
                name.clone()
            ));
            
            assert_eq!(Profiles::<Test>::get(account).unwrap().to_vec(), name);
            
            System::assert_last_event(
                Event::ProfileCreated { who: account }.into()
            );
        });
    }

    #[test]
    fn create_profile_fails_if_exists() {
        new_test_ext().execute_with(|| {
            let account = 1;
            let name = vec![1, 2, 3];
            
            assert_ok!(MyPallet::create_profile(
                RuntimeOrigin::signed(account),
                name.clone()
            ));
            
            assert_noop!(
                MyPallet::create_profile(
                    RuntimeOrigin::signed(account),
                    name
                ),
                Error::<Test>::ProfileAlreadyExists
            );
        });
    }
}
```

### Mock Runtime
```rust
// mock.rs
use frame_support::{parameter_types, traits::ConstU32};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        MyPallet: crate,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
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
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = ();
    type MaxHolds = ();
}

parameter_types! {
    pub const MaxNameLength: u32 = 50;
    pub const MinDeposit: u64 = 100;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxNameLength = MaxNameLength;
    type MinDeposit = MinDeposit;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1000), (2, 2000), (3, 3000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    t.into()
}
```

This covers the essential patterns for professional Substrate development.
