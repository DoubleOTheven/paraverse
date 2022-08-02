use core::ops::Div;

use sp_runtime::traits::{IntegerSquareRoot, Saturating};
use sp_std::ops::Mul;

pub struct DexPricer;

pub enum TokenPair {
	A,
	B,
}

fn add_decimals<T: Saturating + From<u32>>(value: T, decimals: u8) -> T {
	let decimal_multiplier = <u32 as Into<T>>::into(10_u32).saturating_pow(decimals.into()).into();
	(value.saturating_mul(decimal_multiplier)).into()
}

fn remove_decimals<T: Saturating + Div<Output = T> + From<u32>>(value: T, decimals: u8) -> T {
	let decimal_multiplier = <u32 as Into<T>>::into(10_u32).saturating_pow(decimals.into()).into();
	value / decimal_multiplier
}

impl DexPricer {
	pub fn initial_pool_values<T: IntegerSquareRoot + Mul<Output = T>>(
		contribution_a: T,
		contribution_b: T,
	) -> (T, T) {
		let constant_k = contribution_a * contribution_b;
		(constant_k.integer_sqrt(), constant_k)
	}

	pub fn to_contribution_lp_amount<
		T: Saturating + Mul<Output = T> + Div<Output = T> + From<u32>,
	>(
		contribution_a: T,
		lp_total_supply: T,
		total_a: T,
	) -> T {
		let contrib = add_decimals(contribution_a * lp_total_supply, 12);
		remove_decimals(contrib / total_a, 12)
	}

	// Returns (amount A, amount B)
	pub fn from_lp<T: Saturating + Mul<Output = T> + Div<Output = T> + Copy + From<u32>>(
		lp_claim: &T,
		total_a: &T,
		total_b: &T,
		total_lp: &T,
	) -> (T, T) {
		let lp_share = add_decimals(*lp_claim, 12) / *total_lp;
		let amount_a = lp_share * *total_a;
		let amount_b = lp_share * *total_b;
		(remove_decimals(amount_a, 12), remove_decimals(amount_b, 12))
	}

	pub fn token_price<T: Saturating + Div<Output = T> + Mul<Output = T> + From<u32>>(
		token: TokenPair,
		total_a: T,
		total_b: T,
	) -> T {
		match token {
			TokenPair::A => return remove_decimals(add_decimals(total_b, 12) / total_a, 12),
			TokenPair::B => return remove_decimals(add_decimals(total_a, 12) / total_b, 12),
		}
	}

	// pub fn token_prices<T: Div<Output = T> + Mul<Output = T> + From<u32>>(
	// 	total_a: &T,
	// 	total_b: &T,
	// ) -> (T, T) {
	// 	(
	// 		Self::from_precision(Self::to_precision(total_b) / total_a),
	// 		Self::from_precision(Self::to_precision(total_a) / total_b),
	// 	)
	// }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_initial_pool_values() {
		let contribution_a: u128 = 500_000_000_000_000;
		let contribution_b: u128 = 100_000_000_000_000_000;

		let (lp_shares, constant_k) =
			DexPricer::initial_pool_values(contribution_a, contribution_b);

		assert_eq!(lp_shares, 7_071_067_811_865_475);
		assert_eq!(constant_k, 50000000000000000000000000000000);
	}

	#[test]
	fn test_to_contribution_lp_amount() {
		let contribution: u128 = 500_000_000_000_000;
		let total_a: u128 = 100_000_000_000_000_000;
		let total_lp = 7_071_067_811_865_475;

		let first_result = DexPricer::to_contribution_lp_amount(contribution, total_lp, total_a);
		let new_total_a = total_a + contribution;
		let second_result =
			DexPricer::to_contribution_lp_amount(contribution, total_lp, new_total_a);

		assert_eq!(first_result, 3_402_823_669);
		assert_eq!(second_result, 3_385_894_198);
	}

	#[test]
	fn test_from_lp() {
		let lp_claim = 20_000;
		let total_lp = 100_000;

		let total_a = 999_999_999_999;
		let total_b = 696_969_696_969_696;

		let (result_a, result_b) =
			DexPricer::from_lp::<u128>(&lp_claim, &total_a, &total_b, &total_lp);
		let expected_a = 199_999_999_999;
		let expected_b = 139_393_939_393_939;

		assert_eq!(result_a, expected_a);
		assert_eq!(result_b, expected_b);
	}

	#[test]
	fn test_from_lp_for_small_lp_shares() {
		let lp_claim = 1;
		let total_lp = 100_000_000;

		let decimals_18 = 100_000_000_000_000_000;
		let one_billion_a: u128 = 1_000_000_000 * decimals_18;
		let two_billion_b = 2 * one_billion_a;

		let (result_a, result_b) =
			DexPricer::from_lp::<u128>(&lp_claim, &one_billion_a, &two_billion_b, &total_lp);
		let expected_a = 1_000_000_000_000_000_000;
		let expected_b = expected_a * 2;

		assert_eq!(result_a, expected_a);
		assert_eq!(result_b, expected_b);
	}
}
