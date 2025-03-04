// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities

use super::*;
use crate as doas;
use frame_support::{
    ord_parameter_types,
    traits::{ConstU32, ConstU64, Contains}
};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_io;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup}
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Logger module to track execution.
#[frame_support::pallet]
pub mod logger {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(*weight)]
        pub fn privileged_i32_log(origin: OriginFor<T>, i: i32, weight: Weight) -> DispatchResultWithPostInfo {
            // Ensure that the `origin` is `Root`.
            ensure_root(origin)?;
            <I32Log<T>>::append(i);
            Self::deposit_event(Event::AppendI32 { value: i, weight });
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(*weight)]
        pub fn non_privileged_log(origin: OriginFor<T>, i: i32, weight: Weight) -> DispatchResultWithPostInfo {
            // Ensure that the `origin` is some signed account.
            let sender = ensure_signed(origin)?;
            <I32Log<T>>::append(i);
            <AccountLog<T>>::append(sender.clone());
            Self::deposit_event(Event::AppendI32AndAccount {
                sender,
                value: i,
                weight
            });
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AppendI32 {
            value: i32,
            weight: Weight
        },
        AppendI32AndAccount {
            sender: T::AccountId,
            value: i32,
            weight: Weight
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn account_log)]
    pub(super) type AccountLog<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn i32_log)]
    pub(super) type I32Log<T> = StorageValue<_, Vec<i32>, ValueQuery>;
}

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Doas: doas::{Pallet, Call, Event<T>},
        Logger: logger::{Pallet, Call, Storage, Event<T>},
    }
);

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
    fn contains(_: &RuntimeCall) -> bool {
        false
    }
}

impl frame_system::Config for Test {
    type BaseCallFilter = BlockEverything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

// Implement the logger module's `Config` on the Test runtime.
impl logger::Config for Test {
    type RuntimeEvent = RuntimeEvent;
}

ord_parameter_types! {
    pub const One: u64 = 1;
    pub const Two: u64 = 2;
}

// Implement the doas module's `Config` on the Test runtime.
impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Call = RuntimeCall;
    type DoasOrigin = EnsureSignedBy<One, u64>;
}

// New types for dispatchable functions.
pub type DoasCall = doas::Call<Test>;
pub type LoggerCall = logger::Call<Test>;

// Build test environment by setting the root `key` for the Genesis.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
