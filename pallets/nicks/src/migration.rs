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
	traits::{Get, StorageVersion, GetStorageVersion},
	weights::Weight, storage_alias, Twox64Concat, log
};
const LOG_TARGET: &str = "nicks";


// only contains V1 storage format
pub mod v1 {
    use super::*;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

    #[storage_alias]
	pub(super) type NameOf<T: Config> =
		StorageMap<Pallet<T>, Twox64Concat, <T as frame_system::Config>::AccountId, (Nickname<T>, BalanceOf<T>)>;
} 


// contains checks and transforms storage to V2 format
pub fn migrate_to_v2<T: Config>() -> Weight {
    let onchain_version =  Pallet::<T>::on_chain_storage_version();
    if onchain_version < 3 {
            // migrate to v2
            // Very inefficient, mostly here for illustration purposes.
			let count = v1::NameOf::<T>::iter().count();
			info!(target: LOG_TARGET, " >>> Updating MyNicks storage. Migrating {} nicknames...", count);

            NameOf::<T>::translate::<(Nickname<T>, BalanceOf<T>), _>(
                |k: T::AccountId, (nick, deposit): (Nickname<T>, BalanceOf<T>)| {
                    info!(target: LOG_TARGET, " >>> Migrating nickname {:?} {:?} for account  {:?} ", nick.first, nick.last, k);

                    let mut new_nick = nick.clone();

                    new_nick.first = nick.first.into();
                    new_nick.last = nick.last.into();
                    new_nick.third = AccountStatus::default();

                Some((new_nick, deposit))
            });

			// Update storage version.
			StorageVersion::new(3).put::<Pallet::<T>>();
			// Very inefficient, mostly here for illustration purposes.
			let count = NameOf::<T>::iter().count();
			info!(target: LOG_TARGET," <<< MyNicks storage updated! Migrated {} nicknames ✅", count);
			// Return the weight consumed by the migration.
			T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)

    } else {
        info!(target: LOG_TARGET, " >>> Unused migration!");
        // We don't do anything here.
		Weight::zero()
    }

} 