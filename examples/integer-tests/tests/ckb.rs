use molecule::prelude::*;
use molecule_integer_tests::{blockchain, extensions, protocols};

#[test]
fn ckb_schema_consumers_keep_names_and_wire_values() {
    assert_eq!(
        blockchain::Uint32::from_u32(0x01020304).as_slice(),
        [4, 3, 2, 1]
    );
    assert_eq!(
        blockchain::Uint64::from_u64((1 << 53) + 1)
            .try_to_u64()
            .unwrap(),
        (1 << 53) + 1
    );
    assert_eq!(
        blockchain::Uint128::from_u128(u128::MAX)
            .try_to_u128()
            .unwrap(),
        u128::MAX
    );
    assert_eq!(
        blockchain::Uint256::from([0x81; 32]).as_slice(),
        &[0x81; 32]
    );
    assert_eq!(
        extensions::BeUint64::from_u64(256).as_slice(),
        [0, 0, 0, 0, 0, 0, 1, 0]
    );
    let key = |n| {
        extensions::TransactionKey::new_builder()
            .index(extensions::BeUint32::from_u32(n))
            .build()
    };
    assert!(key(255).as_slice() < key(256).as_slice());
    assert_eq!(key(256).index().as_slice(), [0, 0, 1, 0]);
    assert_eq!(protocols::Uint16::from_u16(0x1234).as_slice(), [0x34, 0x12]);
}

#[test]
fn discovery_port_distinguishes_absence_zero_and_invalid_input() {
    assert_eq!(protocols::PortOpt::from_value(None).as_slice(), &[][..]);
    assert_eq!(
        protocols::PortOpt::from_value(Some(0)).as_slice(),
        &[0, 0][..]
    );
    assert_eq!(
        protocols::PortOpt::from_value(Some(u16::MAX)).as_slice(),
        &[255, 255][..]
    );
    assert_eq!(
        protocols::PortOpt::from_value(Some(8115)).as_slice(),
        protocols::PortOpt::new_builder()
            .set(Some(protocols::Uint16::from_u16(8115)))
            .build()
            .as_slice()
    );
    for (wire, expected) in [
        (&[][..], None),
        (&[0, 0], Some(0)),
        (&[255, 255], Some(u16::MAX)),
    ] {
        assert_eq!(
            protocols::PortOptReader::from_slice(wire)
                .unwrap()
                .try_value()
                .unwrap(),
            expected
        );
        assert_eq!(
            protocols::PortOpt::from_slice(wire)
                .unwrap()
                .try_value()
                .unwrap(),
            expected
        );
    }
    for wire in [&[0][..], &[0, 0, 0]] {
        assert!(matches!(
            protocols::PortOptReader::new_unchecked(wire).try_value(),
            Err(molecule::integer::IntegerError::Size { expected: 2, .. })
        ));
    }
}

#[test]
fn repeated_fees_decode_in_order_after_checking_the_count() {
    let values = [u64::MAX, 0, (1 << 53) + 1];
    let legacy = extensions::Uint64Vec::new_builder()
        .extend(values.map(blockchain::Uint64::from_u64))
        .build();
    let packed = extensions::Uint64Vec::try_from_values(&values).unwrap();
    let records = values.map(|fee| (fee, String::from("fee")));
    let mut visits = 0;
    let projected = extensions::Uint64Vec::try_from_values_by(&records, |(fee, _)| {
        visits += 1;
        *fee
    })
    .unwrap();
    assert_eq!(visits, values.len());
    assert_eq!(projected.as_slice(), packed.as_slice());
    assert_eq!(packed.as_slice(), legacy.as_slice());
    assert_eq!(
        extensions::Uint64Vec::try_from_values(&[])
            .unwrap()
            .as_slice(),
        [0; 4]
    );
    let mut fees = packed.as_reader().try_values().unwrap();
    assert_eq!(fees.len(), values.len());
    assert_eq!(fees.next(), Some(values[0]));
    assert!(fees.eq(values[1..].iter().copied()));
    assert!(packed
        .try_values()
        .unwrap()
        .rev()
        .eq(values.iter().rev().copied()));
    assert_eq!(
        extensions::Uint64VecReader::new_unchecked(&[0; 4])
            .try_values()
            .unwrap()
            .len(),
        0
    );
    for wire in [&[][..], &[1, 0, 0, 0], &[0; 5], &[255; 4]] {
        assert!(matches!(
            extensions::Uint64VecReader::new_unchecked(wire).try_values(),
            Err(molecule::integer::IntegerError::Verification(_))
        ));
    }
}
