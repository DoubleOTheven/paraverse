use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(TokenURILimit))]
pub struct ItemDetails<AccountId, TokenURILimit: Get<u32>> {
	pub(super) owner: AccountId,
	pub(super) approved: Option<AccountId>,
	pub(super) is_soul_bound: bool,
	pub(super) token_uri: BoundedVec<u8, TokenURILimit>,
}
