// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

//! Various pieces of common functionality.
use super::*;
use log::info;
// use frame_support::{
// 	traits::{Get, StorageVersion, GetStorageVersion},
// 	weights::Weight, storage_alias, Twox64Concat, BoundedVec, log
// };
const LOG_TARGET: &str = "nicks";
use frame_support::{log, traits::OnRuntimeUpgrade};

// only contains V1 storage format
pub mod v1 {
    use super::*;
	use frame_support::{pallet_prelude::*, weights::Weight};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

    // #[storage_alias]
	// pub(super) type NameOf<T: Config> =
	// 	StorageMap<Pallet<T>, Twox64Concat, <T as frame_system::Config>::AccountId, (BoundedVec<u8, <T as pallet::Config>::MaxLength>, BalanceOf<T>)>;

	#[derive(Decode)]
	pub struct OldNickname<T: Config> {
		pub first: BoundedVec<u8, T::MaxLength>,
		pub last: Option<BoundedVec<u8, T::MaxLength>>,
	}

	impl<T: Config> OldNickname<T> {
		fn migrate_to_v2(self) -> Nickname<T> {
			let third = AccountStatus::Active;

			Nickname {
				first: self.first,
				last: self.last,
				third,
			}
		}
	}

	pub struct MigrateToV2<T>(sp_std::marker::PhantomData<T>);
	impl <T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
		fn on_runtime_upgrade() -> Weight {
			let onchain_version =  Pallet::<T>::on_chain_storage_version();
			let current_version = Pallet::<T>::current_storage_version();
	
			log::info!(
				target: LOG_TARGET,
				"Running migration with current storage version {:?} / onchain {:?}",
				current_version,
				onchain_version
			);
			
			// Since current version is 1, we only need to migrate from 1 to 2
			// maybe replace current_version == 1 by current_version < 2
			if onchain_version == 1 && current_version == 2 {
				// migrate to v2
				// Very inefficient, mostly here for illustration purposes.
				let count = NameOf::<T>::iter().count();
				let mut translated = 0u64;
	
				NameOf::<T>::translate::<
					(OldNickname<T>, BalanceOf<T>), _>(
					|key: T::AccountId, (name, deposit): (OldNickname<T>, BalanceOf<T>)| {
						translated.saturating_inc();
						Some((name.migrate_to_v2(), deposit))
					}
				);
				// option 1
				//StorageVersion::new(2).put::<Pallet::<T>>();

				// option 2
				current_version.put::<Pallet<T>>();

				log::info!(
					target: LOG_TARGET,
					"Upgraded {} names, storage to version {:?}",
					translated,
					current_version
				);


				T::DbWeight::get().reads_writes(translated + 1, translated + 1)
			} else {
				log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			ensure!(
				onchain_version == 1,
				"must upgrade linearly"
			);
			ensure!(
				current_version == 2,
				"migration from version 1 to 2."
			);
			let prev_count = NameOf::<T>::iter().count();
			Ok((prev_count as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), &'static str> {
			let prev_count: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);
			let post_count = NameOf::<T>::iter().count() as u32;
			assert_eq!(
				prev_count, post_count,
				"the records count before and after the migration should be the same"
			);

			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();

			ensure!(current_version == 2, "must_upgrade to v2");
			assert_eq!(
				current_version, onchain_version,
				"after migration, the current_version and onchain_version should be the same"
			);
			NameOf::<T>::iter().for_each(|(key, (name, deposit))| {
				assert!(name.third == AccountStatus::Active,
				"accounts should only be Active. None should be in Inactive status, or undefined state")
			});
			Ok(())
		}

	//on_runtime_upgrade()
	}

//don't code
}