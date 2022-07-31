#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::fungibles::{Inspect, Transfer},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	type AssetIdOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::AssetId;
	type BalanceOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Assets: Inspect<Self::AccountId> + Transfer<Self::AccountId>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	pub enum Event<T: Config> {
		LiquidityProvided(T::AccountId, u64, u128, u128),
		LPTokensMinted(T::AccountId, u64, u128),
		LPTokensBurnt(T::AccountId, u64, u128),
		TokensSwapped(T::AccountId, u64, u128, u128),
	}
	#[pallet::error]
	pub enum Error<T> {
		AddLiquidityFailed,
		InsufficientBalance,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	// #[pallet::unbounded]
	pub(super) type Trades<T: Config> =
		StorageMap<_, Blake2_128Concat, u128, (T::AccountId, T::BlockNumber), OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	impl<T: Config> Pallet<T> {
		/// The account ID of the pot for all trade pairs
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		/// Return the amount of money in the pot for an asset
		pub fn pot(asset_id: AssetIdOf<T>) -> BalanceOf<T> {
			T::Assets::balance(asset_id, &Self::account_id())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			// amountA: u128,
			// amountB: u128,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			let _bal = Self::pot(asset_a);

			Ok(())
		}
	}
}
