use molecule::lazy_reader::Cursor;

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
