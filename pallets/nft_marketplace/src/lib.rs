#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::fungibles::{Inspect, InspectMetadata, Mutate, Transfer},
	};
	use frame_system::pallet_prelude::*;
	use pallet_custom_traits::{Ownership, Transfer as ItemTransfer};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	use crate::types::SaleItem;

	type AssetIdOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::AssetId;
	type BalanceOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Assets: Inspect<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ Mutate<Self::AccountId>
			+ InspectMetadata<Self::AccountId>;
		type ItemId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;
		type SaleId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;
		type NFT: Ownership<Self::ItemId, Self::AccountId>
			+ ItemTransfer<Self::ItemId, Self::AccountId>;

		#[pallet::constant]
		type MarketplaceAccount: Get<frame_support::PalletId>;
	}

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Sales<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::SaleId,
		SaleItem<T::AccountId, T::SaleId, AssetIdOf<T>, T::ItemId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type Counter<T: Config> =
		CountedStorageMap<_, Twox128, T::AccountId, T::SaleId, OptionQuery>;

	impl<T: Config> Pallet<T> {
		/// TODO -> HOW TO CACHE THIS ???
		fn account_id() -> T::AccountId {
			T::MarketplaceAccount::get().into_account_truncating()
		}

		fn increment_ids() -> T::SaleId {
			let key = Self::account_id();
			let result = Counter::<T>::get(&key).unwrap();
			let next = result + 1u32.into();
			Counter::<T>::set(key, next.into());
			next
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		NotFound,
		Unauthorized,
		InsufficientBalance,
		AssetDoesNotExist,
		InvalidPrice,
		SaleNotFound,
		ItemTTransferFailed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SaleCreated(T::SaleId, T::AccountId),
		SaleCanceled(T::SaleId, T::AccountId),
		ItemPurchased(T::ItemId, T::AccountId, BalanceOf<T>),
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(5_000_000)]
		pub fn create_sale(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			item_id: T::ItemId,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(price > 0u32.into(), Error::<T>::InvalidPrice);
			ensure!(T::NFT::is_owner(&item_id, &sender), Error::<T>::Unauthorized);
			let asset = T::Assets::name(&asset_id);
			ensure!(asset.len() > 0, Error::<T>::AssetDoesNotExist);

			let next_id = Self::increment_ids();

			let sale: SaleItem<T::AccountId, T::SaleId, AssetIdOf<T>, T::ItemId, BalanceOf<T>> =
				SaleItem { owner: sender.clone(), id: next_id, asset_id, item_id, price };
			Sales::<T>::insert(next_id, sale);

			Self::deposit_event(Event::<T>::SaleCreated(next_id, sender));

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn purchase(origin: OriginFor<T>, sale_id: T::SaleId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check sale exists
			let sale = Sales::<T>::get(sale_id);
			ensure!(sale.is_some(), Error::<T>::SaleNotFound);
			let sale = sale.unwrap();

			// Check buyer's balance
			let buyer_balance = T::Assets::balance(sale.asset_id, &sender);
			ensure!(&sale.price <= &buyer_balance, Error::<T>::InsufficientBalance);

			// Pay the seller
			T::Assets::transfer(sale.asset_id, &sender, &sale.owner, sale.price, false)?;

			// Transfer Item
			let success = T::NFT::transfer(&sale.item_id, &sender);
			ensure!(success, Error::<T>::ItemTTransferFailed);

			// Emit event
			Self::deposit_event(Event::<T>::ItemPurchased(sale.item_id, sender, sale.price));

			// Delete the sale
			Sales::<T>::remove(sale_id);

			Ok(())
		}
	}
}
