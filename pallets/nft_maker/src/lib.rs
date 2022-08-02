#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod constants;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::fungibles::{Inspect, Mutate, Transfer},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::AtLeast32BitUnsigned;

	use crate::{constants::NFT_ID_COUNT, types::ItemDetails};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Payment: Inspect<Self::AccountId> + Transfer<Self::AccountId> + Mutate<Self::AccountId>;
		type ItemId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		/// Max length for the tokenURI field
		#[pallet::constant]
		type TokenURILimit: Get<u32>;
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
	pub(super) type Counter<T: Config> = CountedStorageMap<_, Twox128, u8, T::ItemId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NftMinted(T::ItemId, T::AccountId),
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	impl<T: Config> Pallet<T> {
		fn increment_ids() -> T::ItemId {
			let result = Counter::<T>::get(NFT_ID_COUNT).unwrap();
			let next = result + 1u32.into();
			Counter::<T>::set(NFT_ID_COUNT, next.into());
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
