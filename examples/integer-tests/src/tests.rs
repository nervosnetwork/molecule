use super::{blockchain, lazy, legacy, old_blockchain, packed};
use molecule::{
    integer::Uint256,
    lazy_reader::{Cursor, Error, Read},
    prelude::*,
};
use std::{boxed::Box, string::String, vec, vec::Vec};

struct Case {
    bits: u16,
    signed: bool,
    order: String,
    value: String,
    bytes: Vec<u8>,
}

fn hex(input: &str) -> Vec<u8> {
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(core::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

// Decimal conversion is an independent test oracle, not a production text API.
fn decimal_u256(input: &str) -> Uint256 {
    let mut bytes = [0; 32];
    for digit in input.bytes() {
        let mut carry = (digit - b'0') as u16;
        for byte in &mut bytes {
            carry += u16::from(*byte) * 10;
            *byte = carry as u8;
            carry >>= 8;
        }
        assert_eq!(carry, 0);
    }
    Uint256::from_le_bytes(bytes)
}

impl Case {
    fn limbs(&self) -> [u64; 4] {
        let mut bytes = self.bytes.clone();
        if self.order == "big" {
            bytes.reverse();
        }
        let fill = if self.signed && bytes.last().unwrap() & 0x80 != 0 {
            255
        } else {
            0
        };
        bytes.resize(32, fill);
        core::array::from_fn(|index| {
            u64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap())
        })
    }
}

struct ShortRead {
    bytes: Vec<u8>,
    maximum: usize,
}
impl Read for ShortRead {
    fn read(&self, out: &mut [u8], offset: usize) -> Result<usize, Error> {
        let count = self
            .bytes
            .len()
            .saturating_sub(offset)
            .min(out.len())
            .min(self.maximum);
        if count > 0 {
            out[..count].copy_from_slice(&self.bytes[offset..offset + count]);
        }
        Ok(count)
    }
}
fn cursor(bytes: &[u8], size: usize, maximum: usize) -> Cursor {
    Cursor::new(
        size,
        Box::new(ShortRead {
            bytes: bytes.to_vec(),
            maximum,
        }),
    )
}

extern "C" {
    fn check_c(
        tag: u32,
        data: *const u8,
        len: u32,
        value: *const u64,
        out: *mut u8,
        capacity: u32,
    ) -> i32;
    fn narrow_i8_c(value: i64, out: *mut u8) -> i32;
    fn narrow_u8_c(value: u64, out: *mut u8) -> i32;
    fn check_c_imports_and_case_fields() -> i32;
}

macro_rules! check_scalar {
    ($case:ident, $entity:ident, $reader:ident, $list:ident, $from:ident, $to:ident, $value:ident, $tag:expr, $width:expr) => {{
        let list = packed::$list::try_from_values(&[$value]).unwrap();
        assert_eq!(&list.as_slice()[..4], 1u32.to_le_bytes());
        assert_eq!(&list.as_slice()[4..], $case.bytes);
        assert!(list.as_reader().try_values().unwrap().eq([$value]));
        let entity = packed::$entity::$from($value);
        assert_eq!(entity.as_slice(), $case.bytes);
        assert_eq!(entity.$to().unwrap(), $value);
        let old = legacy::$entity::from_slice(&$case.bytes).unwrap();
        assert_eq!(old.as_slice(), entity.as_slice());
        for offset in 0..8 {
            let mut input = vec![0xa5; offset];
            input.extend_from_slice(&$case.bytes);
            let reader = packed::$reader::new_unchecked(&input[offset..]);
            assert_eq!(reader.$to().unwrap(), $value);
            assert_eq!(
                reader.try_as_wire_bytes().unwrap().as_ptr(),
                input[offset..].as_ptr()
            );
            if !cfg!(miri) {
                let mut output = [0xa5; $width + 1];
                let limbs = $case.limbs();
                assert_eq!(
                    unsafe {
                        check_c(
                            $tag,
                            reader.as_slice().as_ptr(),
                            $width,
                            limbs.as_ptr(),
                            output.as_mut_ptr(),
                            $width,
                        )
                    },
                    0
                );
                assert_eq!(&output[..$width as usize], $case.bytes);
                assert_eq!(output[$width as usize], 0xa5);
                for wrong in [$width - 1, $width + 1] {
                    output.fill(0xa5);
                    assert_ne!(
                        unsafe {
                            check_c(
                                $tag,
                                reader.as_slice().as_ptr(),
                                $width,
                                limbs.as_ptr(),
                                output.as_mut_ptr(),
                                wrong,
                            )
                        },
                        0
                    );
                    assert_eq!(output, [0xa5; $width + 1]);
                    assert_ne!(
                        unsafe {
                            check_c(
                                $tag,
                                reader.as_slice().as_ptr(),
                                wrong,
                                limbs.as_ptr(),
                                output.as_mut_ptr(),
                                $width,
                            )
                        },
                        0
                    );
                }
            }
        }
        for maximum in [1, 3] {
            assert_eq!(
                lazy::$entity::from(cursor(&$case.bytes, $width as usize, maximum))
                    .$to()
                    .unwrap(),
                $value
            );
            assert!(lazy::$entity::from(cursor(
                &$case.bytes[..$width as usize - 1],
                $width as usize,
                maximum
            ))
            .$to()
            .is_err());
        }
        for wrong in [$width as usize - 1, $width as usize + 1] {
            let bytes = vec![0; wrong];
            assert!(matches!(
                packed::$reader::new_unchecked(&bytes).$to(),
                Err(molecule::integer::IntegerError::Size {
                    expected: $width,
                    actual
                }) if actual == wrong
            ));
            assert!(matches!(
                lazy::$entity::from(cursor(&bytes, wrong, 1)).$to(),
                Err(molecule::integer::IntegerError::Size {
                    expected: $width,
                    actual
                }) if actual == wrong
            ));
        }
    }};
}
include!(concat!(env!("OUT_DIR"), "/cases.rs"));

#[test]
fn fixed_vectors_match_rust_c_lazy_and_published_wire_api() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/vectors.json")).unwrap();
    let cases = cases.as_array().unwrap();
    assert_eq!(cases.len(), 486);
    for case in cases {
        check(&Case {
            bits: case["bits"].as_u64().unwrap() as u16,
            signed: case["signed"].as_bool().unwrap(),
            order: case["byte_order"].as_str().unwrap().into(),
            value: case["value"].as_str().unwrap().into(),
            bytes: hex(case["hex"].as_str().unwrap()),
        });
    }
}

#[test]
fn unsigned_256_has_numeric_order_and_checked_narrowing() {
    assert!(Uint256::from_u64(255) < Uint256::from_u64(256));
    assert_eq!(
        Uint256::from_u128(u128::MAX).try_to_u128().unwrap(),
        u128::MAX
    );
    assert_eq!(Uint256::from_u64(u64::MAX).try_to_u64().unwrap(), u64::MAX);
    assert!(Uint256::from_u128(u128::MAX).try_to_u64().is_err());
    assert!(Uint256::from_be_bytes([255; 32]).try_to_u128().is_err());
    assert_eq!(
        decimal_u256("340282366920938463463374607431768211455"),
        Uint256::from_u128(u128::MAX)
    );
}

#[test]
fn signed_and_unsigned_c_range_errors_leave_output_untouched() {
    if cfg!(miri) {
        return;
    }
    for (signed, unsigned) in [
        (128, 256),
        (-129, 256),
        (i64::MIN, u64::MAX),
        (i64::MAX, u64::MAX),
    ] {
        let mut output = 0xa5;
        assert_ne!(unsafe { narrow_i8_c(signed, &mut output) }, 0);
        assert_eq!(output, 0xa5);
        assert_ne!(unsafe { narrow_u8_c(unsigned, &mut output) }, 0);
        assert_eq!(output, 0xa5);
    }
    assert_eq!(unsafe { check_c_imports_and_case_fields() }, 0);
}

#[test]
fn lazy_field_values_follow_metadata_and_keep_opaque_fields_opaque() {
    let pair = packed::Pair::new_builder()
        .little(packed::U64Le::from_u64(123))
        .big(packed::I64Be::from_i64(-456))
        .build();
    let invoice = packed::Invoice::new_builder()
        .amount(packed::U64Be::from_u64(0x12345678))
        .opaque(packed::Byte32::from([0xa5; 32]))
        .build();
    let amounts = packed::Amounts::try_from_values(&[10, 20, 30]).unwrap();
    for maximum in [1, 3] {
        let value = lazy::Pair::from(cursor(pair.as_slice(), pair.as_slice().len(), maximum));
        assert_eq!(value.little_value().unwrap(), 123);
        assert_eq!(value.big_value().unwrap(), -456);
        let value = lazy::Invoice::from(cursor(
            invoice.as_slice(),
            invoice.as_slice().len(),
            maximum,
        ));
        assert_eq!(value.amount_value().unwrap(), 0x12345678);
        let lazy_amounts = lazy::Amounts::from(cursor(
            amounts.as_slice(),
            amounts.as_slice().len(),
            maximum,
        ));
        assert_eq!(lazy_amounts.get_value(0).unwrap(), 10);
        assert_eq!(lazy_amounts.get_value(2).unwrap(), 30);
        assert!(lazy_amounts.get_value(3).is_err());
        let value = lazy::Pair::from(cursor(pair.as_slice(), pair.as_slice().len() + 1, maximum));
        assert!(value.little_value().is_err());
    }
}

#[test]
fn ckb_integer_methods_preserve_existing_wire_bytes() {
    for (number, expected) in [
        (0u64, [0; 8]),
        (0x0807060504030201, [1, 2, 3, 4, 5, 6, 7, 8]),
        (u64::MAX, [255; 8]),
    ] {
        let current = blockchain::Uint64::from_u64(number);
        let old = old_blockchain::Uint64::from(expected);
        assert_eq!(current.as_slice(), old.as_slice());
    }
    let current = blockchain::CellOutput::new_builder()
        .capacity(blockchain::Uint64::from_u64(12345678))
        .build();
    let old = old_blockchain::CellOutput::new_builder()
        .capacity(old_blockchain::Uint64::from(12345678u64.to_le_bytes()))
        .build();
    assert_eq!(current.as_slice(), old.as_slice());
    let hash = |bytes: &[u8]| {
        let mut out = [0; 32];
        let mut state = blake2b_ref::Blake2bBuilder::new(32)
            .personal(b"ckb-default-hash")
            .build();
        state.update(bytes);
        state.finalize(&mut out);
        out
    };
    assert_eq!(hash(current.as_slice()), hash(old.as_slice()));
    // Default RawTransaction bytes and CKB hash captured with Molecule e289575.
    let transaction = blockchain::RawTransaction::default();
    assert_eq!(
        transaction.as_slice(),
        hex(concat!(
            "340000001c0000002000000024000000280000002c00000030000000",
            "000000000000000000000000000000000400000004000000",
        ))
    );
    assert_eq!(
        hash(transaction.as_slice()).as_slice(),
        hex("f2e7a5362e217ed4d7f985af71b654cafadd0c3b6d6a6c13f1b13a6bfd0d3d14")
    );
}
