#![no_std]

pub mod capi;

pub mod types {
    #![allow(clippy::all)]
    pub use molecule::prelude::{Byte, ByteReader};
    include!(concat!(env!("OUT_DIR"), "/", "types", ".rs"));
}

#[cfg(test)]
mod test_vectors {
    mod default {
        use crate::types::*;
        use molecule::prelude::*;
        molecule_tests_utils_rust::load_tests!(
            "../../test/schemas/types.mol",
            "../../test/vectors/default.yaml",
        );
    }
    mod simple {
        use crate::types::*;
        use molecule::prelude::*;
        molecule_tests_utils_rust::load_tests!(
            "../../test/schemas/types.mol",
            "../../test/vectors/simple.yaml"
        );
    }
}

#[macro_export]
macro_rules! testset {
    (array, $callback:ident) => {
        use $crate::types::*;
        $callback!(Byte2);
        $callback!(Byte3);
        $callback!(Byte4);
        $callback!(Byte5);
        $callback!(Byte6);
        $callback!(Byte7);
        $callback!(Byte8);
        $callback!(Byte9);
        $callback!(Byte10);
        $callback!(Byte11);
        $callback!(Byte12);
        $callback!(Byte13);
        $callback!(Byte14);
        $callback!(Byte15);
        $callback!(Byte16);
        $callback!(Word);
        $callback!(Word2);
        $callback!(Word3);
        $callback!(Word4);
        $callback!(Word5);
        $callback!(Word6);
        $callback!(Word7);
        $callback!(Word8);
        $callback!(Byte3x3);
        $callback!(Byte5x3);
        $callback!(Byte7x3);
        $callback!(Byte9x3);

        $callback!(StructIx3);
    };
    (struct, $callback:ident) => {
        use $crate::types::*;
        $callback!(StructA);
        $callback!(StructB);
        $callback!(StructC);
        $callback!(StructD);
        $callback!(StructE);
        $callback!(StructF);
        $callback!(StructG);
        $callback!(StructH);
        $callback!(StructI);
        $callback!(StructJ);

        $callback!(StructO);
        $callback!(StructP);
    };
    (fixvec, $callback:ident) => {
        use $crate::types::*;
        $callback!(Bytes);
        $callback!(Words);
        $callback!(Byte3Vec);
        $callback!(Byte7Vec);
        $callback!(StructIVec);
        $callback!(StructJVec);
        $callback!(StructPVec);
    };
    (dynvec, $callback:ident) => {
        $callback!(BytesVec);
        $callback!(WordsVec);

        $callback!(ByteOptVec);
        $callback!(WordOptVec);
        $callback!(WordsOptVec);
        $callback!(BytesOptVec);
    };
    (table, $callback:ident) => {
        use $crate::types::*;
        $callback!(Table0);
        $callback!(Table1);
        $callback!(Table2);
        $callback!(Table3);
        $callback!(Table4);
        $callback!(Table5);
        $callback!(Table6);

        $callback!(TableA);
    };
    (option, $callback:ident) => {
        use $crate::types::*;
        $callback!(ByteOpt);
        $callback!(WordOpt);
        $callback!(StructAOpt);
        $callback!(StructPOpt);
        $callback!(BytesOpt);
        $callback!(WordsOpt);
        $callback!(BytesVecOpt);
        $callback!(WordsVecOpt);
        $callback!(Table0Opt);
        $callback!(Table6Opt);
        $callback!(Table6OptOpt);
    };
    (union, $callback:ident) => {
        use $crate::types::*;
        $callback!(UnionA);
        $callback!(UnionB);
    };
    (all, $callback:ident) => {
        use $crate::types::*;
        testset!(array, $callback);
        testset!(struct, $callback);
        testset!(fixvec, $callback);
        testset!(dynvec, $callback);
        testset!(table, $callback);
        testset!(option, $callback);
        testset!(union, $callback);
        $callback!(AllInOne);
    };
}
