use molecule::integer::{pack_fixvec, IntegerError};

#[test]
fn rejects_zero_width_before_encoding_any_element() {
    for values in [&[][..], &[1u16][..]] {
        assert!(matches!(
            pack_fixvec(values, |_| -> [u8; 0] {
                panic!("zero-width encoder was called")
            }),
            Err(IntegerError::ZeroWidth)
        ));
    }
}

#[test]
fn empty_and_nonempty_vectors_keep_their_count_and_byte_order() {
    for (values, expected) in [
        (&[][..], &[0, 0, 0, 0][..]),
        (&[0x0102u16, 0x0304][..], &[2, 0, 0, 0, 2, 1, 4, 3][..]),
    ] {
        let wire = pack_fixvec(values, |value| value.to_le_bytes()).unwrap();
        assert_eq!(&wire[..], expected);
    }
}
