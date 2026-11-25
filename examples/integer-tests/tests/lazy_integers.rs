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

#[test]
fn lazy_table_verify_rejects_short_integer_fields_and_headerless_fixvecs() {
    // 1. Table with 1-byte Uint64 field (instead of 8 bytes) must fail verify(false) and verify(true).
    let truncated_invoice = [13u32, 12, 13]
        .map(u32::to_le_bytes)
        .concat()
        .into_iter()
        .chain([0x01u8])
        .collect::<Vec<_>>();
    let invoice = schema::Invoice::from(Cursor::from(truncated_invoice));
    assert!(invoice.verify(false).is_err());
    assert!(invoice.verify(true).is_err());

    // 2. Table with 1-byte optional Uint64 (invalid non-empty payload) must fail verify.
    let bad_opt_invoice = [21u32, 12, 20]
        .map(u32::to_le_bytes)
        .concat()
        .into_iter()
        .chain(256u64.to_be_bytes())
        .chain([0x42u8])
        .collect::<Vec<_>>();
    let invoice = schema::Invoice::from(Cursor::from(bad_opt_invoice));
    assert!(invoice.verify(false).is_err());

    // 3. Table with headerless 0-byte FixVec<Amount> must fail verify(false).
    let headerless_vec_table = [8u32, 8].map(u32::to_le_bytes).concat();
    let holder = schema::VecHolder::from(Cursor::from(headerless_vec_table));
    assert!(holder.verify(false).is_err());

    // Valid empty FixVec<Amount> (4-byte zero count header) passes verify(false).
    let valid_vec_table = [12u32, 8, 0].map(u32::to_le_bytes).concat();
    let holder = schema::VecHolder::from(Cursor::from(valid_vec_table));
    holder.verify(false).unwrap();
}

#[allow(clippy::all, dead_code)]
mod reg_packed {
    pub mod common {
        pub mod defs {
            include!(concat!(env!("OUT_DIR"), "/reg_packed/common/defs.rs"));
        }
    }
    pub mod leaf {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/leaf.rs"));
    }
    pub mod mid {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/mid.rs"));
    }
    pub mod mod_a {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/mod_a.rs"));
    }
    pub mod mod_b {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/mod_b.rs"));
    }
    pub mod imp_row {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/imp_row.rs"));
    }
    pub mod consumer {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/consumer.rs"));
    }
}

#[allow(clippy::all, dead_code)]
mod reg_lazy {
    pub mod common {
        pub mod defs {
            include!(concat!(env!("OUT_DIR"), "/reg_lazy/common/defs.rs"));
        }
    }
    pub mod leaf {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/leaf.rs"));
    }
    pub mod mid {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/mid.rs"));
    }
    pub mod mod_a {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/mod_a.rs"));
    }
    pub mod mod_b {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/mod_b.rs"));
    }
    pub mod imp_row {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/imp_row.rs"));
    }
    pub mod consumer {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/consumer.rs"));
    }
}

#[test]
fn multi_module_imports_shadowing_and_case_variant_fields_compile_and_agree() {
    use molecule::prelude::{Builder as _, Entity as _};

    let imp_row = reg_packed::imp_row::FeeRow::new_builder()
        .v(reg_packed::imp_row::RowLe::from_u16(1))
        .build();
    let local_row = reg_packed::consumer::FeeRow::new_builder()
        .v(reg_packed::consumer::RowBe::from_u16(256))
        .build();
    // Both rows have identical wire bytes `[0x01, 0x00]`, but opposite endianness!
    assert_eq!(imp_row.as_slice(), &[0x01, 0x00]);
    assert_eq!(local_row.as_slice(), &[0x01, 0x00]);

    let combined = reg_packed::consumer::Combined::new_builder()
        .def(reg_packed::common::defs::DefVal::from_u32(0x1122_3344))
        .leaf(reg_packed::leaf::LeafAmt::from_u64(0x0102_0304_0506_0708))
        .rate(reg_packed::mod_a::FeeRate::from_u32(77))
        .opt_row(
            reg_packed::imp_row::OptFeeRow::new_builder()
                .set(Some(imp_row))
                .build(),
        )
        .local_row(local_row)
        .cases(
            reg_packed::consumer::CaseFields::new_builder()
                .fee_rate(4u8)
                .feerate(9u8)
                .build(),
        )
        .build();

    // Ordinary Rust checks
    let verified_packed = reg_packed::consumer::Combined::from_slice(combined.as_slice()).unwrap();
    assert_eq!(verified_packed.def().try_to_u32().unwrap(), 0x1122_3344);
    assert_eq!(
        verified_packed.leaf().try_to_u64().unwrap(),
        0x0102_0304_0506_0708
    );
    assert_eq!(verified_packed.rate().try_to_u32().unwrap(), 77);
    assert_eq!(
        verified_packed
            .opt_row()
            .to_opt()
            .unwrap()
            .v()
            .try_to_u16()
            .unwrap(),
        1
    );
    assert_eq!(verified_packed.local_row().v().try_to_u16().unwrap(), 256);
    assert_eq!(u8::from(verified_packed.cases().fee_rate()), 4);
    assert_eq!(u8::from(verified_packed.cases().feerate()), 9);

    // Lazy Rust checks: must agree 100% (especially `opt_row` reading 1, NOT 256!)
    let lazy_combined =
        reg_lazy::consumer::Combined::from(Cursor::from(combined.as_slice().to_vec()));
    lazy_combined.verify(false).unwrap();
    assert_eq!(lazy_combined.def_value().unwrap(), 0x1122_3344);
    assert_eq!(lazy_combined.leaf_value().unwrap(), 0x0102_0304_0506_0708);
    assert_eq!(lazy_combined.rate_value().unwrap(), 77);
    assert_eq!(
        lazy_combined.opt_row().unwrap().unwrap().v_value().unwrap(),
        1
    );
    assert_eq!(lazy_combined.local_row().unwrap().v_value().unwrap(), 256);
    assert_eq!(lazy_combined.cases().unwrap().fee_rate().unwrap(), 4);
    assert_eq!(lazy_combined.cases().unwrap().feerate().unwrap(), 9);

    // Shadowing types (`Ok`, `Err`, `Option`, `Result`, `Cursor`, `Error`, `Vec`, `From`) and
    // shadowing field names (`as_slice`, `build`) work in both ordinary and lazy Rust.
    let shadow_s = reg_packed::consumer::ShadowS::new_builder()
        .as_slice(11u8)
        .build(22u8);
    let shadow_s =
        <reg_packed::consumer::ShadowSBuilder as molecule::prelude::Builder>::build(&shadow_s);
    assert_eq!(u8::from(shadow_s.as_slice()), 11);
    assert_eq!(u8::from(shadow_s.build()), 22);
    assert_eq!(
        <reg_packed::consumer::ShadowS as molecule::prelude::Entity>::as_slice(&shadow_s),
        &[11, 22]
    );

    let shadow_t = reg_packed::consumer::ShadowT::new_builder()
        .as_slice(33u8)
        .build();
    let shadow_t_bytes =
        <reg_packed::consumer::ShadowT as molecule::prelude::Entity>::as_slice(&shadow_t);
    let shadow_t_roundtrip = reg_packed::consumer::ShadowT::from_slice(shadow_t_bytes).unwrap();
    assert_eq!(u8::from(shadow_t_roundtrip.as_slice()), 33);
    assert_eq!(u8::from(shadow_t_roundtrip.as_reader().as_slice()), 33);

    let lazy_shadow_t = reg_lazy::consumer::ShadowT::from(Cursor::from(shadow_t_bytes.to_vec()));
    lazy_shadow_t.verify(false).unwrap();
    assert_eq!(lazy_shadow_t.as_slice().unwrap(), 33);
}

#[test]
fn colliding_rust_names_preserve_wire_types_and_field_access() {
    use molecule::prelude::{Builder as _, Entity as _};
    use reg_packed::consumer::{Amounts, Rates, RowsBuilder, Value, Values, T};
    let rows = RowsBuilder::default().build();
    assert_eq!(rows.as_slice(), &[0; 4]);
    let amounts: Amounts = [T::from_u16(0x1234)].into_iter().collect();
    assert_eq!(amounts.get(0).unwrap().as_slice(), &[0x34, 0x12]);
    let bytes = [0x01, 0x02, 0x03, 0x04];
    let wire = [bytes, bytes].concat();
    let rates = Rates::from_slice(&wire).unwrap();
    let lazy = reg_lazy::consumer::Rates::from(Cursor::from(wire));
    assert_eq!(
        rates.little().try_to_u32().unwrap(),
        u32::from_le_bytes(bytes)
    );
    assert_eq!(rates.big().try_to_u32().unwrap(), u32::from_be_bytes(bytes));
    assert_eq!(
        lazy.little_value().unwrap(),
        rates.little().try_to_u32().unwrap()
    );
    assert_eq!(lazy.big_value().unwrap(), rates.big().try_to_u32().unwrap());

    let value = Value::new_builder().as_builder(11u8).into(22u8).build();
    let rebuilt = molecule::prelude::Entity::as_builder(value).build();
    assert_eq!(
        Values::new_builder()
            .push(rebuilt)
            .build()
            .get(0)
            .unwrap()
            .as_slice(),
        &[11, 22]
    );
}

#[test]
fn lazy_union_decodes_and_verifies_nested_payloads() {
    use reg_lazy::consumer::{Inner, Payload};
    let decode = |id: u32, bytes: &[u8]| {
        Payload::try_from(Cursor::from([id.to_le_bytes().as_slice(), bytes].concat()))
    };
    assert!(matches!(decode(0, &[]).unwrap(), Payload::OptByte(None)));
    assert!(matches!(
        decode(0, &[42]).unwrap(),
        Payload::OptByte(Some(42))
    ));
    assert!(decode(0, &[42, 0]).is_err());
    let raw = [2, 0, 0, 0, 3, 4];
    let Payload::OptBytes(Some(bytes)) = decode(1, &raw).unwrap() else {
        panic!("wrong variant")
    };
    assert_eq!(Vec::<u8>::try_from(bytes).unwrap(), [3, 4]);
    assert!(decode(1, &[0, 0, 0, 0, 9]).is_err());
    assert!(matches!(
        decode(2, &[1, 0, 0, 0, 7]).unwrap(),
        Payload::Inner(Inner::Byte(7))
    ));
    assert!(matches!(
        decode(3, &[1, 0, 0, 0, 7]).unwrap(),
        Payload::OptInner(Some(Inner::Byte(7)))
    ));
    assert!(decode(3, &[99, 0, 0, 0]).is_err());
    // A table with an extra field is accepted only by compatible verification.
    let extra = [0, 0, 0, 0, 8, 0, 0, 0, 8, 0, 0, 0];
    let nested = decode(3, &extra).unwrap();
    assert!(nested.verify(false).is_err());
    nested.verify(true).unwrap();
    let Payload::NestedRate(Some(Some(rate))) = decode(4, &[1, 0, 0, 0]).unwrap() else {
        panic!("wrong variant")
    };
    assert_eq!(rate.try_to_u32().unwrap(), 1);
    assert!(decode(4, &[1, 0]).unwrap().verify(false).is_err());
}

struct ShortSource {
    bytes: Vec<u8>,
    chunk: usize,
}

impl molecule::lazy_reader::Read for ShortSource {
    fn read(
        &self,
        output: &mut [u8],
        offset: usize,
    ) -> Result<usize, molecule::lazy_reader::Error> {
        let remaining = self
            .bytes
            .get(offset..)
            .ok_or(molecule::lazy_reader::Error::OutOfBound(
                offset,
                self.bytes.len(),
            ))?;
        let count = remaining.len().min(output.len()).min(self.chunk);
        output[..count].copy_from_slice(&remaining[..count]);
        Ok(count)
    }
}

fn short_cursor(bytes: Vec<u8>, chunk: usize) -> Cursor {
    Cursor::new(bytes.len(), Box::new(ShortSource { bytes, chunk }))
}

#[test]
fn annotated_lazy_integer_getters_retry_short_reads_in_every_container() {
    let a = 0x0102_0304_0506_0708u64.to_be_bytes();
    let b = 0x1122_3344_5566_7788u64.to_be_bytes();

    for chunk in [1usize, 3] {
        // FixVec of annotated Uint64: get(), len(), iter(), into_iter(), verify(), get_value()
        let vec_wire = [2u32.to_le_bytes().as_slice(), &a, &b].concat();
        let values = schema::Values::from(short_cursor(vec_wire.clone(), chunk));
        values.verify(false).unwrap();
        assert_eq!(values.len().unwrap(), 2);
        assert_eq!(values.get(0).unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(values.get(1).unwrap(), 0x1122_3344_5566_7788);
        assert_eq!(values.get_value(0).unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(values.get_value(1).unwrap(), 0x1122_3344_5566_7788);
        assert!(values.get(2).is_err());
        assert_eq!(
            values.iter().collect::<Vec<_>>(),
            vec![0x0102_0304_0506_0708, 0x1122_3344_5566_7788]
        );
        assert_eq!(
            values.into_iter().collect::<Vec<_>>(),
            vec![0x0102_0304_0506_0708, 0x1122_3344_5566_7788]
        );

        let empty_values = schema::Values::from(short_cursor(0u32.to_le_bytes().to_vec(), chunk));
        empty_values.verify(false).unwrap();
        assert_eq!(empty_values.len().unwrap(), 0);
        assert!(empty_values.get(0).is_err());
        assert!(empty_values.into_iter().next().is_none());

        let truncated_values =
            schema::Values::from(short_cursor(vec_wire[..vec_wire.len() - 1].to_vec(), chunk));
        assert!(truncated_values.verify(false).is_err());
        assert!(truncated_values.len().is_err());
        assert!(truncated_values.get(0).is_err());

        // Array of annotated Uint64
        let pair = schema::Pair::from(short_cursor([a, b].concat(), chunk));
        assert_eq!(pair.get(0).unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(pair.get(1).unwrap(), 0x1122_3344_5566_7788);
        assert_eq!(pair.get_value(1).unwrap(), 0x1122_3344_5566_7788);
        assert!(pair.get(2).is_err());
        assert!(pair.get(usize::MAX).is_err());

        // Struct with annotated Uint64
        let fixed = schema::Fixed::from(short_cursor(a.to_vec(), chunk));
        assert_eq!(fixed.amount().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(fixed.amount_value().unwrap(), 0x0102_0304_0506_0708);

        // Table with annotated Uint64 and nested optional Uint64 (including extra compatible field)
        for port in [None, Some(b)] {
            let end = 20 + port.map_or(0, |_| 8);
            let mut wire = [end as u32, 12, 20].map(u32::to_le_bytes).concat();
            wire.extend_from_slice(&a);
            if let Some(bytes) = port {
                wire.extend_from_slice(&bytes);
            }
            let invoice = schema::Invoice::from(short_cursor(wire, chunk));
            assert_eq!(invoice.amount().unwrap(), 0x0102_0304_0506_0708);
            assert_eq!(invoice.amount_value().unwrap(), 0x0102_0304_0506_0708);
            assert_eq!(
                invoice.port().unwrap(),
                port.map(|_| Some(0x1122_3344_5566_7788))
            );
        }

        let mut compat_wire = [34u32, 16, 24, 32].map(u32::to_le_bytes).concat();
        compat_wire.extend_from_slice(&a);
        compat_wire.extend_from_slice(&b);
        compat_wire.extend_from_slice(&[0xaa, 0xbb]);
        let compat_invoice = schema::Invoice::from(short_cursor(compat_wire, chunk));
        assert_eq!(compat_invoice.amount().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(
            compat_invoice.port().unwrap(),
            Some(Some(0x1122_3344_5566_7788))
        );

        // DynVec of optional annotated Uint64: [None, Some(b)]
        let mut dyn_wire = [20u32, 12, 12].map(u32::to_le_bytes).concat();
        dyn_wire.extend_from_slice(&b);
        let maybe_list = schema::MaybeList::from(short_cursor(dyn_wire, chunk));
        assert_eq!(maybe_list.len().unwrap(), 2);
        assert_eq!(maybe_list.get(0).unwrap(), None);
        assert_eq!(maybe_list.get(1).unwrap(), Some(0x1122_3344_5566_7788));
        assert!(maybe_list.get(2).is_err());
        assert_eq!(
            maybe_list.into_iter().collect::<Vec<_>>(),
            vec![None, Some(0x1122_3344_5566_7788)]
        );
    }

    // Zero-progress source returns an error instead of looping forever.
    let stalled = schema::Values::from(short_cursor(
        [1u32.to_le_bytes().as_slice(), &a].concat(),
        0,
    ));
    assert!(matches!(
        stalled.get(0),
        Err(molecule::lazy_reader::Error::Read(0, 4))
    ));
}
