#![no_main]

use libfuzzer_sys::fuzz_target;
use molecule::prelude::{Entity, Reader};
use molecule::lazy_reader::Cursor;
use lazy_reader_tests::{types_api, types_api2, types_moleculec_check::check_mol};

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data.len(), Box::new(data.to_vec()));

    if types_api::AllInOneReader::verify(data, true).is_err() {
        return;
    }

    let all1 = types_api::AllInOne::new_unchecked(molecule::bytes::Bytes::from(data.to_vec()));
    let all2: types_api2::AllInOne = cursor.into();

    check_mol(&all1, &all2).expect("check mol");
});
