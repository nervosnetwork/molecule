#![allow(clippy::cognitive_complexity)]

use std::{convert::TryFrom, iter::FromIterator};

use molecule::prelude::*;

use molecule_ci_tests::testset;

macro_rules! build_empty {
    ($type:ident) => {
        let expected = $type::default();
        let builder = $type::new_builder();
        assert_eq!(
            expected.as_slice().len(),
            builder.expected_length(),
            "failed to check expected length for {}'s builder",
            $type::NAME
        );
        let result = builder.build();
        assert_eq!(
            result.as_slice(),
            expected.as_slice(),
            "failed to build empty for {}",
            $type::NAME
        );
    };
}

#[test]
fn build_empty_is_default() {
    testset!(all, build_empty);
}

macro_rules! verify_build_empty {
    ($type:ident) => {
        let empty = $type::new_builder().build();
        let result = $type::from_slice(empty.as_slice());
        if let Err(err) = result {
            panic!("failed to verify build empty for {}: {}", $type::NAME, err);
        }
    };
}

#[test]
fn build_empty_can_verify() {
    testset!(all, verify_build_empty);
}

#[test]
fn test_conversion() {
    use molecule_ci_tests::types::*;

    assert_eq!(
        Byte11::try_from(&[3; 11][..]).unwrap().as_bytes(),
        &[3; 11][..],
    );
    assert_eq!(
        u32::from_le_bytes(Byte4::from(3u32.to_le_bytes()).into()),
        3u32,
    );
    let _ = BytesVecOpt::from(BytesVec::from_iter([Bytes::from_iter([3, 4])]));
    let _ = UnionA::from(Byte::from(3u8));
}
