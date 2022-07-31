use core::ops::Div;

use sp_runtime::traits::IntegerSquareRoot;
use sp_std::ops::Mul;

pub struct DexPricer;

pub enum Pair {
	A,
	B,
}

// returns a tuple of (initial LP token amount, constant K)
impl DexPricer {
	pub fn initial_pool_values<T: IntegerSquareRoot + Mul<Output = T>>(
		contribution_a: T,
		contribution_b: T,
	) -> (T, T) {
		let product = contribution_a * contribution_b;
		(product.integer_sqrt(), product)
	}

	// total_a is the amount before the contribution
	pub fn to_contribution_lp_amount<T: Mul<Output = T> + Div<Output = T>>(
		contribution_a: T,
		lp_total_supply: T,
		total_a: T,
	) -> T {
		(contribution_a / total_a) * lp_total_supply
	}

	pub fn token_price<T: Div<Output = T>>(token: Pair, total_a: T, total_b: T) -> T {
		match token {
			Pair::A => return total_b / total_a,
			Pair::B => return total_a / total_b,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
