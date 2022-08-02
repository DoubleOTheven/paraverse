#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::fungibles::{Inspect, Mutate, Transfer},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Payment: Inspect<Self::AccountId> + Transfer<Self::AccountId> + Mutate<Self::AccountId>;
		type ItemId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;
		type SaleId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		#[pallet::constant]
		type MarketplaceAccount: Get<frame_support::PalletId>;
	}

	#[pallet::storage]
	pub(super) type Sales<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ItemId,
		u8,
		// Sale<T::AccountId, T::TokenURILimit>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type Counter<T: Config> =
		CountedStorageMap<_, Twox128, T::AccountId, T::ItemId, OptionQuery>;

	impl<T: Config> Pallet<T> {
		/// TODO -> HOW TO CACHE THIS ???
		fn account_id() -> T::AccountId {
			T::MarketplaceAccount::get().into_account_truncating()
		}

		fn increment_ids() -> T::ItemId {
			let key = Self::account_id();
			let result = Counter::<T>::get(&key).unwrap();
			let next = result + 1u32.into();
			Counter::<T>::set(key, next.into());
			next
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SaleCreated(T::ItemId, T::AccountId),
		SaleCanceled(T::ItemId, T::AccountId),
		NftSold(T::ItemId, T::AccountId),
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(5_000_000)]
		pub fn create_sale(origin: OriginFor<T>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let next_id = Self::increment_ids();
			// let nft = ItemDetails { owner: owner.clone(), token_uri };

			// Sales::<T>::set(next_id, nft.into());

			Self::deposit_event(Event::<T>::SaleCreated(next_id, owner));

			Ok(())
		}
	}
}
