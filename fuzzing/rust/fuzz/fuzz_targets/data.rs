#![no_main]

use libfuzzer_sys::fuzz_target;
use molecule_fuzzing::definitions::*;
use molecule_fuzzing::molecule::prelude::*;

fn access_fixvec_reader(data: &[u8]) -> u64 {
    if let Ok(_) = FixVecTypeReader::verify(data, false) {
        let fixvec = FixVecTypeReader::new_unchecked(data);
        let len = fixvec.len();
        if len > 0 {
            let first: u8 = fixvec.get(0).unwrap().into();
            let last: u8 = fixvec.get(len-1).unwrap().into();
            first as u64 + last as u64
        } else {
            0
        }    
    } else {
        0
    }
}

fn access_array_reader(data: &[u8]) -> u64 {
    if let Ok(_) = ArrayTypeReader::verify(data, false) {
        let array = ArrayTypeReader::new_unchecked(data);
        let first : u8 = array.nth0().into();
        let last : u8 = array.nth2().into();
        first as u64 + last as u64    
    } else {
        0
    }
}

fn access_struct_reader(data: &[u8]) -> u64 {
    if let Ok(_) = StructTypeReader::verify(data, false) {
        let struct_ = StructTypeReader::new_unchecked(data);
        let mut total: u64 = 0;
        let f1 = struct_.f1();
        total += access_array(f1.as_slice());
        let f2 = struct_.f2();
        let f2_value: u8 = f2.into();
        total += f2_value as u64;
        total    
    } else {
        0
    }
}

fn access_dynvec_reader(data: &[u8]) -> u64 {
    if let Ok(_) = DynVecTypeReader::verify(data, false) {
        let dynvec = DynVecTypeReader::new_unchecked(data);
        let mut total: u64 = 0;
        let len = dynvec.len();
        if len > 0 {
            let first = dynvec.get(0).unwrap();
            let last = dynvec.get(len - 1).unwrap();
            total += access_fixvec(first.raw_data());
            total += access_fixvec(last.raw_data());
            total
        } else {
            0
        }    
    } else {
        0
    }
}

fn access_opt_reader(data: &[u8]) -> u64 {
    if let Ok(_) = OptTypeReader::verify(data, false) {
        let opt = OptTypeReader::new_unchecked(data);
        if opt.is_some() {
            let dynvec = opt.to_opt().unwrap();
            access_dynvec(dynvec.as_slice())
        } else {
            0
        }    
    } else {
        0
    }
}

fn access_table_reader(data: &[u8]) -> u64 {
    let mut total: u64 = 0;
    if let Ok(_) = TableTypeReader::verify(data, false) {
        let tbl = TableTypeReader::new_unchecked(data);
        let fixvec = tbl.f1();
        total += access_fixvec(fixvec.as_slice());
        let dynvec = tbl.f2();
        total += access_dynvec(dynvec.as_slice());
        let struct_ = tbl.f3();
        access_struct(struct_.as_slice());
        let array = tbl.f4();
        total += access_array(array.as_slice());
        let opt = tbl.f5();
        total += access_opt(opt.as_slice());
    }
    total
}

fn access_union_reader(data: &[u8]) -> u64 {
    let mut total: u64 = 0;
    if let Ok(_) = UnionTypeReader::verify(data, false) {
        let union_ = UnionTypeReader::new_unchecked(data);
        match union_.to_enum() {
            UnionTypeUnionReader::ArrayType(array) => {
                total += access_array(array.as_slice());
            },
            UnionTypeUnionReader::StructType(struct_) => {
                total += access_struct(struct_.as_slice());
            },
            UnionTypeUnionReader::FixVecType(fixvec) => {
                total += access_fixvec(fixvec.as_slice());
            },
            UnionTypeUnionReader::DynVecType(dynvec) => {
                total += access_dynvec(dynvec.as_slice());
            },
            UnionTypeUnionReader::TableType(tbl) => {
                total += access_table(tbl.as_slice());
            },
        }
    }
    total
}

fn access_fixvec(data: &[u8]) -> u64 {
    if let Ok(_) = FixVecTypeReader::verify(data, false) {
        let fixvec = FixVecType::from_slice(data).unwrap();
        let len = fixvec.len();
        if len > 0 {
            let first: u8 = fixvec.get(0).unwrap().into();
            let last: u8 = fixvec.get(len-1).unwrap().into();
            first as u64 + last as u64
        } else {
            0
        }    
    } else {
        0
    }
}

fn access_array(data: &[u8]) -> u64 {
    if let Ok(_) = ArrayTypeReader::verify(data, false) {
        let array = ArrayType::from_slice(data).unwrap();
        let first : u8 = array.nth0().into();
        let last : u8 = array.nth2().into();
        first as u64 + last as u64    
    } else {
        0
    }
}

fn access_struct(data: &[u8]) -> u64 {
    if let Ok(_) = StructTypeReader::verify(data, false) {
        let struct_ = StructTypeReader::from_slice(data).unwrap();
        let mut total: u64 = 0;
        let f1 = struct_.f1();
        total += access_array(f1.as_slice());
        let f2 = struct_.f2();
        let f2_value: u8 = f2.into();
        total += f2_value as u64;
        total    
    } else {
        0
    }
}

fn access_dynvec(data: &[u8]) -> u64 {
    if let Ok(_) = DynVecTypeReader::verify(data, false) {
        let dynvec = DynVecType::from_slice(data).unwrap();
        let mut total: u64 = 0;
        let len = dynvec.len();
        if len > 0 {
            let first = dynvec.get(0).unwrap();
            let last = dynvec.get(len - 1).unwrap();
            total += access_fixvec(&first.raw_data());
            total += access_fixvec(&last.raw_data());
            total
        } else {
            0
        }    
    } else {
        0
    }
}

fn access_opt(data: &[u8]) -> u64 {
    if let Ok(_) = OptTypeReader::verify(data, false) {
        let opt = OptType::from_slice(data).unwrap();
        if opt.is_some() {
            let dynvec = opt.to_opt().unwrap();
            access_dynvec(dynvec.as_slice())
        } else {
            0
        }    
    } else {
        0
    }
}

fn access_table(data: &[u8]) -> u64 {
    let mut total: u64 = 0;
    if let Ok(_) = TableTypeReader::verify(data, false) {
        let tbl = TableType::from_slice(data).unwrap();
        let fixvec = tbl.f1();
        total += access_fixvec(fixvec.as_slice());
        let dynvec = tbl.f2();
        total += access_dynvec(dynvec.as_slice());
        let struct_ = tbl.f3();
        access_struct(struct_.as_slice());
        let array = tbl.f4();
        total += access_array(array.as_slice());
        let opt = tbl.f5();
        total += access_opt(opt.as_slice());
    }
    total
}


fn access_union(data: &[u8]) -> u64 {
    let mut total: u64 = 0;
    if let Ok(_) = UnionTypeReader::verify(data, false) {
        let union_ = UnionType::from_slice(data).unwrap();
        match union_.to_enum() {
            UnionTypeUnion::ArrayType(array) => {
                total += access_array(array.as_slice());
            },
            UnionTypeUnion::StructType(struct_) => {
                total += access_struct(struct_.as_slice());
            },
            UnionTypeUnion::FixVecType(fixvec) => {
                total += access_fixvec(fixvec.as_slice());
            },
            UnionTypeUnion::DynVecType(dynvec) => {
                total += access_dynvec(dynvec.as_slice());
            },
            UnionTypeUnion::TableType(tbl) => {
                total += access_table(tbl.as_slice());
            },
        }
    }
    total
}

fuzz_target!(|data: &[u8]| {
    let mut total : u64 = 0;
    total += access_table_reader(data);
    total += access_array_reader(data);
    total += access_dynvec_reader(data);
    total += access_fixvec_reader(data);
    total += access_opt_reader(data);
    total += access_struct_reader(data);
    total += access_union_reader(data);

    total += access_table(data);
    total += access_array(data);
    total += access_dynvec(data);
    total += access_fixvec(data);
    total += access_opt(data);
    total += access_struct(data);
    total += access_union(data);


    let _ = total;
});
