#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::fungibles::{Inspect, Mutate, Transfer},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Assets: Inspect<Self::AccountId> + Transfer<Self::AccountId> + Mutate<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}
	#[pallet::error]
	pub enum Error<T> {
		InsufficientBalance,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		pub fn mint(origin: OriginFor<T>) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Ok(())
		}
	}
}
