#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
mod dex_pricer;

#[frame_support::pallet]
pub mod pallet {
	use crate::dex_pricer::{DexPricer, TokenPair};
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
		// (who, lp token ID, A contribution, B contribution)
		LiquidityProvided(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		// (who, lp token ID, amount)
		LPTokensMinted(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
		// (LP, pool ID, amount)
		LiquitdityClaimed(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
		// (Caller, pool ID, amount A, amount B)
		TokensSwapped(T::AccountId, AssetIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		// (token ID, price, block)
		PriceSet(AssetIdOf<T>, BalanceOf<T>, T::BlockNumber),
		// (account, has_permission)
		PriceOraclePermissionSet(T::AccountId, bool),
		// (pool ID, asset A ID, asset B ID)
		PoolCreated(AssetIdOf<T>, AssetIdOf<T>, AssetIdOf<T>),
		// (pool ID, From Asset ID, amount)
		AssetsSwapped(AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
	}
	#[pallet::error]
	pub enum Error<T> {
		AddLiquidityFailed,
		InsufficientBalance,
		NotAuthorized,
		PoolExists,
		DexNotFound,
		UnequalPair,
		UnableToSwap,
		TokenNotInPool,
		SwapExceedsFunds,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Price<T: Config> =
		StorageMap<_, Twox128, AssetIdOf<T>, (BalanceOf<T>, T::BlockNumber), OptionQuery>;

	// This is used to store real world values per token. NOTE: in AMMs the swap price is relative
	// to the other token in the pair. Having a real world price for each token is useful for
	// artbitrage opportunities.
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type PriceOracle<T: Config> =
		StorageMap<_, Twox128, T::AccountId, bool, OptionQuery>;

	// TODO: Change the pool ID from a u64 to a hash of the Pair. This can prevent duplicate pools,
	// although currently the "root" sets the initial pools Use Blake hasher bc/ I plan to allow
	// anyone to create a DEX in the future Key is the Pool ID in u64, then a tuple of (asset A ID,
	// asset B ID, LP Token ID)
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Pools<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		AssetIdOf<T>,
		(AssetIdOf<T>, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>),
		OptionQuery,
	>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	impl<T: Config> Pallet<T> {
		/// The account ID of the pot for all trade pairs
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		/// Return the amount of money in the pot for an asset
		fn pot(asset_id: AssetIdOf<T>) -> BalanceOf<T> {
			T::Assets::balance(asset_id, &Self::account_id())
		}

		fn take_from_pot(
			asset_id: AssetIdOf<T>,
			receiver: &T::AccountId,
			amount: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			T::Assets::transfer(asset_id, &Self::account_id(), receiver, amount, false)
		}

		fn add_to_pot(
			asset_id: AssetIdOf<T>,
			from: &T::AccountId,
			amount: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			T::Assets::transfer(asset_id, &Self::account_id(), &from, amount, false)
		}

		fn mint(
			asset_id: AssetIdOf<T>,
			receiver: &T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::Assets::mint_into(asset_id, receiver, amount)
		}

		fn burn(
			asset_id: AssetIdOf<T>,
			holder: &T::AccountId,
			amount: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			T::Assets::burn_from(asset_id, holder, amount)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		pub fn swap(
			origin: OriginFor<T>,
			pool_id: AssetIdOf<T>,
			from_asset_id: AssetIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let result = Pools::<T>::get(pool_id);
			ensure!(result.is_some(), Error::<T>::DexNotFound);

			// Get the pool data
			let (asset_a, asset_b, _, _) = result.unwrap();
			let fee_numerator = 5u32; // 0.5% -> TODO keep this in storage so each pool can have different fees
			let fee_denominator = 1000u32;
			ensure!(
				from_asset_id == asset_a || from_asset_id == asset_b,
				Error::<T>::TokenNotInPool,
			);

			// Calculate the swap amount and pool fee
			let mut from_asset_amount = TokenPair::A(amount);
			if asset_b == from_asset_id {
				from_asset_amount = TokenPair::B(amount);
			}
			let total_a = Self::pot(asset_a);
			let total_b = Self::pot(asset_b);
			let swap_price_result = DexPricer::to_swap_values(
				&from_asset_amount,
				&total_a,
				&total_b,
				fee_numerator,
				fee_denominator,
			);
			ensure!(swap_price_result.is_ok(), Error::<T>::UnableToSwap);

			let (other_amount, _) = swap_price_result.ok().unwrap();

			match from_asset_amount {
				TokenPair::A(_) => {
					// Check user balance
					let user_a_balance = T::Assets::balance(asset_a, &sender);
					ensure!(user_a_balance >= amount, Error::<T>::InsufficientBalance);

					// Check swap amount against pot balance
					let pot_b_balance = Self::pot(asset_b);
					ensure!(pot_b_balance <= other_amount, Error::<T>::SwapExceedsFunds);

					Self::add_to_pot(asset_a, &sender, amount)?;
					Self::take_from_pot(asset_b, &sender, other_amount)?;
				},
				TokenPair::B(_) => {
					// Check user balance
					let user_b_balance = T::Assets::balance(asset_b, &sender);
					ensure!(user_b_balance >= amount, Error::<T>::InsufficientBalance);

					// Check swap amount against pot balance
					let pot_a_balance = Self::pot(asset_a);
					ensure!(pot_a_balance <= other_amount, Error::<T>::SwapExceedsFunds);

					Self::add_to_pot(asset_b, &sender, amount)?;
					Self::take_from_pot(asset_a, &sender, other_amount)?;
				},
			}

			Self::deposit_event(Event::AssetsSwapped(pool_id, from_asset_id, amount));

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: AssetIdOf<T>,
			contribution_a: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let result = Pools::<T>::get(pool_id);
			ensure!(result.is_some(), Error::<T>::DexNotFound);

			// Get the token IDs for the pool
			let (asset_a, asset_b, lp, _) = result.unwrap();

			// Calculate the current price of Asset A and Asset B
			let total_a = Self::pot(asset_a);
			let total_b = Self::pot(asset_b);
			let (price_of_a, price_of_b, _) = DexPricer::token_prices(&total_a, &total_b);

			// Calculate B amount for Asset A contribution. A/B contributions must be equal value
			let equal_amount_b = (contribution_a * price_of_a) / price_of_b;
			let asset_a_balance = T::Assets::balance(asset_a, &sender);
			let asset_b_balance = T::Assets::balance(asset_b, &sender);
			ensure!(
				contribution_a <= asset_a_balance && equal_amount_b <= asset_b_balance,
				Error::<T>::InsufficientBalance,
			);

			// Calculate LP tokens
			let total_lp = Self::pot(lp);
			let contribution_lp_amount =
				DexPricer::to_contribution_lp_amount(contribution_a, total_lp, total_a);

			// Transfer funds
			Self::add_to_pot(asset_a, &sender, contribution_a)?;
			Self::add_to_pot(asset_a, &sender, equal_amount_b)?;
			Self::mint(lp, &sender, contribution_lp_amount)?;

			Self::deposit_event(Event::LPTokensMinted(sender.clone(), lp, contribution_lp_amount));
			Self::deposit_event(Event::LiquidityProvided(
				sender.clone(),
				pool_id,
				contribution_a,
				equal_amount_b,
			));

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn claim_liquidity(
			origin: OriginFor<T>,
			pool_id: AssetIdOf<T>,
			lp_claim: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let result = Pools::<T>::get(pool_id);
			ensure!(result.is_some(), Error::<T>::DexNotFound);

			let (asset_a, asset_b, lp, _) = result.unwrap();
			let lp_balance = T::Assets::balance(lp, &sender);
			ensure!(lp_claim <= lp_balance, Error::<T>::InsufficientBalance);

			// Calculate asset A and B shares from LP tokens
			let total_lp = Self::pot(lp);
			let total_a = Self::pot(asset_a);
			let total_b = Self::pot(asset_b);
			let (amount_a, amount_b) = DexPricer::from_lp(&lp_claim, &total_a, &total_b, &total_lp);

			Self::burn(lp, &sender, lp_claim)?;
			Self::take_from_pot(asset_a, &sender, amount_a)?;
			Self::take_from_pot(asset_a, &sender, amount_b)?;

			Self::deposit_event(Event::LiquitdityClaimed(sender.clone(), lp, lp_claim));

			Ok(())
		}

		#[pallet::weight((1_000_000, Pays::Yes))]
		pub fn authorize_pricing_oracle(
			origin: OriginFor<T>,
			who: T::AccountId,
			is_permissioned: bool,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			PriceOracle::<T>::insert(&who, is_permissioned);
			Self::deposit_event(Event::PriceOraclePermissionSet(who, is_permissioned));
			Ok(Pays::No.into())
		}

		#[pallet::weight((1_000_000, Pays::Yes))]
		pub fn create_pool(
			origin: OriginFor<T>,
			pool_id: AssetIdOf<T>,
			asset_a_id: AssetIdOf<T>,
			asset_b_id: AssetIdOf<T>,
			contribution_a: BalanceOf<T>,
			contribution_b: BalanceOf<T>,
			lp_id: AssetIdOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin.clone())?;
			let admin = ensure_signed(origin)?;
			let pool = Pools::<T>::get(pool_id);
			ensure!(pool.is_none(), Error::<T>::PoolExists);

			let bal_a = T::Assets::balance(asset_a_id, &admin);
			let bal_b = T::Assets::balance(asset_b_id, &admin);
			ensure!(contribution_a <= bal_a, Error::<T>::InsufficientBalance);
			ensure!(contribution_b <= bal_b, Error::<T>::InsufficientBalance);

			let (lp_amount, constant_k) =
				DexPricer::initial_pool_values(contribution_a, contribution_b);

			Self::add_to_pot(asset_a_id, &admin, contribution_a)?;
			Self::add_to_pot(asset_b_id, &admin, contribution_b)?;
			Self::mint(lp_id, &admin, lp_amount)?;

			Pools::<T>::insert(pool_id, (asset_a_id, asset_b_id, lp_id, constant_k));
			Self::deposit_event(Event::PoolCreated(asset_a_id, asset_b_id, lp_id));
			Self::deposit_event(Event::LiquidityProvided(
				admin.clone(),
				pool_id,
				contribution_a,
				contribution_b,
			));

			Ok(Pays::No.into())
		}

		#[pallet::weight((1_000_000, Pays::Yes))]
		pub fn set_price(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			price: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(PriceOracle::<T>::get(&sender).is_some(), Error::<T>::NotAuthorized);

			let current_block = <frame_system::Pallet<T>>::block_number();
			Price::<T>::insert(asset_id, (price, current_block));
			Self::deposit_event(Event::PriceSet(asset_id, price, current_block));

			Ok(Pays::No.into())
		}
	}
}
