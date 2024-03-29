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
use frame_support::{
	traits::{Get, StorageVersion, GetStorageVersion, OnRuntimeUpgrade},
	weights::Weight, storage_alias, Twox64Concat, BoundedVec, log
};
const LOG_TARGET: &str = "nicks";


// only contains V1 storage format
pub mod v1 {
    use super::*;
	// use crate::Config;
	use frame_support::{pallet_prelude::*, weights::Weight};
	use sp_runtime::Saturating;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

    // #[storage_alias]
	// pub(super) type NameOf<T: Config> =
	// 	StorageMap<Pallet<T>, Twox64Concat, <T as frame_system::Config>::AccountId, (BoundedVec<u8, <T as pallet::Config>::MaxLength>, BalanceOf<T>)>;
	
	#[derive(Encode, Decode)]
	pub struct NickName<T: Config> {
		pub first: BoundedVec<u8, T::MaxLength>,
		pub last: Option<BoundedVec<u8, T::MaxLength>>,
	}

	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl <T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let onchain_version =  Pallet::<T>::on_chain_storage_version();
			let current_version = Pallet::<T>::current_storage_version();

			let count = NameOf::<T>::iter().count();
			let mut translated = 0u64;
	
			log::info!(
				target: LOG_TARGET,
				"Running migration with current storage version {:?} / onchain {:?}",
				current_version,
				onchain_version
			);

			if onchain_version == 0 && current_version == 1 {

				// We transform the storage values from the old into the new format.
				// NameOf::<T>::translate::<(Vec<u8>, BalanceOf<T>), _>(
				NameOf::<T>::translate(
					|key: T::AccountId, (nick, deposit): (Vec<u8>, BalanceOf<T>)| {
						info!(target: LOG_TARGET, "     Migrated nickname for {:?}...", key);
						translated.saturating_inc();

						// We split the nick at ' ' (<space>).
						match nick.iter().rposition(|&x| x == b" "[0]) {
							Some(ndx) => {
								let bounded_first: BoundedVec<_, _> = nick[0..ndx].to_vec().try_into().unwrap();
								let bounded_last: BoundedVec<_, _> = nick[ndx + 1..].to_vec().try_into().unwrap();
								Some((Nickname {
									first: bounded_first,
									last: Some(bounded_last)
								}, deposit))
						},
							None => {
								let bounded_name: BoundedVec<_, _> = nick.to_vec().try_into().unwrap();
								Some((Nickname { first: bounded_name, last: None }, deposit))
							}
						}
					}
				);


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
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			ensure!(
				onchain_version == 0,
				"must upgrade linearly"
			);
			ensure!(
				current_version == 1,
				"migration from version 0 to 1."
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
	
			ensure!(current_version == 1, "must_upgrade to v1");
			assert_eq!(
				current_version, onchain_version,
				"after migration, the current_version and onchain_version should be the same"
			);
			NameOf::<T>::iter().for_each(|(key, (nicks, deposit))| {
				assert!(nicks.last.is_none(),
				"accounts should only be Active. None should be in Inactive status, or undefined state")
			});
			Ok(())
		}
		
	}
}


