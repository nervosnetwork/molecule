/* TODO Generated from schemas. */

#![allow(clippy::unnecessary_operation)]

use molecule::prelude::*;
use proptest::prelude::*;

use molecule_ci_tests::{capi, types};

const PROB_IS_SOME: f64 = 0.618;

fn arbitrary_byte() -> impl Strategy<Value = types::Byte> {
    any::<u8>().prop_map(types::Byte::new)
}

fn arbitrary_word() -> impl Strategy<Value = types::Word> {
    prop::collection::vec(any::<u8>(), 2).prop_map(|data| types::Word::new_unchecked(data.into()))
}

fn arbitrary_word2() -> impl Strategy<Value = types::Word2> {
    prop::collection::vec(any::<u8>(), 4).prop_map(|data| types::Word2::new_unchecked(data.into()))
}

fn arbitrary_structa() -> impl Strategy<Value = types::StructA> {
    prop::collection::vec(any::<u8>(), 6)
        .prop_map(|data| types::StructA::new_unchecked(data.into()))
}

fn arbitrary_bytes() -> impl Strategy<Value = types::Bytes> {
    prop::collection::vec(any::<u8>(), 0..32)
        .prop_map(|data| data.into_iter().map(types::Byte::new).collect::<Vec<_>>())
        .prop_map(|data| types::Bytes::new_builder().extend(data).build())
}

fn arbitrary_words() -> impl Strategy<Value = types::Words> {
    prop::collection::vec(arbitrary_word(), 0..32)
        .prop_map(|data| types::Words::new_builder().extend(data).build())
}

fn arbitrary_bytesvec() -> impl Strategy<Value = types::BytesVec> {
    prop::collection::vec(arbitrary_bytes(), 0..32)
        .prop_map(|data| types::BytesVec::new_builder().extend(data).build())
}

fn arbitrary_bytesopt() -> impl Strategy<Value = types::BytesOpt> {
    prop::option::weighted(PROB_IS_SOME, arbitrary_bytes())
        .prop_map(|data| types::BytesOpt::new_builder().set(data).build())
}

fn arbitrary_table0() -> impl Strategy<Value = types::Table0> {
    prop::strategy::Just(types::Table0::default())
}

fn arbitrary_table1() -> impl Strategy<Value = types::Table1> {
    any::<u8>().prop_map(|data| types::Table1::new_builder().f1(data.into()).build())
}

fn arbitrary_table5() -> impl Strategy<Value = types::Table5> {
    (
        arbitrary_byte(),
        arbitrary_word2(),
        arbitrary_structa(),
        arbitrary_bytes(),
        arbitrary_bytesvec(),
    )
        .prop_map(|(d1, d2, d3, d4, d5)| {
            types::Table5::new_builder()
                .f1(d1)
                .f2(d2)
                .f3(d3)
                .f4(d4)
                .f5(d5)
                .build()
        })
}

fn arbitrary_table6() -> impl Strategy<Value = types::Table6> {
    (
        arbitrary_byte(),
        arbitrary_word2(),
        arbitrary_structa(),
        arbitrary_bytes(),
        arbitrary_bytesvec(),
        arbitrary_table5(),
    )
        .prop_map(|(d1, d2, d3, d4, d5, d6)| {
            types::Table6::new_builder()
                .f1(d1)
                .f2(d2)
                .f3(d3)
                .f4(d4)
                .f5(d5)
                .f6(d6)
                .build()
        })
}

fn arbitrary_table6opt() -> impl Strategy<Value = types::Table6Opt> {
    prop::option::weighted(PROB_IS_SOME, arbitrary_table6())
        .prop_map(|data| types::Table6Opt::new_builder().set(data).build())
}

fn arbitrary_uniona() -> impl Strategy<Value = types::UnionA> {
    prop_oneof![
        1 => arbitrary_byte().prop_map(Into::into),
        1 => arbitrary_word().prop_map(Into::into),
        1 => arbitrary_structa().prop_map(Into::into),
        1 => arbitrary_bytes().prop_map(Into::into),
        1 => arbitrary_words().prop_map(Into::into),
        1 => arbitrary_table0().prop_map(Into::into),
        1 => arbitrary_table6().prop_map(Into::into),
        1 => arbitrary_table6opt().prop_map(Into::into),
    ]
    .prop_map(|data: types::UnionAUnion| types::UnionA::new_builder().set(data).build())
}

fn arbitrary_tablea() -> impl Strategy<Value = types::TableA> {
    (
        arbitrary_word2(),
        arbitrary_structa(),
        arbitrary_bytes(),
        arbitrary_bytesvec(),
        arbitrary_table1(),
        arbitrary_bytesopt(),
        arbitrary_uniona(),
        arbitrary_byte(),
    )
        .prop_map(|(d1, d2, d3, d4, d5, d6, d7, d8)| {
            types::TableA::new_builder()
                .f1(d1)
                .f2(d2)
                .f3(d3)
                .f4(d4)
                .f5(d5)
                .f6(d6)
                .f7(d7)
                .f8(d8)
                .build()
        })
}

fn arbitrary_inputs() -> impl Strategy<Value = types::TableA> {
    arbitrary_tablea()
}

proptest! {
    #[test]
    fn random_input(input in prop::collection::vec(any::<u8>(), 0..4096)) {
        if let Ok(lucky) = types::TableA::from_slice(&input) {
            let f1 = lucky.f1();
            let f2 = lucky.f2();
            let f3 = lucky.f3();
            let f4 = lucky.f4();
            let f5 = lucky.f5();
            let f6 = lucky.f6();
            let f7 = lucky.f7();
            let f8 = lucky.f8();
            let copied = types::TableA::new_builder()
                .f1(f1)
                .f2(f2)
                .f3(f3)
                .f4(f4)
                .f5(f5)
                .f6(f6)
                .f7(f7)
                .f8(f8)
                .build();
            assert_eq!(
                lucky.as_slice(), copied.as_slice(),
                "\nlucky : {:#x};\ncopied: {:#x};\nError(rust): data should be same with the copied data\n",
                lucky, copied,
            );
            assert!(capi::tablea_verify(&input), "\nError(c): the data should be valid\n");
        } else {
            assert!(!capi::tablea_verify(&input), "\nError(c): the data should be invalid\n");
        }
    }

    #[test]
    fn arbitrary_valid_input(input in arbitrary_inputs()) {
        let result = types::TableA::from_slice(input.as_slice());
        assert!(
            result.is_ok(),
            "\ninput: {:?};\nError(rust): input must be valid, but {}\n",
            input.as_slice(),
            result.unwrap_err(),
        );
        assert!(capi::tablea_verify(input.as_slice()), "\nError(c): the data should be valid\n");
    }
}
