#![allow(clippy::cognitive_complexity)]

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
