#![no_main]

use lazy_reader_tests::{types_api, types_api2, types_moleculec_check::*, TypesCheckErr};
use libfuzzer_sys::fuzz_target;
use molecule::lazy_reader::Cursor;
use molecule::prelude::{Entity, Reader};

fn fuzz_f0(_data: &[u8], _cursor: Cursor) {
    // byte pass

    // let
}
fn fuzz_f1(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte2 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte2Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte2::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f1(&d1, &d2).expect("f1");
}

fn fuzz_f2(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte3 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f2(&d1, &d2).expect("f2");
}
fn fuzz_f3(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte4 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte4Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte4::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f3(&d1, &d2).expect("f3");
}
fn fuzz_f4(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte5 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte5Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte5::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f4(&d1, &d2).expect("f4");
}
fn fuzz_f5(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte6 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte6Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte6::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f5(&d1, &d2).expect("f5");
}
fn fuzz_f6(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte7 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte7Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte7::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f6(&d1, &d2).expect("f6");
}
fn fuzz_f7(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte8 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte8Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte8::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f7(&d1, &d2).expect("f7");
}
fn fuzz_f8(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte9 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte9Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte9::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f8(&d1, &d2).expect("f8");
}
fn fuzz_f9(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte10 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte10Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte10::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f9(&d1, &d2).expect("f9");
}
fn fuzz_f10(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte11 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte11Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte11::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f10(&d1, &d2).expect("f10");
}
fn fuzz_f11(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte12 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte12Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte12::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f11(&d1, &d2).expect("f11");
}
fn fuzz_f12(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte13 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte13Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte13::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f12(&d1, &d2).expect("f12");
}
fn fuzz_f13(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte14 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte14Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte14::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f13(&d1, &d2).expect("f13");
}
fn fuzz_f14(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte15 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte15Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte15::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f14(&d1, &d2).expect("f14");
}
fn fuzz_f15(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte16 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte16Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte16::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f15(&d1, &d2).expect("f15");
}
fn fuzz_f16(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::WordReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f16(&d1, &d2).expect("f16");
}
fn fuzz_f17(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word2 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word2Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word2::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f17(&d1, &d2).expect("f17");
}
fn fuzz_f18(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word3 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f18(&d1, &d2).expect("f18");
}
fn fuzz_f19(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word4 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word4Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word4::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f19(&d1, &d2).expect("f19");
}
fn fuzz_f20(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word5 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word5Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word5::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f20(&d1, &d2).expect("f20");
}
fn fuzz_f21(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word6 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word6Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word6::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f21(&d1, &d2).expect("f21");
}
fn fuzz_f22(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word7 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word7Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word7::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f22(&d1, &d2).expect("f22");
}
fn fuzz_f23(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Word8 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Word8Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Word8::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f23(&d1, &d2).expect("f23");
}
fn fuzz_f24(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte3x3 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte3x3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte3x3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f24(&d1, &d2).expect("f24");
}
fn fuzz_f25(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte5x3 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte5x3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte5x3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f25(&d1, &d2).expect("f25");
}
fn fuzz_f26(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte7x3 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte7x3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte7x3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f26(&d1, &d2).expect("f26");
}
fn fuzz_f27(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Byte9x3 = cursor.into();
    let _r2 = try_get_vec(&d2);

    if types_api::Byte9x3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Byte9x3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f27(&d1, &d2).expect("f27");
}
fn fuzz_f28(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructA = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructAReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructA::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f28(&d1, &d2).expect("f28");
}
fn fuzz_f29(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructB = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructBReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructB::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f29(&d1, &d2).expect("f29");
}
fn fuzz_f30(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructC = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructCReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructC::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f30(&d1, &d2).expect("f30");
}
fn fuzz_f31(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructD = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructDReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructD::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f31(&d1, &d2).expect("f31");
}
fn fuzz_f32(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructE = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructEReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructE::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f32(&d1, &d2).expect("f32");
}
fn fuzz_f33(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructF = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();

    if types_api::StructFReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructF::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f33(&d1, &d2).expect("f33");
}
fn fuzz_f34(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructG = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructGReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructG::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f34(&d1, &d2).expect("f34");
}
fn fuzz_f35(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructH = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();
    let _ = d2.f3();
    let _ = d2.f4();

    if types_api::StructHReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructH::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f35(&d1, &d2).expect("f35");
}
fn fuzz_f36(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructI = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();

    if types_api::StructIReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructI::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f36(&d1, &d2).expect("f36");
}
fn fuzz_f37(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructJ = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();

    if types_api::StructJReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructJ::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f37(&d1, &d2).expect("f37");
}
fn fuzz_f38(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructIx3 = cursor.into();
    let _ = try_get_vec(&d2);

    if types_api::StructIx3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructIx3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f38(&d1, &d2).expect("f38");
}
fn fuzz_f39(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructO = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();

    if types_api::StructOReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructO::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f39(&d1, &d2).expect("f39");
}
fn fuzz_f40(data: &[u8], cursor: Cursor) {
    let d2: types_api2::StructP = cursor.into();
    let _ = d2.f1();
    let _ = d2.f2();

    if types_api::StructPReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::StructP::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f40(&d1, &d2).expect("f40");
}
fn fuzz_f41(data: &[u8], cursor: Cursor) {
    if types_api::BytesReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::Bytes::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::Bytes = cursor.clone().into();

    for i in 0..d1.len() {
        TypesCheckErr::check_1_data(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into())
            .expect("check 1 data");
    }
}
fn fuzz_f42(data: &[u8], cursor: Cursor) {
    if types_api::WordsReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::Words::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::Words = cursor.clone().into();

    for i in 0..d1.len() {
        check_f16(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into()).expect("check 1 data");
    }
}
fn fuzz_f43(data: &[u8], cursor: Cursor) {
    if types_api::Byte3VecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::Byte3Vec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::Byte3Vec = cursor.clone().into();

    for i in 0..d1.len() {
        check_f2(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into()).expect("check 1 data");
    }
}
fn fuzz_f44(data: &[u8], cursor: Cursor) {
    if types_api::Byte7VecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::Byte7Vec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::Byte7Vec = cursor.clone().into();

    for i in 0..d1.len() {
        check_f6(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into()).expect("check 1 data");
    }
}
fn fuzz_f45(data: &[u8], cursor: Cursor) {
    if types_api::StructIVecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::StructIVec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::StructIVec = cursor.clone().into();

    for i in 0..d1.len() {
        check_f36(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into()).expect("check 1 data");
    }
}
fn fuzz_f46(data: &[u8], cursor: Cursor) {
    if types_api::StructJVecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::StructJVec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::StructJVec = cursor.clone().into();

    for i in 0..d1.len() {
        check_f37(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into()).expect("check 1 data");
    }
}
fn fuzz_f47(data: &[u8], cursor: Cursor) {
    if types_api::StructPVecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::StructPVec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::StructPVec = cursor.clone().into();

    for i in 0..d1.len() {
        check_f40(&d1.get(i).unwrap(), &d2.get(i).expect("get d2").into()).expect("check 1 data");
    }
}
fn fuzz_f48(data: &[u8], cursor: Cursor) {
    if types_api::BytesVecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::BytesVec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::BytesVec = cursor.clone().into();

    for i in 0..d1.len() {
        let dd1 = d1.get(i).unwrap();
        let dd2: Vec<u8> = d2.get(i).expect("get d2").try_into().unwrap();
        for j in 0..dd1.len() {
            TypesCheckErr::check_1_data(&dd1.get(j).unwrap(), &dd2.get(j).unwrap().clone().into())
                .expect("check 1 data");
        }
    }
}
fn fuzz_f49(data: &[u8], cursor: Cursor) {
    if types_api::WordsVecReader::verify(data, true).is_err() {
        return;
    }
    let d1 = types_api::WordsVec::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::WordsVec = cursor.clone().into();

    for i in 0..d1.len() {
        let dd1 = d1.get(i).unwrap();
        let dd2 = d2.get(i).expect("get d2");
        for j in 0..dd1.len() {
            check_f16(&dd1.get(j).unwrap(), &dd2.get(j).unwrap().clone().into())
                .expect("check 1 data");
        }
    }
}
fn fuzz_f50(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table0 = cursor.into();

    if types_api::Table0Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table0::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f50(&d1, &d2).expect("f50");
}
fn fuzz_f51(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table1 = cursor.into();

    if types_api::Table1Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table1::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f51(&d1, &d2).expect("f51");
}
fn fuzz_f52(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table2 = cursor.into();

    if types_api::Table2Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table2::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f52(&d1, &d2).expect("f52");
}
fn fuzz_f53(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table3 = cursor.into();

    if types_api::Table3Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table3::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f53(&d1, &d2).expect("f53");
}
fn fuzz_f54(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table4 = cursor.into();

    if types_api::Table4Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table4::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f54(&d1, &d2).expect("f54");
}
fn fuzz_f55(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table5 = cursor.into();

    if types_api::Table5Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table5::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f55(&d1, &d2).expect("f55");
}
fn fuzz_f56(data: &[u8], cursor: Cursor) {
    let d2: types_api2::Table6 = cursor.into();

    if types_api::Table6Reader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::Table6::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    check_f56(&d1, &d2).expect("f56");
}
fn fuzz_f57(data: &[u8], cursor: Cursor) {
    if types_api::BytesOptReader::verify(data, true).is_err() {
        return;
    }

    let _d1 = types_api::BytesOpt::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let _d2: types_api2::BytesOpt = cursor.into();
    // check_f57(&d1.try_into(), &d2).expect("f57");
}
fn fuzz_f74(data: &[u8], cursor: Cursor) {
    if types_api::TableBReader::verify(data, true).is_err() {
        return;
    }

    let d1 = types_api::TableB::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let d2: types_api2::TableB = cursor.into();
    check_f74(&d1, &d2).expect("f74");
}

fn fuzz_all(data: &[u8], cursor: Cursor) {
    if types_api::AllInOneReader::verify(&data, true).is_err() {
        return;
    }

    let all1 = types_api::AllInOne::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let all2: types_api2::AllInOne = cursor.into();

    check_mol(&all1, &all2).expect("check mol");
}

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data.len(), Box::new(data.to_vec()));
    fuzz_f0(data, cursor.clone());
    fuzz_f1(data, cursor.clone());
    fuzz_f2(data, cursor.clone());
    fuzz_f3(data, cursor.clone());
    fuzz_f4(data, cursor.clone());
    fuzz_f5(data, cursor.clone());
    fuzz_f6(data, cursor.clone());
    fuzz_f7(data, cursor.clone());
    fuzz_f8(data, cursor.clone());
    fuzz_f9(data, cursor.clone());
    fuzz_f10(data, cursor.clone());
    fuzz_f11(data, cursor.clone());
    fuzz_f12(data, cursor.clone());
    fuzz_f13(data, cursor.clone());
    fuzz_f14(data, cursor.clone());
    fuzz_f15(data, cursor.clone());
    fuzz_f16(data, cursor.clone());
    fuzz_f17(data, cursor.clone());
    fuzz_f18(data, cursor.clone());
    fuzz_f19(data, cursor.clone());
    fuzz_f20(data, cursor.clone());
    fuzz_f21(data, cursor.clone());
    fuzz_f22(data, cursor.clone());
    fuzz_f23(data, cursor.clone());
    fuzz_f24(data, cursor.clone());
    fuzz_f25(data, cursor.clone());
    fuzz_f26(data, cursor.clone());
    fuzz_f27(data, cursor.clone());
    fuzz_f28(data, cursor.clone());
    fuzz_f29(data, cursor.clone());
    fuzz_f30(data, cursor.clone());
    fuzz_f31(data, cursor.clone());
    fuzz_f32(data, cursor.clone());
    fuzz_f33(data, cursor.clone());
    fuzz_f34(data, cursor.clone());
    fuzz_f35(data, cursor.clone());
    fuzz_f36(data, cursor.clone());
    fuzz_f37(data, cursor.clone());
    fuzz_f38(data, cursor.clone());
    fuzz_f39(data, cursor.clone());
    fuzz_f40(data, cursor.clone());
    fuzz_f41(data, cursor.clone());
    fuzz_f42(data, cursor.clone());
    fuzz_f43(data, cursor.clone());
    fuzz_f44(data, cursor.clone());
    fuzz_f45(data, cursor.clone());
    fuzz_f46(data, cursor.clone());
    fuzz_f47(data, cursor.clone());
    fuzz_f48(data, cursor.clone());
    fuzz_f49(data, cursor.clone());
    fuzz_f50(data, cursor.clone());
    fuzz_f51(data, cursor.clone());
    fuzz_f52(data, cursor.clone());
    fuzz_f53(data, cursor.clone());
    fuzz_f54(data, cursor.clone());
    fuzz_f55(data, cursor.clone());
    fuzz_f56(data, cursor.clone());
    fuzz_f57(data, cursor.clone());
    fuzz_f74(data, cursor.clone());

    fuzz_all(data, cursor.clone());
});
