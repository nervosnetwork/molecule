use crate::{types_api, types_api2, Mol2Vec, ResCheckErr, TypesCheckErr};
use molecule::lazy_reader::Cursor;
use molecule::prelude::Byte;

pub fn check_mol(d1: &types_api::AllInOne, d2: &types_api2::AllInOne) -> ResCheckErr {
    check_f0(&d1.f0(), &d2.f0()?.into())?;
    check_f1(&d1.f1(), &d2.f1()?.into())?;
    check_f2(&d1.f2(), &d2.f2()?.into())?;
    check_f3(&d1.f3(), &d2.f3()?.into())?;
    check_f4(&d1.f4(), &d2.f4()?.into())?;
    check_f5(&d1.f5(), &d2.f5()?.into())?;
    check_f6(&d1.f6(), &d2.f6()?.into())?;
    check_f7(&d1.f7(), &d2.f7()?.into())?;
    check_f8(&d1.f8(), &d2.f8()?.into())?;
    check_f9(&d1.f9(), &d2.f9()?.into())?;
    check_f10(&d1.f10(), &d2.f10()?.into())?;
    check_f11(&d1.f11(), &d2.f11()?.into())?;
    check_f12(&d1.f12(), &d2.f12()?.into())?;
    check_f13(&d1.f13(), &d2.f13()?.into())?;
    check_f14(&d1.f14(), &d2.f14()?.into())?;
    check_f15(&d1.f15(), &d2.f15()?.into())?;

    check_f16(&d1.f16(), &d2.f16()?.into())?;
    check_f17(&d1.f17(), &d2.f17()?.into())?;
    check_f18(&d1.f18(), &d2.f18()?.into())?;
    check_f19(&d1.f19(), &d2.f19()?.into())?;
    check_f20(&d1.f20(), &d2.f20()?.into())?;
    check_f21(&d1.f21(), &d2.f21()?.into())?;
    check_f22(&d1.f22(), &d2.f22()?.into())?;
    check_f23(&d1.f23(), &d2.f23()?.into())?;

    check_f24(&d1.f24(), &d2.f24()?.into())?;
    check_f25(&d1.f25(), &d2.f25()?.into())?;
    check_f26(&d1.f26(), &d2.f26()?.into())?;
    check_f27(&d1.f27(), &d2.f27()?.into())?;

    check_f28(&d1.f28(), &d2.f28()?.into())?;
    check_f29(&d1.f29(), &d2.f29()?.into())?;
    check_f30(&d1.f30(), &d2.f30()?.into())?;
    check_f31(&d1.f31(), &d2.f31()?.into())?;
    check_f32(&d1.f32(), &d2.f32()?.into())?;
    check_f33(&d1.f33(), &d2.f33()?.into())?;
    check_f34(&d1.f34(), &d2.f34()?.into())?;
    check_f35(&d1.f35(), &d2.f35()?.into())?;
    check_f36(&d1.f36(), &d2.f36()?.into())?;
    check_f37(&d1.f37(), &d2.f37()?.into())?;
    check_f38(&d1.f38(), &d2.f38()?.into())?;
    check_f39(&d1.f39(), &d2.f39()?.into())?;
    check_f40(&d1.f40(), &d2.f40()?.into())?;

    check_f41(&d1.f41(), &d2.f41()?)?;
    check_f42(&d1.f42(), &d2.f42()?.into())?;
    check_f43(&d1.f43(), &d2.f43()?.into())?;
    check_f44(&d1.f44(), &d2.f44()?.into())?;
    check_f45(&d1.f45(), &d2.f45()?.into())?;
    check_f46(&d1.f46(), &d2.f46()?.into())?;
    check_f47(&d1.f47(), &d2.f47()?.into())?;
    check_f48(&d1.f48(), &d2.f48()?.into())?;
    check_f49(&d1.f49(), &d2.f49()?.into())?;

    check_f50(&d1.f50(), &d2.f50()?.into())?;
    check_f51(&d1.f51(), &d2.f51()?.into())?;
    check_f52(&d1.f52(), &d2.f52()?.into())?;
    check_f53(&d1.f53(), &d2.f53()?.into())?;
    check_f54(&d1.f54(), &d2.f54()?.into())?;
    check_f55(&d1.f55(), &d2.f55()?.into())?;
    check_f56(&d1.f56(), &d2.f56()?.into())?;

    check_f57(&d1.f57().to_opt(), &d2.f57()?)?;
    check_f58(&d1.f58().to_opt(), &d2.f58()?.map(|f| f.into()))?;
    check_f59(&d1.f59().to_opt(), &d2.f59()?)?;
    check_f60(&d1.f60().to_opt(), &d2.f60()?)?;
    check_f61(
        &d1.f61().to_opt(),
        &d2.f61()?.map(|f| f.try_into().unwrap()),
    )?;
    check_f62(&d1.f62().to_opt(), &d2.f62()?)?;
    check_f63(&d1.f63().to_opt(), &d2.f63()?)?;
    check_f64(&d1.f64().to_opt(), &d2.f64()?)?;
    check_f65(&d1.f65().to_opt(), &d2.f65()?)?;
    check_f66(&d1.f66().to_opt(), &d2.f66()?)?;
    check_f67(&d1.f67().to_opt().map(|f| f.to_opt()), &d2.f67()?)?;

    check_f68(&d1.f68(), &d2.f68()?)?;
    check_f69(&d1.f69(), &d2.f69()?)?;
    check_f70(&d1.f70(), &d2.f70()?)?;
    check_f71(&d1.f71(), &d2.f71()?)?;

    check_f72(&d1.f72(), &d2.f72()?)?;
    check_f73(&d1.f73(), &d2.f73()?)?;
    check_f74(&d1.f74(), &d2.f74()?)?;

    Ok(())
}

pub fn try_get_vec<T: Mol2Vec>(t: &T) -> ResCheckErr {
    for i in 0..t.mol_len()? {
        t.mol_get(i)?;
    }
    Ok(())
}

pub fn check_f0(d1: &Byte, d2: &Byte) -> ResCheckErr {
    TypesCheckErr::check_1_data(d1, d2)
}
pub fn check_f1(d1: &types_api::Byte2, d2: &types_api2::Byte2) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f2(d1: &types_api::Byte3, d2: &types_api2::Byte3) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f3(d1: &types_api::Byte4, d2: &types_api2::Byte4) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f4(d1: &types_api::Byte5, d2: &types_api2::Byte5) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f5(d1: &types_api::Byte6, d2: &types_api2::Byte6) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f6(d1: &types_api::Byte7, d2: &types_api2::Byte7) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f7(d1: &types_api::Byte8, d2: &types_api2::Byte8) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f8(d1: &types_api::Byte9, d2: &types_api2::Byte9) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f9(d1: &types_api::Byte10, d2: &types_api2::Byte10) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f10(d1: &types_api::Byte11, d2: &types_api2::Byte11) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f11(d1: &types_api::Byte12, d2: &types_api2::Byte12) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f12(d1: &types_api::Byte13, d2: &types_api2::Byte13) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f13(d1: &types_api::Byte14, d2: &types_api2::Byte14) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f14(d1: &types_api::Byte15, d2: &types_api2::Byte15) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f15(d1: &types_api::Byte16, d2: &types_api2::Byte16) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f16(d1: &types_api::Word, d2: &types_api2::Word) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
pub fn check_f17(d1: &types_api::Word2, d2: &types_api2::Word2) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f18(d1: &types_api::Word3, d2: &types_api2::Word3) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f19(d1: &types_api::Word4, d2: &types_api2::Word4) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f20(d1: &types_api::Word5, d2: &types_api2::Word5) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f21(d1: &types_api::Word6, d2: &types_api2::Word6) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f22(d1: &types_api::Word7, d2: &types_api2::Word7) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f23(d1: &types_api::Word8, d2: &types_api2::Word8) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f24(d1: &types_api::Byte3x3, d2: &types_api2::Byte3x3) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f25(d1: &types_api::Byte5x3, d2: &types_api2::Byte5x3) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f26(d1: &types_api::Byte7x3, d2: &types_api2::Byte7x3) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f27(d1: &types_api::Byte9x3, d2: &types_api2::Byte9x3) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f28(d1: &types_api::StructA, d2: &types_api2::StructA) -> ResCheckErr {
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;
    check_f1(&d1.f3(), &d2.f3()?.into())?;
    check_f1(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f29(d1: &types_api::StructB, d2: &types_api2::StructB) -> ResCheckErr {
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;
    check_f1(&d1.f3(), &d2.f3()?.into())?;
    check_f2(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f30(d1: &types_api::StructC, d2: &types_api2::StructC) -> ResCheckErr {
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;
    check_f1(&d1.f3(), &d2.f3()?.into())?;
    check_f3(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f31(d1: &types_api::StructD, d2: &types_api2::StructD) -> ResCheckErr {
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;
    check_f1(&d1.f3(), &d2.f3()?.into())?;
    check_f4(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f32(d1: &types_api::StructE, d2: &types_api2::StructE) -> ResCheckErr {
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f1(&d1.f2(), &d2.f2()?.into())?;
    check_f0(&d1.f3(), &d2.f3()?.into())?;
    check_f1(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f33(d1: &types_api::StructF, d2: &types_api2::StructF) -> ResCheckErr {
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f2(&d1.f2(), &d2.f2()?.into())?;
    check_f0(&d1.f3(), &d2.f3()?.into())?;

    Ok(())
}
pub fn check_f34(d1: &types_api::StructG, d2: &types_api2::StructG) -> ResCheckErr {
    check_f2(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;
    check_f1(&d1.f3(), &d2.f3()?.into())?;
    check_f17(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f35(d1: &types_api::StructH, d2: &types_api2::StructH) -> ResCheckErr {
    check_f2(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;
    check_f1(&d1.f3(), &d2.f3()?.into())?;
    check_f3(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
pub fn check_f36(d1: &types_api::StructI, d2: &types_api2::StructI) -> ResCheckErr {
    check_f2(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
pub fn check_f37(d1: &types_api::StructJ, d2: &types_api2::StructJ) -> ResCheckErr {
    check_f5(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
pub fn check_f38(d1: &types_api::StructIx3, d2: &types_api2::StructIx3) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        check_f36(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }

    Ok(())
}
pub fn check_f39(d1: &types_api::StructO, d2: &types_api2::StructO) -> ResCheckErr {
    check_f38(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
pub fn check_f40(d1: &types_api::StructP, d2: &types_api2::StructP) -> ResCheckErr {
    check_f37(&d1.f1(), &d2.f1()?.into())?;
    check_f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
pub fn check_f41(d1: &types_api::Bytes, d2: &Cursor) -> ResCheckErr {
    d2.verify_fixed_size(d1.len())?;
    let d2: Vec<u8> = d2.clone().try_into().unwrap();
    TypesCheckErr::check_mol_data(d1, &d2)
}
pub fn check_f42(d1: &types_api::Words, d2: &types_api2::Words) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f43(d1: &types_api::Byte3Vec, d2: &types_api2::Byte3Vec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f44(d1: &types_api::Byte7Vec, d2: &types_api2::Byte7Vec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f45(d1: &types_api::StructIVec, d2: &types_api2::StructIVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        check_f36(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f46(d1: &types_api::StructJVec, d2: &types_api2::StructJVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        check_f37(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f47(d1: &types_api::StructPVec, d2: &types_api2::StructPVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        check_f40(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f48(d1: &types_api::BytesVec, d2: &types_api2::BytesVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        check_f41(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f49(d1: &types_api::WordsVec, d2: &types_api2::WordsVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        check_f42(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
pub fn check_f50(_d1: &types_api::Table0, _d2: &types_api2::Table0) -> ResCheckErr {
    // assert!(!d2.cursor.table_has_extra_fields(0)?);
    Ok(())
}
pub fn check_f51(d1: &types_api::Table1, d2: &types_api2::Table1) -> ResCheckErr {
    assert!(d2.cursor.table_has_extra_fields(0)?);
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    Ok(())
}
pub fn check_f52(d1: &types_api::Table2, d2: &types_api2::Table2) -> ResCheckErr {
    assert!(d2.cursor.table_has_extra_fields(1)?);
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f17(&d1.f2(), &d2.f2()?.into())?;
    Ok(())
}
pub fn check_f53(d1: &types_api::Table3, d2: &types_api2::Table3) -> ResCheckErr {
    assert!(d2.cursor.table_has_extra_fields(2)?);
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f17(&d1.f2(), &d2.f2()?.into())?;
    check_f28(&d1.f3(), &d2.f3()?.into())?;
    Ok(())
}
pub fn check_f54(d1: &types_api::Table4, d2: &types_api2::Table4) -> ResCheckErr {
    assert!(d2.cursor.table_has_extra_fields(3)?);
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f17(&d1.f2(), &d2.f2()?.into())?;
    check_f28(&d1.f3(), &d2.f3()?.into())?;
    check_f41(&d1.f4(), &d2.f4()?.try_into().unwrap())?;
    Ok(())
}
pub fn check_f55(d1: &types_api::Table5, d2: &types_api2::Table5) -> ResCheckErr {
    assert!(d2.cursor.table_has_extra_fields(4)?);
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f17(&d1.f2(), &d2.f2()?.into())?;
    check_f28(&d1.f3(), &d2.f3()?.into())?;
    check_f41(&d1.f4(), &d2.f4()?.try_into().unwrap())?;
    check_f48(&d1.f5(), &d2.f5()?.into())?;
    Ok(())
}
pub fn check_f56(d1: &types_api::Table6, d2: &types_api2::Table6) -> ResCheckErr {
    assert!(d2.cursor.table_has_extra_fields(5)?);
    check_f0(&d1.f1(), &d2.f1()?.into())?;
    check_f17(&d1.f2(), &d2.f2()?.into())?;
    check_f28(&d1.f3(), &d2.f3()?.into())?;
    check_f41(&d1.f4(), &d2.f4()?.try_into().unwrap())?;
    check_f48(&d1.f5(), &d2.f5()?.into())?;
    check_f55(&d1.f6(), &d2.f6()?.into())?;
    Ok(())
}
pub fn check_f57(d1: &Option<Byte>, d2: &Option<u8>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    TypesCheckErr::check_1_data(d1.as_ref().unwrap(), &d2.as_ref().unwrap().clone().into())?;

    Ok(())
}
pub fn check_f58(d1: &Option<types_api::Word>, d2: &Option<types_api2::Word>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f16(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f59(d1: &Option<types_api::StructA>, d2: &Option<types_api2::StructA>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f28(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f60(d1: &Option<types_api::StructP>, d2: &Option<types_api2::StructP>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f40(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f61(d1: &Option<types_api::Bytes>, d2: &Option<Cursor>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f41(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f62(d1: &Option<types_api::Words>, d2: &Option<types_api2::Words>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f42(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f63(
    d1: &Option<types_api::BytesVec>,
    d2: &Option<types_api2::BytesVec>,
) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f48(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f64(
    d1: &Option<types_api::WordsVec>,
    d2: &Option<types_api2::WordsVec>,
) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f49(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f65(d1: &Option<types_api::Table0>, d2: &Option<types_api2::Table0>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f50(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f66(d1: &Option<types_api::Table6>, d2: &Option<types_api2::Table6>) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f56(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f67(
    d1: &Option<Option<types_api::Table6>>,
    d2: &Option<Option<types_api2::Table6>>,
) -> ResCheckErr {
    if d1.is_some() != d2.is_some() {
        return Err(TypesCheckErr::Opt(format!(
            "different option: {:?}  {:?}",
            d1.is_some(),
            d2.is_some()
        )));
    }
    if d1.is_none() {
        return Ok(());
    }

    check_f66(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
pub fn check_f68(d1: &types_api::ByteOptVec, d2: &types_api2::ByteOptVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?;
        check_f57(&dd1, &dd2)?;
    }
    Ok(())
}
pub fn check_f69(d1: &types_api::WordOptVec, d2: &types_api2::WordOptVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?.map(|f| f.into());
        check_f58(&dd1, &dd2)?;
    }
    Ok(())
}
pub fn check_f70(d1: &types_api::WordsOptVec, d2: &types_api2::WordsOptVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?.map(|f| f.into());
        check_f62(&dd1, &dd2)?;
    }
    Ok(())
}
pub fn check_f71(d1: &types_api::BytesOptVec, d2: &types_api2::BytesOptVec) -> ResCheckErr {
    TypesCheckErr::check_length(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?.map(|f| f.try_into().unwrap());
        check_f61(&dd1, &dd2)?;
    }
    Ok(())
}
pub fn check_f72(d1: &types_api::UnionA, d2: &types_api2::UnionA) -> ResCheckErr {
    match d1.to_enum() {
        types_api::UnionAUnion::Byte(v) => {
            let v2 = d2.as_byte()?;
            TypesCheckErr::check_1_data(&v, &v2.into())?;
        }
        types_api::UnionAUnion::Word(v) => {
            let v2 = d2.as_word()?;
            check_f16(&v, &v2.into())?;
        }
        types_api::UnionAUnion::StructA(v) => {
            let v2 = d2.as_struct_a()?;
            check_f28(&v, &v2)?;
        }
        types_api::UnionAUnion::Bytes(v) => {
            let v2 = d2.as_bytes()?;
            check_f41(&v, &v2.try_into().unwrap())?;
        }
        types_api::UnionAUnion::Words(v) => {
            let v2 = d2.as_words()?;
            check_f42(&v, &v2)?;
        }
        types_api::UnionAUnion::Table0(v) => {
            let v2 = d2.as_table0()?;
            check_f50(&v, &v2)?;
        }
        types_api::UnionAUnion::Table6(v) => {
            let v2 = d2.as_table6()?;
            check_f56(&v, &v2)?;
        }
        types_api::UnionAUnion::Table6Opt(v) => {
            let v2 = d2.as_table6_opt()?;
            check_f66(&v.to_opt(), &v2)?;
        }
    };

    Ok(())
}
pub fn check_f73(d1: &types_api::TableA, d2: &types_api2::TableA) -> ResCheckErr {
    check_f17(&d1.f1(), &d2.f1()?.into())?;
    check_f28(&d1.f2(), &d2.f2()?.into())?;
    check_f41(&d1.f3(), &d2.f3()?.try_into().unwrap())?;
    check_f48(&d1.f4(), &d2.f4()?)?;
    check_f51(&d1.f5(), &d2.f5()?)?;
    check_f61(&d1.f6().to_opt(), &d2.f6()?.map(|f| f.try_into().unwrap()))?;
    check_f72(&d1.f7(), &d2.f7()?)?;
    Ok(())
}
pub fn check_f74(d1: &types_api::TableB, d2: &types_api2::TableB) -> ResCheckErr {
    check_f0(&d1.f1().nth0(), &d2.f1()?.into())?;
    check_f0(&d1.f2().nth0(), &(d2.f2()? as u8).into())?;

    TypesCheckErr::check_1_data(
        &u16::from_le_bytes(d1.f3().raw_data().to_vec().try_into().unwrap()),
        &d2.f3()?,
    )?;
    TypesCheckErr::check_1_data(
        &i16::from_le_bytes(d1.f4().raw_data().to_vec().try_into().unwrap()),
        &d2.f4()?,
    )?;

    TypesCheckErr::check_1_data(
        &u32::from_le_bytes(d1.f5().raw_data().to_vec().try_into().unwrap()),
        &d2.f5()?,
    )?;

    TypesCheckErr::check_1_data(
        &i32::from_le_bytes(d1.f6().raw_data().to_vec().try_into().unwrap()),
        &d2.f6()?,
    )?;

    TypesCheckErr::check_1_data(
        &u64::from_le_bytes(d1.f7().raw_data().to_vec().try_into().unwrap()),
        &d2.f7()?,
    )?;

    TypesCheckErr::check_1_data(
        &i64::from_le_bytes(d1.f8().raw_data().to_vec().try_into().unwrap()),
        &d2.f8()?,
    )?;

    Ok(())
}
