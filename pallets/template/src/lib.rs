#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, sp_runtime::traits::AccountIdConversion, PalletId};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LiquidityProvided(T::AccountId, u64, u128, u128),
		LPTokensMinted(T::AccountId, u64, u128),
		LPTokensBurnt(T::AccountId, u64, u128),
		TokensSwapped(T::AccountId, u64, u128, u128),

		PriceSet(T::AssetId, T::Balance, T::BlockNumber),
		PriceOraclePermissionSet(T::AccountId, bool),
	}
	#[pallet::error]
	pub enum Error<T> {
		AddLiquidityFailed,
		InsufficientBalance,
		NotAuthorized,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Price<T: Config> = StorageMap<_, Twox128, T::AssetId, T::Balance, OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type PriceOracle<T: Config> =
		StorageMap<_, Twox128, T::AccountId, bool, OptionQuery>;

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
		pub fn pot(asset_id: T::AssetId) -> T::Balance {
			pallet_assets::Pallet::<T>::balance(asset_id, &Self::account_id())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_a: T::AssetId,
			// amountA: u128,
			// amountB: u128,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			let _bal = Self::pot(asset_a);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn authorize_pricing_oracle(
			origin: OriginFor<T>,
			who: T::AccountId,
			is_permissioned: bool,
		) -> DispatchResult {
			ensure_root(origin)?;
			PriceOracle::<T>::insert(&who, is_permissioned);
			Self::deposit_event(Event::PriceOraclePermissionSet(who, is_permissioned));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			price: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(PriceOracle::<T>::get(&sender).is_some(), Error::<T>::NotAuthorized);

			let current_block = <frame_system::Pallet<T>>::block_number();
			Price::<T>::insert(asset_id, price);
			Self::deposit_event(Event::PriceSet(asset_id, price, current_block));

			Ok(())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub prices: Vec<(T::AssetId, T::Balance)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { prices: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (asset_id, price) in &self.prices {
				Price::<T>::insert(asset_id, price);
			}
		}
	}
}
