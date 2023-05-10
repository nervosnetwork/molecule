pub use crate as molecule_std;

pub use molecule::*;

#[cfg(feature = "with-primitive-types")]
pub mod primitives {
    include!(concat!(env!("OUT_DIR"), "/primitive_types.rs"));
}

pub mod prelude {
    pub use molecule::prelude::*;

    #[cfg(feature = "with-primitive-types")]
    pub use crate::primitives::*;
}

#[cfg(feature = "with-primitive-types")]
mod primitives_pack;

pub mod pack {
    use crate::prelude::Entity;
    pub trait Unpack<T> {
        /// Unpack binary data into rust types.
        fn unpack(&self) -> T;
    }

    /// A syntactic sugar to convert a rust type into binary data.
    pub trait Pack<T: Entity> {
        /// Packs a rust type into binary data.
        fn pack(&self) -> T;
    }

    #[cfg(feature = "with-primitive-types")]
    pub use crate::primitives_pack::*;
}
