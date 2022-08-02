#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::fungibles::{Inspect, Mutate, Transfer},
	};
	use frame_system::pallet_prelude::*;

	use crate::types::ItemDetails;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Payment: Inspect<Self::AccountId> + Transfer<Self::AccountId> + Mutate<Self::AccountId>;
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		/// Max length for the tokenURI field
		#[pallet::constant]
		type TokenURILimit: Get<u32>;
	}

	#[pallet::storage]
	pub(super) type Item<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ItemId,
		ItemDetails<T::AccountId, T::TokenURILimit>,
		OptionQuery,
	>;

	#[pallet::event]
	// #[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InsufficientBalance,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		pub fn mint(origin: OriginFor<T>) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Ok(())
		}
	}
}
