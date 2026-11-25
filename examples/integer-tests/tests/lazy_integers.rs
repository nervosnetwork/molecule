use molecule::lazy_reader::Cursor;

#[allow(clippy::all)]
pub mod packed_schema {
    include!(concat!(env!("OUT_DIR"), "/packed/lazy_names.rs"));
}

#[allow(clippy::all)]
pub mod schema {
    include!(concat!(env!("OUT_DIR"), "/lazy/lazy_names.rs"));
}

use molecule_integer_tests::lazy_ckb::blockchain as lazy_blockchain;

#[allow(clippy::all)]
pub mod packed_blockchain {
    include!(concat!(env!("OUT_DIR"), "/packed/blockchain.rs"));
}

#[allow(clippy::all)]
pub mod cross_mod_packed {
    pub mod plain_leaf {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/plain_leaf.rs"));
    }
    pub mod plain_child {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/plain_child.rs"));
    }
    pub mod plain_root {
        include!(concat!(env!("OUT_DIR"), "/reg_packed/plain_root.rs"));
    }
}

#[allow(clippy::all)]
pub mod cross_mod_lazy {
    pub mod plain_leaf {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/plain_leaf.rs"));
    }
    pub mod plain_child {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/plain_child.rs"));
    }
    pub mod plain_root {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/plain_root.rs"));
    }
}

#[allow(clippy::all, dead_code)]
mod private_union_case {
    pub mod pkg {
        // Deliberately private (not `pub mod`) so any reference to `pkg::internal`
        // from sibling `pkg_root` fails to compile with E0603.
        mod internal {
            include!(concat!(env!("OUT_DIR"), "/reg_lazy/pkg/internal.rs"));
        }
        pub mod outer {
            include!(concat!(env!("OUT_DIR"), "/reg_lazy/pkg/outer.rs"));
        }
    }
    pub mod pkg_root {
        include!(concat!(env!("OUT_DIR"), "/reg_lazy/pkg_root.rs"));
    }
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
        let remaining =
            self.bytes
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

        // Multi-field Struct with annotated Uint64 at non-zero offset + exact total-size checks
        let fixed = schema::Fixed::from(short_cursor(a.to_vec(), chunk));
        fixed.verify(false).unwrap();
        assert_eq!(fixed.amount().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(fixed.amount_value().unwrap(), 0x0102_0304_0506_0708);
        assert!(schema::Fixed::from(short_cursor(a[..7].to_vec(), chunk))
            .amount()
            .is_err());
        assert!(
            schema::Fixed::from(short_cursor([a.as_slice(), &[0]].concat(), chunk))
                .amount()
                .is_err()
        );

        let two = schema::TwoAmounts::from(short_cursor([a, b].concat(), chunk));
        two.verify(false).unwrap();
        assert_eq!(two.first().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(two.second().unwrap(), 0x1122_3344_5566_7788);
        assert_eq!(two.first_value().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(two.second_value().unwrap(), 0x1122_3344_5566_7788);
        assert!(schema::TwoAmounts::from(short_cursor(a.to_vec(), chunk))
            .first()
            .is_err());
        assert!(schema::TwoAmounts::from(short_cursor(
            [a.as_slice(), b.as_slice(), &[0]].concat(),
            chunk
        ))
        .first()
        .is_err());

        // Table with annotated Uint64 and nested optional Uint64 (including extra compatible field)
        for port in [None, Some(b)] {
            let end = 20 + port.map_or(0, |_| 8);
            let mut wire = [end as u32, 12, 20].map(u32::to_le_bytes).concat();
            wire.extend_from_slice(&a);
            if let Some(bytes) = port {
                wire.extend_from_slice(&bytes);
            }
            let invoice = schema::Invoice::from(short_cursor(wire, chunk));
            invoice.verify(false).unwrap();
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
        assert!(compat_invoice.verify(false).is_err());
        compat_invoice.verify(true).unwrap();
        assert_eq!(compat_invoice.amount().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(
            compat_invoice.port().unwrap(),
            Some(Some(0x1122_3344_5566_7788))
        );

        let one_field_wire = [16u32.to_le_bytes().as_slice(), &8u32.to_le_bytes(), &a].concat();
        let short_invoice = schema::Invoice::from(short_cursor(one_field_wire, chunk));
        assert!(matches!(
            short_invoice.verify(true),
            Err(molecule::lazy_reader::Error::FieldCount(1))
        ));
        assert!(matches!(
            short_invoice.amount(),
            Err(molecule::lazy_reader::Error::FieldCount(1))
        ));

        // DynVec of optional annotated Uint64: [None, Some(b)], empty, corrupt offsets, bad payload
        let mut dyn_wire = [20u32, 12, 12].map(u32::to_le_bytes).concat();
        dyn_wire.extend_from_slice(&b);
        let maybe_list = schema::MaybeList::from(short_cursor(dyn_wire, chunk));
        maybe_list.verify(false).unwrap();
        assert_eq!(maybe_list.len().unwrap(), 2);
        assert_eq!(maybe_list.get(0).unwrap(), None);
        assert_eq!(maybe_list.get(1).unwrap(), Some(0x1122_3344_5566_7788));
        assert!(maybe_list.get(2).is_err());
        assert_eq!(
            maybe_list.into_iter().collect::<Vec<_>>(),
            vec![None, Some(0x1122_3344_5566_7788)]
        );

        let empty_maybe = schema::MaybeList::from(short_cursor(4u32.to_le_bytes().to_vec(), chunk));
        empty_maybe.verify(false).unwrap();
        assert_eq!(empty_maybe.len().unwrap(), 0);
        assert!(empty_maybe.get(0).is_err());

        let corrupt_maybe = schema::MaybeList::from(short_cursor(
            [16u32, 12, 99, 0].map(u32::to_le_bytes).concat(),
            chunk,
        ));
        assert!(corrupt_maybe.verify(false).is_err());
        assert!(corrupt_maybe.get(0).is_err());

        let bad_payload_maybe = schema::MaybeList::from(short_cursor(
            [13u32, 12, 12]
                .map(u32::to_le_bytes)
                .concat()
                .into_iter()
                .chain([0x42])
                .collect(),
            chunk,
        ));
        assert!(bad_payload_maybe.verify(false).is_err());
        assert!(bad_payload_maybe.get(1).is_err());

        // Custom-named annotated integer containers (Amounts, Order, VecHolder) + Union
        let amounts = schema::Amounts::from(short_cursor(vec_wire.clone(), chunk));
        amounts.verify(false).unwrap();
        assert_eq!(amounts.len().unwrap(), 2);
        assert_eq!(amounts.get(0).unwrap(), a);
        assert_eq!(amounts.get_value(1).unwrap(), 0x1122_3344_5566_7788);

        let order_wire = [28u32, 12, 20]
            .map(u32::to_le_bytes)
            .concat()
            .into_iter()
            .chain(a)
            .chain(b)
            .collect::<Vec<_>>();
        let order = schema::Order::from(short_cursor(order_wire, chunk));
        order.verify(false).unwrap();
        assert_eq!(order.amount().unwrap(), a);
        assert_eq!(order.amount_value().unwrap(), 0x0102_0304_0506_0708);
        assert_eq!(order.tip().unwrap(), Some(b));
        assert_eq!(order.tip_value().unwrap(), Some(0x1122_3344_5566_7788));

        let holder_wire = [8u32 + vec_wire.len() as u32, 8]
            .map(u32::to_le_bytes)
            .concat()
            .into_iter()
            .chain(vec_wire.iter().copied())
            .collect::<Vec<_>>();
        let holder = schema::VecHolder::from(short_cursor(holder_wire, chunk));
        holder.verify(false).unwrap();
        assert_eq!(
            holder.amounts().unwrap().get_value(0).unwrap(),
            0x0102_0304_0506_0708
        );

        let union_wire = [
            0u32.to_le_bytes().as_slice(),
            &0x0102_0304_0506_0708u64.to_le_bytes(),
        ]
        .concat();
        let mixed =
            reg_lazy::consumer::MixedUnion::try_from(short_cursor(union_wire, chunk)).unwrap();
        mixed.verify(false).unwrap();
        let reg_lazy::consumer::MixedUnion::LeafAmt(leaf) = mixed else {
            panic!("wrong variant")
        };
        assert_eq!(leaf.try_to_u64().unwrap(), 0x0102_0304_0506_0708);

        // Mixed integer + non-integer child Table / Union / Bytes in an annotated schema
        let payload_bytes = [
            1u32.to_le_bytes().as_slice(),
            &2u32.to_le_bytes(),
            &[0x33, 0x44],
        ]
        .concat();
        let reg_lazy::consumer::Payload::OptBytes(Some(raw)) =
            reg_lazy::consumer::Payload::try_from(short_cursor(payload_bytes, chunk)).unwrap()
        else {
            panic!("wrong variant")
        };
        assert_eq!(raw.read_array::<2>().unwrap(), [0x33, 0x44]);

        let payload_inner = [
            3u32.to_le_bytes().as_slice(),
            &0u32.to_le_bytes(),
            &4u32.to_le_bytes(),
        ]
        .concat();
        let nested =
            reg_lazy::consumer::Payload::try_from(short_cursor(payload_inner, chunk)).unwrap();
        nested.verify(false).unwrap();

        // Empty table in an annotated schema (0 fields) verifies strictly and compatibly on short reads
        let empty_opt = schema::Option::from(short_cursor(4u32.to_le_bytes().to_vec(), chunk));
        empty_opt.verify(false).unwrap();
        let extra_opt = schema::Option::from(short_cursor(
            [8u32, 8].map(u32::to_le_bytes).concat(),
            chunk,
        ));
        assert!(extra_opt.verify(false).is_err());
        extra_opt.verify(true).unwrap();
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

    // Re-entrant mutable borrow fails gracefully without panicking.
    let borrowed = schema::Values::from(short_cursor(
        [1u32.to_le_bytes().as_slice(), &a].concat(),
        3,
    ));
    let _guard = borrowed.cursor.data_source.borrow_mut();
    assert!(matches!(
        borrowed.get(0),
        Err(molecule::lazy_reader::Error::Common)
    ));
}

struct CountingSource {
    bytes: Vec<u8>,
    chunk: usize,
    calls: std::rc::Rc<std::cell::Cell<usize>>,
}

impl molecule::lazy_reader::Read for CountingSource {
    fn read(
        &self,
        output: &mut [u8],
        offset: usize,
    ) -> Result<usize, molecule::lazy_reader::Error> {
        self.calls.set(self.calls.get() + 1);
        let remaining =
            self.bytes
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

fn counting_cursor(bytes: Vec<u8>, chunk: usize) -> (Cursor, std::rc::Rc<std::cell::Cell<usize>>) {
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let cur = Cursor::new(
        bytes.len(),
        Box::new(CountingSource {
            bytes,
            chunk,
            calls: calls.clone(),
        }),
    );
    (cur, calls)
}

#[test]
fn dynvec_and_fixvec_iteration_scale_linearly_in_read_calls() {
    use molecule::prelude::{Builder as _, Entity as _};

    let build_bytes_vec = |n: usize| -> Vec<u8> {
        let mut builder = packed_blockchain::BytesVec::new_builder();
        for i in 0..n {
            let payload = [(i & 0xff) as u8, ((i >> 8) & 0xff) as u8];
            let item = packed_blockchain::Bytes::new_builder()
                .extend(payload.into_iter().map(Into::into))
                .build();
            builder = builder.push(item);
        }
        builder.build().as_slice().to_vec()
    };

    let wire_1k = build_bytes_vec(1_024);
    let wire_2k = build_bytes_vec(2_048);

    // Borrowed iter() on CKB BytesVec: must scale linearly (not O(n^2)).
    let (cur_1k, calls_1k) = counting_cursor(wire_1k.clone(), usize::MAX);
    let vec_1k = lazy_blockchain::BytesVec::from(cur_1k);
    assert_eq!(vec_1k.iter().count(), 1_024);
    let iter_1k = calls_1k.get();

    let (cur_2k, calls_2k) = counting_cursor(wire_2k.clone(), usize::MAX);
    let vec_2k = lazy_blockchain::BytesVec::from(cur_2k);
    assert_eq!(vec_2k.iter().count(), 2_048);
    let iter_2k = calls_2k.get();

    assert!(
        iter_2k <= iter_1k * 2 + 64,
        "borrowed DynVec::iter() grew super-linearly: 1k={iter_1k}, 2k={iter_2k}"
    );
    assert!(
        iter_2k < 5_000,
        "borrowed DynVec::iter() read count unexpectedly high: {iter_2k}"
    );

    // Consuming into_iter() on CKB BytesVec: must also scale linearly.
    let (cur_1k, calls_1k) = counting_cursor(wire_1k.clone(), usize::MAX);
    assert_eq!(
        lazy_blockchain::BytesVec::from(cur_1k).into_iter().count(),
        1_024
    );
    let into_1k = calls_1k.get();

    let (cur_2k, calls_2k) = counting_cursor(wire_2k.clone(), usize::MAX);
    assert_eq!(
        lazy_blockchain::BytesVec::from(cur_2k).into_iter().count(),
        2_048
    );
    let into_2k = calls_2k.get();

    assert!(
        into_2k <= into_1k * 2 + 64,
        "consuming DynVec::into_iter() grew super-linearly: 1k={into_1k}, 2k={into_2k}"
    );
    assert_eq!(into_1k, iter_1k);
    assert_eq!(into_2k, iter_2k);

    // Manual indexed get(i) loop and verify(false) also scale linearly.
    let (cur_1k, calls_1k) = counting_cursor(wire_1k.clone(), usize::MAX);
    let v1 = lazy_blockchain::BytesVec::from(cur_1k);
    for i in 0..v1.len().unwrap() {
        assert_eq!(v1.get(i).unwrap().size, 2);
    }
    let get_1k = calls_1k.get();

    let (cur_2k, calls_2k) = counting_cursor(wire_2k.clone(), usize::MAX);
    let v2 = lazy_blockchain::BytesVec::from(cur_2k);
    for i in 0..v2.len().unwrap() {
        assert_eq!(v2.get(i).unwrap().size, 2);
    }
    let get_2k = calls_2k.get();
    assert!(
        get_2k <= get_1k * 2 + 64,
        "DynVec::get() loop grew super-linearly: 1k={get_1k}, 2k={get_2k}"
    );

    // Short reads (3-byte chunks) also succeed through BytesVec::iter() and verify().
    let small_wire = build_bytes_vec(12);
    let small = lazy_blockchain::BytesVec::from(short_cursor(small_wire, 3));
    small.verify(false).unwrap();
    let decoded: Vec<_> = small
        .iter()
        .map(|cur| cur.read_array::<2>().unwrap())
        .collect();
    assert_eq!(decoded.len(), 12);
    assert_eq!(decoded[0], [0, 0]);
    assert_eq!(decoded[11], [11, 0]);

    // FixVec iteration (Values) also scales linearly without header cache thrashing.
    let build_values = |n: usize| -> Vec<u8> {
        let mut wire = Vec::with_capacity(4 + n * 8);
        wire.extend_from_slice(&(n as u32).to_le_bytes());
        for i in 0..n {
            wire.extend_from_slice(&(i as u64).to_be_bytes());
        }
        wire
    };
    let (fcur_1k, fcalls_1k) = counting_cursor(build_values(1_024), usize::MAX);
    assert_eq!(schema::Values::from(fcur_1k).into_iter().count(), 1_024);
    let f1 = fcalls_1k.get();
    let (fcur_2k, fcalls_2k) = counting_cursor(build_values(2_048), usize::MAX);
    assert_eq!(schema::Values::from(fcur_2k).into_iter().count(), 2_048);
    let f2 = fcalls_2k.get();
    assert!(
        f2 <= f1 * 2 + 16,
        "FixVec::into_iter() grew super-linearly: 1k={f1}, 2k={f2}"
    );
    assert!(f2 <= 2_100, "FixVec::into_iter() thrashed cache: {f2}");
}

#[test]
fn cross_module_unannotated_child_verification_retries_short_reads() {
    use cross_mod_packed::{plain_child, plain_leaf, plain_root};
    use molecule::prelude::{Builder as _, Entity as _};

    let leaf = plain_leaf::PlainLeaf::new_builder()
        .tag(7u8)
        .data(
            plain_leaf::PlainBytes::new_builder()
                .extend([0x11u8, 0x22, 0x33].into_iter().map(Into::into))
                .build(),
        )
        .build();
    let list = plain_child::PlainLeafVec::new_builder()
        .push(leaf.clone())
        .build();
    let opt = plain_child::OptPlainLeaf::new_builder()
        .set(Some(leaf.clone()))
        .build();
    let plain_u = plain_child::PlainUnion::new_builder()
        .set(leaf.clone())
        .build();
    let child = plain_child::PlainChild::new_builder()
        .a(9u8)
        .leaf(leaf)
        .list(list.clone())
        .opt(opt.clone())
        .u(plain_u.clone())
        .build();
    let pair = plain_child::PlainPair::new_builder().a(1u8).b(2u8).build();
    let pair_vec = plain_child::PlainPairVec::new_builder().push(pair).build();
    let opt_pair_vec = plain_child::OptPlainPairVec::new_builder()
        .set(Some(pair_vec.clone()))
        .build();
    let opt_u = plain_child::OptPlainUnion::new_builder()
        .set(Some(plain_u.clone()))
        .build();
    let wrap = plain_root::LocalWrap::new_builder()
        .set(plain_u.clone())
        .build();
    let union_vec = plain_root::PlainUnionVec::new_builder()
        .push(plain_u.clone())
        .push(
            plain_child::PlainUnion::new_builder()
                .set(molecule::prelude::Byte::new(5))
                .build(),
        )
        .build();
    let root = plain_root::PlainRoot::new_builder()
        .amount(plain_root::RootAmt::from_u64(0xdead_beef_cafe_babe))
        .child(child.clone())
        .list(list.clone())
        .u(plain_u.clone())
        .opt(opt.clone())
        .wrap(wrap)
        .build();

    let root_bytes = root.as_slice().to_vec();
    for chunk in [1usize, 3, usize::MAX] {
        let lazy_root =
            cross_mod_lazy::plain_root::PlainRoot::from(short_cursor(root_bytes.clone(), chunk));
        lazy_root.verify(false).unwrap();
        lazy_root.verify(true).unwrap();
        assert_eq!(lazy_root.amount_value().unwrap(), 0xdead_beef_cafe_babe);
        // Reading fields through imported unannotated child readers also retries short reads.
        let lazy_child = lazy_root.child().unwrap();
        assert_eq!(lazy_child.a().unwrap(), 9);
        let lazy_leaf = lazy_child.leaf().unwrap();
        assert_eq!(lazy_leaf.tag().unwrap(), 7);
        assert_eq!(
            Vec::<u8>::try_from(lazy_leaf.data().unwrap()).unwrap(),
            vec![0x11, 0x22, 0x33]
        );
        let lazy_list = lazy_child.list().unwrap();
        assert_eq!(lazy_list.len().unwrap(), 1);
        assert_eq!(
            Vec::<u8>::try_from(lazy_list.get(0).unwrap().data().unwrap()).unwrap(),
            vec![0x11, 0x22, 0x33]
        );
        let lazy_opt_leaf = lazy_child.opt().unwrap().unwrap();
        assert_eq!(
            Vec::<u8>::try_from(lazy_opt_leaf.data().unwrap()).unwrap(),
            vec![0x11, 0x22, 0x33]
        );
        // Decoding an imported unannotated Union field directly via getter retries short reads.
        let cross_mod_lazy::plain_child::PlainUnion::PlainLeaf(u_leaf) = lazy_root.u().unwrap()
        else {
            panic!("wrong variant")
        };
        assert_eq!(u_leaf.tag().unwrap(), 7);
        assert_eq!(
            Vec::<u8>::try_from(u_leaf.data().unwrap()).unwrap(),
            vec![0x11, 0x22, 0x33]
        );
        // DynVec of imported unannotated Union retries short reads in get(), iter(), verify().
        let lazy_uvec = cross_mod_lazy::plain_root::PlainUnionVec::from(short_cursor(
            union_vec.as_slice().to_vec(),
            chunk,
        ));
        lazy_uvec.verify(false).unwrap();
        assert!(matches!(
            lazy_uvec.get(1).unwrap(),
            cross_mod_lazy::plain_child::PlainUnion::Byte(5)
        ));
        assert_eq!(lazy_uvec.iter().count(), 2);

        // Union in annotated root holding imported unannotated Table / Option / DynVec / FixVec / Union
        for (variant_id, payload) in [
            (1u32, child.as_slice()),
            (2u32, opt.as_slice()),
            (3u32, list.as_slice()),
            (4u32, pair_vec.as_slice()),
            (5u32, opt_pair_vec.as_slice()),
            (6u32, plain_u.as_slice()),
            (7u32, opt_u.as_slice()),
        ] {
            let wire = [variant_id.to_le_bytes().as_slice(), payload].concat();
            let u = cross_mod_lazy::plain_root::PlainRootUnion::try_from(short_cursor(wire, chunk))
                .unwrap();
            u.verify(false).unwrap();
            u.verify(true).unwrap();
        }
    }

    // Extra field inside the imported child table must fail strict verify(false) and pass verify(true)
    // even under 1-byte and 3-byte short reads.
    let mut compat_leaf = [22u32, 16, 17, 21].map(u32::to_le_bytes).concat();
    compat_leaf.push(7u8); // tag (1 byte at 16..17)
    compat_leaf.extend_from_slice(&0u32.to_le_bytes()); // data = empty PlainBytes (4 bytes at 17..21)
    compat_leaf.push(0xff); // extra field (1 byte at 21..22)
    let compat_opt = plain_child::OptPlainLeaf::new_unchecked(compat_leaf.into());
    let compat_root = root.clone().as_builder().opt(compat_opt).build();
    for chunk in [1usize, 3] {
        let lazy_compat = cross_mod_lazy::plain_root::PlainRoot::from(short_cursor(
            compat_root.as_slice().to_vec(),
            chunk,
        ));
        assert!(lazy_compat.verify(false).is_err());
        lazy_compat.verify(true).unwrap();
    }

    // Corrupt PlainBytes inside 2-hop imported PlainLeaf must fail verify even on 1-byte short reads.
    let bad_leaf_bytes = [15u32, 12, 13]
        .map(u32::to_le_bytes)
        .concat()
        .into_iter()
        .chain([7u8, 0x01, 0x02]) // 2-byte truncated PlainBytes header
        .collect::<Vec<_>>();
    let bad_opt = plain_child::OptPlainLeaf::new_unchecked(bad_leaf_bytes.into());
    let bad_root = root.as_builder().opt(bad_opt).build();
    for chunk in [1usize, 3] {
        let lazy_bad = cross_mod_lazy::plain_root::PlainRoot::from(short_cursor(
            bad_root.as_slice().to_vec(),
            chunk,
        ));
        assert!(lazy_bad.verify(false).is_err());
        assert!(lazy_bad.verify(true).is_err());
    }

    // Large payload (> MAX_CACHE_SIZE) read through imported unannotated child reader on short reads.
    let big_payload: Vec<u8> = (0..(molecule::lazy_reader::MAX_CACHE_SIZE + 37))
        .map(|i| (i & 0xff) as u8)
        .collect();
    let big_leaf = plain_leaf::PlainLeaf::new_builder()
        .tag(3u8)
        .data(
            plain_leaf::PlainBytes::new_builder()
                .extend(big_payload.iter().copied().map(Into::into))
                .build(),
        )
        .build();
    let big_root = plain_root::PlainRoot::new_builder()
        .amount(plain_root::RootAmt::from_u64(42))
        .opt(
            plain_child::OptPlainLeaf::new_builder()
                .set(Some(big_leaf))
                .build(),
        )
        .build();
    let lazy_big =
        cross_mod_lazy::plain_root::PlainRoot::from(short_cursor(big_root.as_slice().to_vec(), 3));
    lazy_big.verify(false).unwrap();
    assert_eq!(
        Vec::<u8>::try_from(lazy_big.opt().unwrap().unwrap().data().unwrap()).unwrap(),
        big_payload
    );

    // Importing a public union whose variant type lives in a private module must compile without
    // exposing the private module and still verify/decode under 1-byte and 3-byte short reads.
    let inner_table = [9u32, 8]
        .map(u32::to_le_bytes)
        .concat()
        .into_iter()
        .chain([0x5au8])
        .collect::<Vec<_>>();
    let outer_union = [0u32.to_le_bytes().as_slice(), &inner_table].concat();
    let pub_root_wire = [(20 + outer_union.len()) as u32, 12, 20]
        .map(u32::to_le_bytes)
        .concat()
        .into_iter()
        .chain(99u64.to_le_bytes())
        .chain(outer_union)
        .collect::<Vec<_>>();
    for chunk in [1usize, 3] {
        let pub_root = private_union_case::pkg_root::PublicRoot::from(short_cursor(
            pub_root_wire.clone(),
            chunk,
        ));
        pub_root.verify(false).unwrap();
        assert_eq!(pub_root.amount_value().unwrap(), 99);
        let private_union_case::pkg::outer::PublicOuter::PrivateInner(inner) =
            pub_root.outer().unwrap()
        else {
            panic!("wrong variant")
        };
        assert_eq!(inner.a().unwrap(), 0x5a);
    }
}
