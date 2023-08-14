use crate::{types_api, types_api2, Mol2Vec, ResCheckErr, TypesCheckErr};
use molecule::prelude::Byte;

pub fn check_mol(d1: &types_api::AllInOne, d2: &types_api2::AllInOne) -> ResCheckErr {
    f0(&d1.f0(), &d2.f0()?.into())?;
    f1(&d1.f1(), &d2.f1()?.into())?;
    f2(&d1.f2(), &d2.f2()?.into())?;
    f3(&d1.f3(), &d2.f3()?.into())?;
    f4(&d1.f4(), &d2.f4()?.into())?;
    f5(&d1.f5(), &d2.f5()?.into())?;
    f6(&d1.f6(), &d2.f6()?.into())?;
    f7(&d1.f7(), &d2.f7()?.into())?;
    f8(&d1.f8(), &d2.f8()?.into())?;
    f9(&d1.f9(), &d2.f9()?.into())?;
    f10(&d1.f10(), &d2.f10()?.into())?;
    f11(&d1.f11(), &d2.f11()?.into())?;
    f12(&d1.f12(), &d2.f12()?.into())?;
    f13(&d1.f13(), &d2.f13()?.into())?;
    f14(&d1.f14(), &d2.f14()?.into())?;
    f15(&d1.f15(), &d2.f15()?.into())?;

    f16(&d1.f16(), &d2.f16()?.into())?;
    f17(&d1.f17(), &d2.f17()?.into())?;
    f18(&d1.f18(), &d2.f18()?.into())?;
    f19(&d1.f19(), &d2.f19()?.into())?;
    f20(&d1.f20(), &d2.f20()?.into())?;
    f21(&d1.f21(), &d2.f21()?.into())?;
    f22(&d1.f22(), &d2.f22()?.into())?;
    f23(&d1.f23(), &d2.f23()?.into())?;

    f24(&d1.f24(), &d2.f24()?.into())?;
    f25(&d1.f25(), &d2.f25()?.into())?;
    f26(&d1.f26(), &d2.f26()?.into())?;
    f27(&d1.f27(), &d2.f27()?.into())?;

    f28(&d1.f28(), &d2.f28()?.into())?;
    f29(&d1.f29(), &d2.f29()?.into())?;
    f30(&d1.f30(), &d2.f30()?.into())?;
    f31(&d1.f31(), &d2.f31()?.into())?;
    f32(&d1.f32(), &d2.f32()?.into())?;
    f33(&d1.f33(), &d2.f33()?.into())?;
    f34(&d1.f34(), &d2.f34()?.into())?;
    f35(&d1.f35(), &d2.f35()?.into())?;
    f36(&d1.f36(), &d2.f36()?.into())?;
    f37(&d1.f37(), &d2.f37()?.into())?;
    f38(&d1.f38(), &d2.f38()?.into())?;
    f39(&d1.f39(), &d2.f39()?.into())?;
    f40(&d1.f40(), &d2.f40()?.into())?;

    f41(&d1.f41(), &d2.f41()?.try_into().unwrap())?;
    f42(&d1.f42(), &d2.f42()?.into())?;
    f43(&d1.f43(), &d2.f43()?.into())?;
    f44(&d1.f44(), &d2.f44()?.into())?;
    f45(&d1.f45(), &d2.f45()?.into())?;
    f46(&d1.f46(), &d2.f46()?.into())?;
    f47(&d1.f47(), &d2.f47()?.into())?;
    f48(&d1.f48(), &d2.f48()?.into())?;
    f49(&d1.f49(), &d2.f49()?.into())?;

    f50(&d1.f50(), &d2.f50()?.into())?;
    f51(&d1.f51(), &d2.f51()?.into())?;
    f52(&d1.f52(), &d2.f52()?.into())?;
    f53(&d1.f53(), &d2.f53()?.into())?;
    f54(&d1.f54(), &d2.f54()?.into())?;
    f55(&d1.f55(), &d2.f55()?.into())?;
    f56(&d1.f56(), &d2.f56()?.into())?;

    f57(&d1.f57().to_opt(), &d2.f57()?)?;
    f58(&d1.f58().to_opt(), &d2.f58()?.map(|f| f.into()))?;
    f59(&d1.f59().to_opt(), &d2.f59()?)?;
    f60(&d1.f60().to_opt(), &d2.f60()?)?;
    f61(
        &d1.f61().to_opt(),
        &d2.f61()?.map(|f| f.try_into().unwrap()),
    )?;
    f62(&d1.f62().to_opt(), &d2.f62()?)?;
    f63(&d1.f63().to_opt(), &d2.f63()?)?;
    f64(&d1.f64().to_opt(), &d2.f64()?)?;
    f65(&d1.f65().to_opt(), &d2.f65()?)?;
    f66(&d1.f66().to_opt(), &d2.f66()?)?;
    f67(&d1.f67().to_opt().map(|f| f.to_opt()), &d2.f67()?)?;

    f68(&d1.f68(), &d2.f68()?)?;
    f69(&d1.f69(), &d2.f69()?)?;
    f70(&d1.f70(), &d2.f70()?)?;
    f71(&d1.f71(), &d2.f71()?)?;

    f72(&d1.f72(), &d2.f72()?)?;
    f73(&d1.f73(), &d2.f73()?)?;

    Ok(())
}

fn f0(d1: &Byte, d2: &Byte) -> ResCheckErr {
    TypesCheckErr::check_1_data(d1, d2)
}
fn f1(d1: &types_api::Byte2, d2: &types_api2::Byte2) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f2(d1: &types_api::Byte3, d2: &types_api2::Byte3) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f3(d1: &types_api::Byte4, d2: &types_api2::Byte4) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f4(d1: &types_api::Byte5, d2: &types_api2::Byte5) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f5(d1: &types_api::Byte6, d2: &types_api2::Byte6) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f6(d1: &types_api::Byte7, d2: &types_api2::Byte7) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f7(d1: &types_api::Byte8, d2: &types_api2::Byte8) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f8(d1: &types_api::Byte9, d2: &types_api2::Byte9) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f9(d1: &types_api::Byte10, d2: &types_api2::Byte10) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f10(d1: &types_api::Byte11, d2: &types_api2::Byte11) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f11(d1: &types_api::Byte12, d2: &types_api2::Byte12) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f12(d1: &types_api::Byte13, d2: &types_api2::Byte13) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f13(d1: &types_api::Byte14, d2: &types_api2::Byte14) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f14(d1: &types_api::Byte15, d2: &types_api2::Byte15) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f15(d1: &types_api::Byte16, d2: &types_api2::Byte16) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f16(d1: &types_api::Word, d2: &types_api2::Word) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f17(d1: &types_api::Word2, d2: &types_api2::Word2) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f18(d1: &types_api::Word3, d2: &types_api2::Word3) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f19(d1: &types_api::Word4, d2: &types_api2::Word4) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f20(d1: &types_api::Word5, d2: &types_api2::Word5) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f21(d1: &types_api::Word6, d2: &types_api2::Word6) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f22(d1: &types_api::Word7, d2: &types_api2::Word7) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f23(d1: &types_api::Word8, d2: &types_api2::Word8) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f24(d1: &types_api::Byte3x3, d2: &types_api2::Byte3x3) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f25(d1: &types_api::Byte5x3, d2: &types_api2::Byte5x3) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f26(d1: &types_api::Byte7x3, d2: &types_api2::Byte7x3) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f27(d1: &types_api::Byte9x3, d2: &types_api2::Byte9x3) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f28(d1: &types_api::StructA, d2: &types_api2::StructA) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;
    f1(&d1.f3(), &d2.f3()?.into())?;
    f1(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f29(d1: &types_api::StructB, d2: &types_api2::StructB) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;
    f1(&d1.f3(), &d2.f3()?.into())?;
    f2(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f30(d1: &types_api::StructC, d2: &types_api2::StructC) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;
    f1(&d1.f3(), &d2.f3()?.into())?;
    f3(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f31(d1: &types_api::StructD, d2: &types_api2::StructD) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;
    f1(&d1.f3(), &d2.f3()?.into())?;
    f4(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f32(d1: &types_api::StructE, d2: &types_api2::StructE) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f1(&d1.f2(), &d2.f2()?.into())?;
    f0(&d1.f3(), &d2.f3()?.into())?;
    f1(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f33(d1: &types_api::StructF, d2: &types_api2::StructF) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f2(&d1.f2(), &d2.f2()?.into())?;
    f0(&d1.f3(), &d2.f3()?.into())?;

    Ok(())
}
fn f34(d1: &types_api::StructG, d2: &types_api2::StructG) -> ResCheckErr {
    f2(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;
    f1(&d1.f3(), &d2.f3()?.into())?;
    f17(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f35(d1: &types_api::StructH, d2: &types_api2::StructH) -> ResCheckErr {
    f2(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;
    f1(&d1.f3(), &d2.f3()?.into())?;
    f3(&d1.f4(), &d2.f4()?.into())?;

    Ok(())
}
fn f36(d1: &types_api::StructI, d2: &types_api2::StructI) -> ResCheckErr {
    f2(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
fn f37(d1: &types_api::StructJ, d2: &types_api2::StructJ) -> ResCheckErr {
    f5(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
fn f38(d1: &types_api::StructIx3, d2: &types_api2::StructIx3) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len())?;
    for i in 0..d1.mol_len()? {
        f36(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }

    Ok(())
}
fn f39(d1: &types_api::StructO, d2: &types_api2::StructO) -> ResCheckErr {
    f38(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
fn f40(d1: &types_api::StructP, d2: &types_api2::StructP) -> ResCheckErr {
    f37(&d1.f1(), &d2.f1()?.into())?;
    f0(&d1.f2(), &d2.f2()?.into())?;

    Ok(())
}
fn f41(d1: &types_api::Bytes, d2: &Vec<u8>) -> ResCheckErr {
    TypesCheckErr::check_mol_data(d1, d2)
}
fn f42(d1: &types_api::Words, d2: &types_api2::Words) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f43(d1: &types_api::Byte3Vec, d2: &types_api2::Byte3Vec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f44(d1: &types_api::Byte7Vec, d2: &types_api2::Byte7Vec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        TypesCheckErr::check_mol_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f45(d1: &types_api::StructIVec, d2: &types_api2::StructIVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        f36(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f46(d1: &types_api::StructJVec, d2: &types_api2::StructJVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        f37(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f47(d1: &types_api::StructPVec, d2: &types_api2::StructPVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        f40(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f48(d1: &types_api::BytesVec, d2: &types_api2::BytesVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        f41(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f49(d1: &types_api::WordsVec, d2: &types_api2::WordsVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        f42(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
    }
    Ok(())
}
fn f50(_d1: &types_api::Table0, _d2: &types_api2::Table0) -> ResCheckErr {
    Ok(())
}
fn f51(d1: &types_api::Table1, d2: &types_api2::Table1) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    Ok(())
}
fn f52(d1: &types_api::Table2, d2: &types_api2::Table2) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f17(&d1.f2(), &d2.f2()?.into())?;
    Ok(())
}
fn f53(d1: &types_api::Table3, d2: &types_api2::Table3) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f17(&d1.f2(), &d2.f2()?.into())?;
    f28(&d1.f3(), &d2.f3()?.into())?;
    Ok(())
}
fn f54(d1: &types_api::Table4, d2: &types_api2::Table4) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f17(&d1.f2(), &d2.f2()?.into())?;
    f28(&d1.f3(), &d2.f3()?.into())?;
    f41(&d1.f4(), &d2.f4()?.try_into().unwrap())?;
    Ok(())
}
fn f55(d1: &types_api::Table5, d2: &types_api2::Table5) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f17(&d1.f2(), &d2.f2()?.into())?;
    f28(&d1.f3(), &d2.f3()?.into())?;
    f41(&d1.f4(), &d2.f4()?.try_into().unwrap())?;
    f48(&d1.f5(), &d2.f5()?.into())?;
    Ok(())
}
fn f56(d1: &types_api::Table6, d2: &types_api2::Table6) -> ResCheckErr {
    f0(&d1.f1(), &d2.f1()?.into())?;
    f17(&d1.f2(), &d2.f2()?.into())?;
    f28(&d1.f3(), &d2.f3()?.into())?;
    f41(&d1.f4(), &d2.f4()?.try_into().unwrap())?;
    f48(&d1.f5(), &d2.f5()?.into())?;
    f55(&d1.f6(), &d2.f6()?.into())?;
    Ok(())
}
fn f57(d1: &Option<Byte>, d2: &Option<u8>) -> ResCheckErr {
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
fn f58(d1: &Option<types_api::Word>, d2: &Option<types_api2::Word>) -> ResCheckErr {
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

    f16(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f59(d1: &Option<types_api::StructA>, d2: &Option<types_api2::StructA>) -> ResCheckErr {
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

    f28(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f60(d1: &Option<types_api::StructP>, d2: &Option<types_api2::StructP>) -> ResCheckErr {
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

    f40(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f61(d1: &Option<types_api::Bytes>, d2: &Option<Vec<u8>>) -> ResCheckErr {
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

    f41(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f62(d1: &Option<types_api::Words>, d2: &Option<types_api2::Words>) -> ResCheckErr {
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

    f42(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f63(d1: &Option<types_api::BytesVec>, d2: &Option<types_api2::BytesVec>) -> ResCheckErr {
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

    f48(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f64(d1: &Option<types_api::WordsVec>, d2: &Option<types_api2::WordsVec>) -> ResCheckErr {
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

    f49(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f65(d1: &Option<types_api::Table0>, d2: &Option<types_api2::Table0>) -> ResCheckErr {
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

    f50(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f66(d1: &Option<types_api::Table6>, d2: &Option<types_api2::Table6>) -> ResCheckErr {
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

    f56(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f67(
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

    f66(d1.as_ref().unwrap(), d2.as_ref().unwrap())?;

    Ok(())
}
fn f68(d1: &types_api::ByteOptVec, d2: &types_api2::ByteOptVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?;
        f57(&dd1, &dd2)?;
    }
    Ok(())
}
fn f69(d1: &types_api::WordOptVec, d2: &types_api2::WordOptVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?.map(|f| f.into());
        f58(&dd1, &dd2)?;
    }
    Ok(())
}
fn f70(d1: &types_api::WordsOptVec, d2: &types_api2::WordsOptVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?.map(|f| f.into());
        f62(&dd1, &dd2)?;
    }
    Ok(())
}
fn f71(d1: &types_api::BytesOptVec, d2: &types_api2::BytesOptVec) -> ResCheckErr {
    TypesCheckErr::check_lenght(d1.mol_len()?, d2.len()?)?;
    for i in 0..d1.mol_len()? {
        let dd1 = d1.mol_get(i)?.to_opt();
        let dd2 = d2.get(i)?.map(|f| f.try_into().unwrap());
        f61(&dd1, &dd2)?;
    }
    Ok(())
}
fn f72(d1: &types_api::UnionA, d2: &types_api2::UnionA) -> ResCheckErr {
    match d1.to_enum() {
        types_api::UnionAUnion::Byte(v) => {
            let v2 = d2.as_byte()?;
            TypesCheckErr::check_1_data(&v, &v2.into())?;
        }
        types_api::UnionAUnion::Word(v) => {
            let v2 = d2.as_word()?;
            f16(&v, &v2.into())?;
        }
        types_api::UnionAUnion::StructA(v) => {
            let v2 = d2.as_structa()?;
            f28(&v, &v2)?;
        }
        types_api::UnionAUnion::Bytes(v) => {
            let v2 = d2.as_bytes()?;
            f41(&v, &v2.try_into().unwrap())?;
        }
        types_api::UnionAUnion::Words(v) => {
            let v2 = d2.as_words()?;
            f42(&v, &v2)?;
        }
        types_api::UnionAUnion::Table0(v) => {
            let v2 = d2.as_table0()?;
            f50(&v, &v2)?;
        }
        types_api::UnionAUnion::Table6(v) => {
            let v2 = d2.as_table6()?;
            f56(&v, &v2)?;
        }
        types_api::UnionAUnion::Table6Opt(v) => {
            let v2 = d2.as_table6opt()?;
            f66(&v.to_opt(), &v2)?;
        }
    };

    Ok(())
}
fn f73(d1: &types_api::TableA, d2: &types_api2::TableA) -> ResCheckErr {
    f17(&d1.f1(), &d2.f1()?.into())?;
    f28(&d1.f2(), &d2.f2()?.into())?;
    f41(&d1.f3(), &d2.f3()?.try_into().unwrap())?;
    f48(&d1.f4(), &d2.f4()?)?;
    f51(&d1.f5(), &d2.f5()?)?;
    f61(&d1.f6().to_opt(), &d2.f6()?.map(|f| f.try_into().unwrap()))?;
    f72(&d1.f7(), &d2.f7()?)?;
    Ok(())
}
