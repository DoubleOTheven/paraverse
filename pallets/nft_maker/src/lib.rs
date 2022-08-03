#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_custom_traits::{Ownership, Transfer};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	use crate::types::ItemDetails;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ItemId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		/// Max length for the tokenURI field
		#[pallet::constant]
		type TokenURILimit: Get<u32>;

		// Unique key to count IDs for NFTs
		#[pallet::constant]
		type NFTMakerAccount: Get<frame_support::PalletId>;
	}

	#[pallet::storage]
	pub(super) type Items<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ItemId,
		ItemDetails<T::AccountId, T::TokenURILimit>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type Counter<T: Config> =
		CountedStorageMap<_, Twox128, T::AccountId, T::ItemId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftMinted(T::ItemId, T::AccountId),
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	impl<T: Config> Ownership<T::ItemId, T::AccountId> for Pallet<T> {
		fn is_owner(id: &T::ItemId, who: &T::AccountId) -> bool {
			let item = Items::<T>::get(id);
			if !item.is_some() {
				return false
			}

			item.unwrap().owner == *who
		}
	}

	impl<T: Config> Transfer<T::ItemId, T::AccountId> for Pallet<T> {
		fn transfer(id: &T::ItemId, to: &T::AccountId) -> bool {
			let item = Items::<T>::get(id);
			if item.is_none() {
				return false
			}
			let mut item = item.unwrap();

			item.owner = to.clone();
			Items::<T>::insert(id, &item);

			true
		}
	}

	impl<T: Config> Pallet<T> {
		fn account_id() -> T::AccountId {
			T::NFTMakerAccount::get().into_account_truncating()
		}

		fn increment_ids() -> T::ItemId {
			let key = Self::account_id();
			let result = Counter::<T>::get(&key).unwrap();
			let next = result + 1u32.into();
			Counter::<T>::set(key, next.into());
			next
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(5_000_000)]
		pub fn mint(
			origin: OriginFor<T>,
			token_uri: BoundedVec<u8, T::TokenURILimit>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let next_id = Self::increment_ids();
			let nft = ItemDetails { owner: owner.clone(), token_uri };

			Items::<T>::set(next_id, nft.into());

			Self::deposit_event(Event::<T>::NftMinted(next_id, owner));

			Ok(())
		}
	}
}
