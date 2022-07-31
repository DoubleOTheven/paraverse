#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use core::ops::Div;

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::fungibles::{Inspect, Mutate, Transfer},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	type AssetIdOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::AssetId;
	type BalanceOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Assets: Inspect<Self::AccountId> + Transfer<Self::AccountId> + Mutate<Self::AccountId>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LiquidityProvided(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		LPTokensMinted(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
		// LP, pool ID, amount
		LiquitdityClaimed(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
		// Caller, pool ID, amount A, amount B
		TokensSwapped(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		PriceSet(AssetIdOf<T>, BalanceOf<T>, T::BlockNumber),
		PriceOraclePermissionSet(T::AccountId, bool),
	}
	#[pallet::error]
	pub enum Error<T> {
		AddLiquidityFailed,
		InsufficientBalance,
		NotAuthorized,
		DexNotFound,
		UnequalPair,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Price<T: Config> =
		StorageMap<_, Twox128, AssetIdOf<T>, (BalanceOf<T>, T::BlockNumber), OptionQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type PriceOracle<T: Config> =
		StorageMap<_, Twox128, T::AccountId, bool, OptionQuery>;

	// Use Blake hasher bc/ I plan to allow anyone to create a DEX in the future
	// Key is the Pool ID in u64, then a tuple of (asset A ID, asset B ID, LP Token ID)
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Pools<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		(AssetIdOf<T>, AssetIdOf<T>, AssetIdOf<T>),
		OptionQuery,
	>;

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
			pool_id: u64,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let result = Pools::<T>::get(pool_id);
			ensure!(result.is_some(), Error::<T>::DexNotFound);

			let (asset_a, asset_b, lp) = result.unwrap();
			let first_bal = T::Assets::balance(asset_a, &sender);
			let second_bal = T::Assets::balance(asset_b, &sender);
			ensure!(
				amount_a <= first_bal && amount_b <= second_bal,
				Error::<T>::InsufficientBalance,
			);

			let (price_a, _) = Price::<T>::get(asset_a).unwrap_or_default();
			let (price_b, _) = Price::<T>::get(asset_b).unwrap_or_default();

			ensure!(amount_a * price_a == amount_b * price_b, Error::<T>::UnequalPair);

			// TODO Calculate LP Share accurately
			let lp_share = amount_a + amount_b;

			T::Assets::transfer(asset_a, &sender, &Self::account_id(), amount_a, true)?;
			T::Assets::transfer(asset_b, &sender, &Self::account_id(), amount_b, true)?;
			T::Assets::mint_into(lp, &sender, lp_share)?;

			Self::deposit_event(Event::LPTokensMinted(sender.clone(), lp, lp_share));
			Self::deposit_event(Event::LiquidityProvided(sender.clone(), lp, amount_a, amount_b));

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn claim_liquidity(
			origin: OriginFor<T>,
			pool_id: u64,
			lp_amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let result = Pools::<T>::get(pool_id);
			ensure!(result.is_some(), Error::<T>::DexNotFound);

			let (asset_a, asset_b, lp) = result.unwrap();
			let lp_bal = T::Assets::balance(lp, &sender);
			ensure!(lp_bal >= lp_amount, Error::<T>::InsufficientBalance);

			// TODO calculate shares accurately
			let amount_a = &lp_amount.div(2u32.into());
			// let amount_a = T::Assets::div(lp_amount, 2);
			let amount_b = amount_a;

			T::Assets::burn_from(lp, &sender, lp_amount)?;
			T::Assets::transfer(asset_a, &Self::account_id(), &sender, *amount_a, true)?;
			T::Assets::transfer(asset_b, &Self::account_id(), &sender, *amount_b, true)?;

			Self::deposit_event(Event::LiquitdityClaimed(sender.clone(), lp, lp_amount));

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
			asset_id: AssetIdOf<T>,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(PriceOracle::<T>::get(&sender).is_some(), Error::<T>::NotAuthorized);

			let current_block = <frame_system::Pallet<T>>::block_number();
			Price::<T>::insert(asset_id, (price, current_block));
			Self::deposit_event(Event::PriceSet(asset_id, price, current_block));

			Ok(())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config>
	where
		AssetIdOf<T>: MaybeSerializeDeserialize,
	{
		pub prices: Vec<(AssetIdOf<T>, BalanceOf<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T>
	where
		AssetIdOf<T>: MaybeSerializeDeserialize,
	{
		fn default() -> Self {
			Self { prices: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
	where
		AssetIdOf<T>: MaybeSerializeDeserialize,
	{
		fn build(&self) {
			for (asset_id, price) in &self.prices {
				Price::<T>::insert(asset_id, price);
			}
		}
	}
}
