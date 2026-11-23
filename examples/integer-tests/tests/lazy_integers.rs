use molecule::lazy_reader::Cursor;

#[allow(clippy::all)]
pub mod packed_schema {
    include!(concat!(env!("OUT_DIR"), "/packed/lazy_names.rs"));
}

#[allow(clippy::all)]
pub mod schema {
    include!(concat!(env!("OUT_DIR"), "/lazy/lazy_names.rs"));
}

#[test]
fn conventional_numeric_getters_use_the_declared_order_in_every_container() {
    let value = 256u64.to_be_bytes();
    let fixed = schema::Fixed::from(Cursor::from(value));
    assert_eq!(fixed.amount().unwrap(), 256);
    assert_eq!(fixed.amount_value().unwrap(), 256);

    let pair = schema::Pair::from(Cursor::from([value, value].concat()));
    assert_eq!(pair.get(1).unwrap(), 256);
    let values = [2u32.to_le_bytes().as_slice(), &value, &value].concat();
    assert!(schema::Values::from(Cursor::from(values))
        .into_iter()
        .eq([256, 256]));

    for port in [None, Some(value)] {
        let end = 20 + port.map_or(0, |_| 8);
        let mut wire = [end as u32, 12, 20].map(u32::to_le_bytes).concat();
        wire.extend_from_slice(&value);
        if let Some(bytes) = port {
            wire.extend_from_slice(&bytes);
        }
        let invoice = schema::Invoice::from(Cursor::from(wire));
        assert_eq!(invoice.amount().unwrap(), 256);
        assert_eq!(invoice.amount_value().unwrap(), 256);
        assert_eq!(invoice.port().unwrap(), port.map(|_| Some(256)));
    }
}

#[test]
fn custom_named_integer_containers_provide_checked_lazy_access() {
    let a = 256u64.to_be_bytes();
    let b = 512u64.to_be_bytes();

    let pair = schema::AmountPair::from(Cursor::from([a, b].concat()));
    assert_eq!(pair.get_value(0).unwrap(), 256);
    assert_eq!(pair.get_value(1).unwrap(), 512);
    assert!(pair.get_value(2).is_err());

    let vec_wire = [2u32.to_le_bytes().as_slice(), &a, &b].concat();
    let amounts = schema::Amounts::from(Cursor::from(vec_wire));
    assert_eq!(amounts.get_value(0).unwrap(), 256);
    assert_eq!(amounts.get_value(1).unwrap(), 512);
    assert!(amounts.get_value(2).is_err());

    assert_eq!(
        schema::AmountOpt::from(Cursor::from(Vec::new()))
            .try_value()
            .unwrap(),
        None
    );
    assert_eq!(
        schema::AmountOpt::from(Cursor::from(a))
            .try_value()
            .unwrap(),
        Some(256)
    );
    assert!(matches!(
        schema::AmountOpt::from(Cursor::from([1u8, 2, 3])).try_value(),
        Err(molecule::integer::IntegerError::Size {
            expected: 8,
            actual: 3
        })
    ));

    for tip in [None, Some(b)] {
        let end = 20 + tip.map_or(0, |_| 8);
        let mut wire = [end as u32, 12, 20].map(u32::to_le_bytes).concat();
        wire.extend_from_slice(&a);
        if let Some(bytes) = tip {
            wire.extend_from_slice(&bytes);
        }
        let order = schema::Order::from(Cursor::from(wire));
        assert_eq!(order.amount_value().unwrap(), 256);
        assert_eq!(order.tip_value().unwrap(), tip.map(|_| 512));
    }

    // Ordinary Rust bindings also compile and round-trip when a schema defines `table Option {}`.
    assert_eq!(
        packed_schema::AmountOpt::from_value(Some(512))
            .try_value()
            .unwrap(),
        Some(512)
    );
    assert_eq!(
        packed_schema::AmountOpt::from_value(None)
            .try_value()
            .unwrap(),
        None
    );
}
