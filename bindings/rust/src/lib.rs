pub mod error;
pub mod prelude;

pub use bytes;
pub use faster_hex;

// little endian
pub type ItemId = u32;
// size of item id
pub const ITEM_ID_SIZE: usize = 4;

#[inline]
pub fn extract_item_id(slice: &[u8]) -> ItemId {
    #[allow(clippy::cast_ptr_alignment)]
    let le = slice.as_ptr() as *const ItemId;
    ItemId::from_le(unsafe { *le })
}
