#![no_std]

#[allow(clippy::all)]
/// Numeric access is opt-in; unannotated byte arrays (including `Byte32` and `Uint256`) remain opaque.
/// ```compile_fail
/// use molecule::prelude::*;
/// use molecule_integer_tests::packed::Byte32;
/// let _ = Byte32::default().as_reader().try_as_wire_bytes();
/// ```
/// ```compile_fail
/// use molecule::prelude::*;
/// use molecule_integer_tests::blockchain::Uint256;
/// let _ = Uint256::default().as_reader().try_as_wire_bytes();
/// ```
/// The decoded iterator borrows the wire buffer and can outlive its reader.
/// ```edition2024
/// # mod schema { include!(concat!(env!("OUT_DIR"), "/packed/numbers.rs")); }
/// use molecule::prelude::*;
/// fn amounts(wire: &[u8]) -> impl ExactSizeIterator<Item = u64> + '_ {
///     let reader = schema::AmountsReader::from_slice(wire).unwrap();
///     reader.try_values().unwrap()
/// }
/// assert_eq!(amounts(&[0; 4]).len(), 0);
/// ```
pub mod packed {
    include!(concat!(env!("OUT_DIR"), "/packed/numbers.rs"));
}
#[allow(clippy::all)]
pub mod lazy {
    include!(concat!(env!("OUT_DIR"), "/lazy/numbers.rs"));
}
#[allow(clippy::all)]
pub mod legacy {
    include!(concat!(env!("OUT_DIR"), "/legacy/numbers.rs"));
}
#[allow(clippy::all)]
pub mod blockchain {
    include!(concat!(env!("OUT_DIR"), "/packed/blockchain.rs"));
}
#[allow(clippy::all)]
pub mod extensions {
    include!(concat!(env!("OUT_DIR"), "/packed/extensions.rs"));
}
#[allow(clippy::all)]
pub mod protocols {
    include!(concat!(env!("OUT_DIR"), "/packed/protocols.rs"));
}
#[allow(clippy::all)]
pub mod old_blockchain {
    include!(concat!(env!("OUT_DIR"), "/legacy/blockchain.rs"));
}

// Existing downstream conveniences remain coherent with the new named methods.
impl From<u64> for packed::U64Le {
    fn from(value: u64) -> Self {
        <Self as From<[u8; 8]>>::from(value.to_le_bytes())
    }
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod allocation;
#[cfg(test)]
mod tests;
