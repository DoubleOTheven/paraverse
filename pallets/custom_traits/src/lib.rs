#![cfg_attr(not(feature = "std"), no_std)]

pub trait Ownership<Id, AccountId> {
	fn is_owner(id: &Id, who: &AccountId) -> bool;
}

pub trait Transfer<Id, AccountId> {
	fn transfer(id: &Id, to: &AccountId) -> bool;
}
