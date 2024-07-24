#![allow(clippy::cognitive_complexity)]

use molecule::prelude::*;
use slices::u8_slice as s;

use molecule_ci_tests::types;

macro_rules! test_default {
    ($type:ident, $expected:expr) => {
        let result = types::$type::default();
        assert_eq!(
            result.as_slice(),
            &$expected[..],
            "failed to test {} default",
            stringify!($type)
        );
        assert!(
            types::$type::from_slice(result.as_slice()).is_ok(),
            "failed to verify {} default",
            stringify!($type)
        );
    };
}

macro_rules! test_option_set_default {
    ($type:ident, $type_inner:ident) => {
        let expected = types::$type_inner::default();
        let builder = types::$type::new_builder().set(Some(Default::default()));
        assert_eq!(
            expected.as_slice().len(),
            builder.expected_length(),
            "failed to check expected length for {}'s builder with {}",
            stringify!($type),
            stringify!($type_inner),
        );
        let result = builder.build();
        assert_eq!(
            result.as_slice(),
            expected.as_slice(),
            "failed to test {} with {}",
            stringify!($type),
            stringify!($type_inner),
        );
    };
}

macro_rules! test_vector_push_default {
    ($type:ident, $item:ident, $expected1:expr, $expected2:expr, $expected3:expr) => {
        let t = types::$type::default();
        let t = test_vector_push_default!($type, $item, t, $expected1);
        let t = test_vector_push_default!($type, $item, t, $expected2);
        let _ = test_vector_push_default!($type, $item, t, $expected3);
    };
    ($type:ident, $item:ident, $input:ident, $expected:expr) => {{
        let expected = $expected;
        let builder = $input.as_builder().push(types::$item::default());
        let result = builder.build();
        assert_eq!(
            result.as_slice(),
            &expected[..],
            "failed to test {} with {} items",
            stringify!($type),
            result.len(),
        );
        assert!(
            types::$type::from_slice(result.as_slice()).is_ok(),
            "failed to verify {} with {} items",
            stringify!($type),
            result.len(),
        );
        assert_eq!(
            expected.len(),
            builder.expected_length(),
            "failed to check expected length for {}'s builder with {} items",
            stringify!($type),
            result.len(),
        );
        result
    }};
}

#[test]
fn option_default() {
    let slice = s!("0x");
    test_default!(ByteOpt, slice);
    test_default!(WordOpt, slice);
    test_default!(StructAOpt, slice);
    test_default!(StructPOpt, slice);
    test_default!(BytesOpt, slice);
    test_default!(WordsOpt, slice);
    test_default!(BytesVecOpt, slice);
    test_default!(WordsVecOpt, slice);
    test_default!(Table0Opt, slice);
    test_default!(Table6Opt, slice);
    test_default!(Table6OptOpt, slice);
}

#[test]
fn union_default() {
    test_default!(
        UnionA,
        s!("0x\
            02000000\
            00\
            ")
    );
}

#[test]
fn array_default() {
    test_default!(Byte2, s!("0x0000"));
    test_default!(Byte3, s!("0x000000"));
    test_default!(Byte4, s!("0x00000000"));
    test_default!(Byte5, s!("0x00000000_00"));
    test_default!(Byte6, s!("0x00000000_0000"));
    test_default!(Byte7, s!("0x00000000_000000"));
    test_default!(Byte8, s!("0x00000000_00000000"));
    test_default!(Byte9, s!("0x00000000_00000000_00"));
    test_default!(Byte10, s!("0x00000000_00000000_0000"));
    test_default!(Byte11, s!("0x00000000_00000000_000000"));
    test_default!(Byte12, s!("0x00000000_00000000_00000000"));
    test_default!(Byte13, s!("0x00000000_00000000_00000000_00"));
    test_default!(Byte14, s!("0x00000000_00000000_00000000_0000"));
    test_default!(Byte15, s!("0x00000000_00000000_00000000_000000"));
    test_default!(Byte16, s!("0x00000000_00000000_00000000_00000000"));

    test_default!(Word, s!("0x0000"));
    test_default!(Word2, s!("0x00000000"));
    test_default!(Word3, s!("0x00000000_0000"));
    test_default!(Word4, s!("0x00000000_00000000"));
    test_default!(Word5, s!("0x00000000_00000000_0000"));
    test_default!(Word6, s!("0x00000000_00000000_00000000"));
    test_default!(Word7, s!("0x00000000_00000000_00000000_0000"));
    test_default!(Word8, s!("0x00000000_00000000_00000000_00000000"));

    test_default!(
        Byte3x3,
        s!("0x\
            000000\
            000000\
            000000\
            ")
    );
    test_default!(
        Byte5x3,
        s!("0x\
            00000000_00\
            00000000_00\
            00000000_00\
            ")
    );
    test_default!(
        Byte7x3,
        s!("0x\
            00000000_000000\
            00000000_000000\
            00000000_000000\
            ")
    );
    test_default!(
        Byte9x3,
        s!("0x\
            00000000_00000000_00\
            00000000_00000000_00\
            00000000_00000000_00\
            ")
    );
    test_default!(
        StructIx3,
        s!("0x\
            00000000\
            00000000\
            00000000\
            ")
    );
}

#[test]
fn struct_default() {
    test_default!(
        StructA,
        s!("0x\
            00\
            00\
            0000\
            0000\
            ")
    );
    test_default!(
        StructB,
        s!("0x\
            00\
            00\
            0000\
            000000\
            ")
    );
    test_default!(
        StructC,
        s!("0x\
            00\
            00\
            0000\
            00000000\
            ")
    );
    test_default!(
        StructD,
        s!("0x\
            00\
            00\
            0000\
            00000000_00\
            ")
    );
    test_default!(
        StructE,
        s!("0x\
            00\
            0000\
            00\
            0000\
            ")
    );
    test_default!(
        StructF,
        s!("0x\
            00\
            000000\
            00\
            ")
    );
    test_default!(
        StructG,
        s!("0x\
            000000\
            00\
            0000\
            00000000\
            ")
    );
    test_default!(
        StructH,
        s!("0x\
            000000\
            00\
            0000\
            00000000\
            ")
    );
    test_default!(
        StructI,
        s!("0x\
            000000\
            00\
            ")
    );
    test_default!(
        StructJ,
        s!("0x\
            00000000_0000\
            00\
            ")
    );
    test_default!(
        StructO,
        s!("0x\
            00000000_00000000_00000000\
            00\
            ")
    );
    test_default!(
        StructP,
        s!("0x\
            00000000_000000\
            00\
            ")
    );
}

#[test]
fn fixvec_default() {
    let slice = s!("0x00000000");
    test_default!(Bytes, slice);
    test_default!(Words, slice);
    test_default!(Byte3Vec, slice);
    test_default!(Byte7Vec, slice);
    test_default!(StructJVec, slice);
    test_default!(StructPVec, slice);
}

#[test]
fn dynvec_default() {
    let slice = s!("0x04000000");
    test_default!(BytesVec, slice);
    test_default!(WordsVec, slice);
    test_default!(ByteOptVec, slice);
    test_default!(WordOptVec, slice);
    test_default!(WordsOptVec, slice);
    test_default!(BytesOptVec, slice);
}

#[test]
fn table_default() {
    test_default!(
        Table0,
        s!("0x\
            04000000\
            ")
    );
    test_default!(
        Table1,
        s!("0x\
            09000000\
            \
            08000000\
            \
            00\
            ")
    );
    test_default!(
        Table2,
        s!("0x\
            11000000\
            \
            0c000000\
            0d000000\
            \
            00\
            00000000\
            ")
    );
    test_default!(
        Table3,
        s!("0x\
            1b000000\
            \
            10000000\
            11000000\
            15000000\
            \
            00\
            00000000\
            00000000_0000\
            ")
    );
    test_default!(
        Table4,
        s!("0x\
            23000000\
            \
            14000000\
            15000000\
            19000000\
            1f000000\
            \
            00\
            00000000\
            00000000_0000\
            00000000\
            ")
    );
    test_default!(
        Table5,
        s!("0x\
            2b000000\
            \
            18000000\
            19000000\
            1d000000\
            23000000\
            27000000\
            \
            00\
            00000000\
            00000000_0000\
            00000000\
            04000000\
            ")
    );
    test_default!(
        Table6,
        s!("0x\
            5a000000\
            \
            1c000000\
            1d000000\
            21000000\
            27000000\
            2b000000\
            2f000000\
            \
            00\
            00000000\
            00000000_0000\
            00000000\
            04000000\
            \
            2b000000\
            18000000_19000000_1d000000_23000000_27000000\
            00\
            00000000\
            00000000_0000\
            00000000\
            04000000\
            ")
    );
}

#[test]
fn option_set_default() {
    test_option_set_default!(ByteOpt, Byte);
    test_option_set_default!(WordOpt, Word);
    test_option_set_default!(StructAOpt, StructA);
    test_option_set_default!(StructPOpt, StructP);
    test_option_set_default!(BytesOpt, Bytes);
    test_option_set_default!(WordsOpt, Words);
    test_option_set_default!(BytesVecOpt, BytesVec);
    test_option_set_default!(WordsVecOpt, WordsVec);
    test_option_set_default!(Table0Opt, Table0);
    test_option_set_default!(Table6Opt, Table6);
    test_option_set_default!(Table6OptOpt, Table6Opt);
}

#[test]
fn fixvec_push_default() {
    test_vector_push_default!(
        Bytes,
        Byte,
        s!("0x\
            01000000\
            00\
            "),
        s!("0x\
            02000000\
            00\
            00\
            "),
        s!("0x\
            03000000\
            00\
            00\
            00\
            ")
    );
    test_vector_push_default!(
        Words,
        Word,
        s!("0x\
            01000000\
            0000\
            "),
        s!("0x\
            02000000\
            0000\
            0000\
            "),
        s!("0x\
            03000000\
            0000\
            0000\
            0000\
            ")
    );
    test_vector_push_default!(
        Byte3Vec,
        Byte3,
        s!("0x\
            01000000\
            000000\
            "),
        s!("0x\
            02000000\
            000000\
            000000\
            "),
        s!("0x\
            03000000\
            000000\
            000000\
            000000\
            ")
    );
    test_vector_push_default!(
        Byte7Vec,
        Byte7,
        s!("0x\
            01000000\
            00000000_000000\
            "),
        s!("0x\
            02000000\
            00000000_000000\
            00000000_000000\
            "),
        s!("0x\
            03000000\
            00000000_000000\
            00000000_000000\
            00000000_000000\
            ")
    );
    test_vector_push_default!(
        StructIVec,
        StructI,
        s!("0x\
            01000000\
            00000000\
            "),
        s!("0x\
            02000000\
            00000000\
            00000000\
            "),
        s!("0x\
            03000000\
            00000000\
            00000000\
            00000000\
            ")
    );
    test_vector_push_default!(
        StructJVec,
        StructJ,
        s!("0x\
            01000000\
            00000000_000000\
            "),
        s!("0x\
            02000000\
            00000000_000000\
            00000000_000000\
            "),
        s!("0x\
            03000000\
            00000000_000000\
            00000000_000000\
            00000000_000000\
            ")
    );
    test_vector_push_default!(
        StructPVec,
        StructP,
        s!("0x\
            01000000\
            00000000_00000000\
            "),
        s!("0x\
            02000000\
            00000000_00000000\
            00000000_00000000\
            "),
        s!("0x\
            03000000\
            00000000_00000000\
            00000000_00000000\
            00000000_00000000\
            ")
    );
}

#[test]
fn dynvec_push_default() {
    let s1 = s!("0x\
                 0c000000\
                 \
                 08000000\
                 \
                 00000000\
                 ");
    let s2 = s!("0x\
                 14000000\
                 \
                 0c000000\
                 10000000\
                 \
                 00000000\
                 00000000\
                 ");
    let s3 = s!("0x\
                 1c000000\
                 \
                 10000000\
                 14000000\
                 18000000\
                 \
                 00000000\
                 00000000\
                 00000000\
                 ");
    test_vector_push_default!(BytesVec, Bytes, s1, s2, s3);
    test_vector_push_default!(WordsVec, Words, s1, s2, s3);
    let s1 = s!("0x\
                 08000000\
                 \
                 08000000\
                 ");
    let s2 = s!("0x\
                 0c000000\
                 \
                 0c000000\
                 0c000000\
                 ");
    let s3 = s!("0x\
                 10000000\
                 \
                 10000000\
                 10000000\
                 10000000\
                 ");
    test_vector_push_default!(ByteOptVec, ByteOpt, s1, s2, s3);
    test_vector_push_default!(WordOptVec, WordOpt, s1, s2, s3);
    test_vector_push_default!(WordsOptVec, WordsOpt, s1, s2, s3);
    test_vector_push_default!(BytesOptVec, BytesOpt, s1, s2, s3);
}
