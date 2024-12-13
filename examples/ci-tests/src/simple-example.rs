use molecule::prelude::*;

use molecule_ci_tests::types;

fn display_empty_data() {
    let data = types::AllInOne::default();
    println!("EmptyData Length = {}\n", data.as_slice().len());
    println!("EmptyData Debug:\n{:?}\n", data);
    println!("EmptyData Display:\n{}\n", data);
    println!("EmptyData LowerHex:\n{:x}\n", data);
    println!("EmptyData LowerHex (alternate):\n{:#x}\n", data);
}

fn display_test_data() {
    let f0: types::Byte = 0x12u8.into();
    let f2 = types::Byte3::new_builder().nth1(f0).build();
    let f29 = types::StructB::new_builder()
        .f2(0x34u8)
        .f4(f2.clone())
        .build();
    let f41 = {
        let mut f41_builder = types::Bytes::new_builder().push(0x12).push(0x12).push(0x13);
        assert_eq!(f41_builder.replace(1, 0x34), Some(0x12.into()));
        assert_eq!(f41_builder.replace(2, 0x56), Some(0x13.into()));
        assert_eq!(f41_builder.replace(3, 0x56), None);
        f41_builder.build()
    };
    let f43 = types::Byte3Vec::new_builder()
        .push(f2.clone())
        .push(f2.clone())
        .push(f2.clone())
        .build();
    let f48 = types::BytesVec::new_builder()
        .push(f41.clone())
        .push(f41.clone())
        .push(f41.clone())
        .build();
    let f61 = types::BytesOpt::new_builder()
        .set(Some(f41.clone()))
        .build();
    let f62 = types::WordsOpt::new_builder().build();
    let f72 = types::UnionA::new_builder().set(f41.clone()).build();
    let data = types::AllInOne::new_builder()
        .f0(f0)
        .f2(f2)
        .f29(f29)
        .f41(f41)
        .f43(f43)
        .f48(f48)
        .f61(f61)
        .f62(f62)
        .f72(f72)
        .build();
    for (i, b) in data.as_slice().iter().enumerate() {
        if i % 32 == 0 {
            print!("\nAllInOneTestData :  ");
        }
        print!("{:02x}", *b);
    }
    println!();
}

fn main() {
    display_empty_data();
    display_test_data();
}
