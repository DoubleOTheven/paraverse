use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, TypeInfo, MaxEncodedLen)]
pub struct SaleItem<AccountId, Id, AssetId, ItemId, Price> {
	pub(super) owner: AccountId,
	pub(super) id: Id,
	pub(super) item_id: ItemId,
	pub(super) asset_id: AssetId,
	pub(super) price: Price,
}
