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

// use frame_support::{
// 	traits::{Get, StorageVersion, GetStorageVersion},
// 	weights::Weight, storage_alias, Twox64Concat, BoundedVec, log
// };
const LOG_TARGET: &str = "nicks";
use frame_support::{log, traits::OnRuntimeUpgrade};

pub mod v2 {
    use super::*;
	use frame_support::{pallet_prelude::*, weights::Weight};
	use sp_runtime::Saturating;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	#[derive(Decode, Encode)]
	pub struct OldNickname<T: Config> {
		pub first: BoundedVec<u8, T::MaxLength>,
		pub last: Option<BoundedVec<u8, T::MaxLength>>,
	}

	impl<T: Config> OldNickname<T> {
		fn migrate_to_v2(self) -> Nickname<T> {
			Nickname {
				first: self.first,
				last: self.last,
				third: AccountStatus::default(),
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
			
			if onchain_version == 1 && current_version == 2 {
				// migrate to v2
				// Very inefficient, mostly here for illustration purposes.
				let count = NameOf::<T>::iter().count();
				let mut translated = 0u64;
	
				NameOf::<T>::translate::<
					(OldNickname<T>, BalanceOf<T>), _>(
					|_key: T::AccountId, (name, deposit): (OldNickname<T>, BalanceOf<T>)| {
						translated.saturating_inc();
						Some((name.migrate_to_v2(), deposit))
					}
				);

				// option 2
				current_version.put::<Pallet<T>>();

				log::info!(
					target: LOG_TARGET,
					"Upgraded {} names from {} initial names, storage to version {:?}",
					count,
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
			log::info!(
				target: LOG_TARGET,
				"pre_upgrade: current storage version {:?}",
				Pallet::<T>::current_storage_version()
			);
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
			let names = NameOf::<T>::iter_keys().count() as u32;			
			let decodable_names = NameOf::<T>::iter_values().count() as u32;
			log::info!(
				target: LOG_TARGET,
				"pre_upgrade: {:?} names, {:?} decodable names, {:?} total",
				names,
				decodable_names,
				prev_count,
			);
			ensure!(
				names == decodable_names,
				"Not all values are decodable."
			);

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

	}

}