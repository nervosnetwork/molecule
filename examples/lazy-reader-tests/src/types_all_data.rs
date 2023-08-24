#![allow(unused_imports)]
// #![allow(dead_code)]

use crate::types_api;
use crate::types_api2;

use core::convert::TryInto;
use lazy_static::lazy_static;
use molecule::lazy_reader::Cursor;
use molecule::prelude::{Builder, Entity, Reader};
use molecule::{bytes::Bytes, prelude::Byte};
use rand::{random, rngs::ThreadRng, thread_rng, Rng, RngCore};

use super::*;

#[derive(Default)]
pub struct TypesAll {
    f0: TypesArray<u8, 1>,
    f1: TypesArray<u8, 2>,
    f2: TypesArray<u8, 3>,
    f3: TypesArray<u8, 4>,
    f4: TypesArray<u8, 5>,
    f5: TypesArray<u8, 6>,
    f6: TypesArray<u8, 7>,
    f7: TypesArray<u8, 8>,
    f8: TypesArray<u8, 9>,
    f9: TypesArray<u8, 10>,
    f10: TypesArray<u8, 11>,
    f11: TypesArray<u8, 12>,
    f12: TypesArray<u8, 13>,
    f13: TypesArray<u8, 14>,
    f14: TypesArray<u8, 15>,
    f15: TypesArray<u8, 16>,

    f16: TypesArrayWord,
    f17: TypesArray<TypesArrayWord, 2>,
    f18: TypesArray<TypesArrayWord, 3>,
    f19: TypesArray<TypesArrayWord, 4>,
    f20: TypesArray<TypesArrayWord, 5>,
    f21: TypesArray<TypesArrayWord, 6>,
    f22: TypesArray<TypesArrayWord, 7>,
    f23: TypesArray<TypesArrayWord, 8>,

    f24: TypesArray<TypesArray<u8, 3>, 3>,
    f25: TypesArray<TypesArray<u8, 5>, 3>,
    f26: TypesArray<TypesArray<u8, 7>, 3>,
    f27: TypesArray<TypesArray<u8, 9>, 3>,

    f28: TypesStructA,
    f29: TypesStructB,
    f30: TypesStructC,
    f31: TypesStructD,
    f32: TypesStructE,
    f33: TypesStructF,
    f34: TypesStructG,
    f35: TypesStructH,
    f36: TypesStructI,
    f37: TypesStructJ,
    f38: TypesStructIx3,
    f39: TypesStructO,
    f40: TypesStructP,

    f41: TypesVec<u8>,
    f42: TypesVec<TypesArrayWord>,
    f43: TypesVec<TypesArray<u8, 3>>,
    f44: TypesVec<TypesArray<u8, 7>>,
    f45: TypesVec<TypesStructI>,
    f46: TypesVec<TypesStructJ>,
    f47: TypesVec<TypesStructP>,
    f48: TypesVec<TypesVec<u8>>,
    f49: TypesVec<TypesVec<TypesArrayWord>>,

    f50: TypesTable0,
    f51: TypesTable1,
    f52: TypesTable2,
    f53: TypesTable3,
    f54: TypesTable4,
    f55: TypesTable5,
    f56: TypesTable6,

    f57: TypesOption<u8>,
    f58: TypesOption<TypesArrayWord>,
    f59: TypesOption<TypesStructA>,
    f60: TypesOption<TypesStructP>,
    f61: TypesOption<TypesVec<u8>>,
    f62: TypesOption<TypesVec<TypesArrayWord>>,
    f63: TypesOption<TypesVec<TypesVec<u8>>>,
    f64: TypesOption<TypesVec<TypesVec<TypesArrayWord>>>,
    f65: TypesOption<TypesTable0>,
    f66: TypesOption<TypesTable6>,
    f67: TypesOption<TypesOption<TypesTable6>>,

    f68: TypesVec<TypesOption<u8>>,
    f69: TypesVec<TypesOption<TypesArrayWord>>,
    f70: TypesVec<TypesOption<TypesVec<TypesArrayWord>>>,
    f71: TypesVec<TypesOption<TypesVec<u8>>>,

    f72: TypesUnionA,
    f73: TypesTableA,
    f74: TypesTableB,
}

impl TypesAll {
    pub fn new_by_config(config: &TypesConfig) -> Self {
        let mut rng = thread_rng();
        Self {
            f0: TypesArray::new_rng(&mut rng, config),
            f1: TypesArray::new_rng(&mut rng, config),
            f2: TypesArray::new_rng(&mut rng, config),
            f3: TypesArray::new_rng(&mut rng, config),
            f4: TypesArray::new_rng(&mut rng, config),
            f5: TypesArray::new_rng(&mut rng, config),
            f6: TypesArray::new_rng(&mut rng, config),
            f7: TypesArray::new_rng(&mut rng, config),
            f8: TypesArray::new_rng(&mut rng, config),
            f9: TypesArray::new_rng(&mut rng, config),
            f10: TypesArray::new_rng(&mut rng, config),
            f11: TypesArray::new_rng(&mut rng, config),
            f12: TypesArray::new_rng(&mut rng, config),
            f13: TypesArray::new_rng(&mut rng, config),
            f14: TypesArray::new_rng(&mut rng, config),
            f15: TypesArray::new_rng(&mut rng, config),
            f16: TypesArray::new_rng(&mut rng, config),
            f17: TypesArray::new_rng(&mut rng, config),
            f18: TypesArray::new_rng(&mut rng, config),
            f19: TypesArray::new_rng(&mut rng, config),
            f20: TypesArray::new_rng(&mut rng, config),
            f21: TypesArray::new_rng(&mut rng, config),
            f22: TypesArray::new_rng(&mut rng, config),
            f23: TypesArray::new_rng(&mut rng, config),
            f24: TypesArray::new_rng(&mut rng, config),
            f25: TypesArray::new_rng(&mut rng, config),
            f26: TypesArray::new_rng(&mut rng, config),
            f27: TypesArray::new_rng(&mut rng, config),
            f28: TypesStructA::new_rng(&mut rng, config),
            f29: TypesStructB::new_rng(&mut rng, config),
            f30: TypesStructC::new_rng(&mut rng, config),
            f31: TypesStructD::new_rng(&mut rng, config),
            f32: TypesStructE::new_rng(&mut rng, config),
            f33: TypesStructF::new_rng(&mut rng, config),
            f34: TypesStructG::new_rng(&mut rng, config),
            f35: TypesStructH::new_rng(&mut rng, config),
            f36: TypesStructI::new_rng(&mut rng, config),
            f37: TypesStructJ::new_rng(&mut rng, config),
            f38: TypesStructIx3::new_rng(&mut rng, config),
            f39: TypesStructO::new_rng(&mut rng, config),
            f40: TypesStructP::new_rng(&mut rng, config),
            f41: TypesVec::new_rng(&mut rng, config),
            f42: TypesVec::new_rng(&mut rng, config),
            f43: TypesVec::new_rng(&mut rng, config),
            f44: TypesVec::new_rng(&mut rng, config),
            f45: TypesVec::new_rng(&mut rng, config),
            f46: TypesVec::new_rng(&mut rng, config),
            f47: TypesVec::new_rng(&mut rng, config),
            f48: TypesVec::new_rng(&mut rng, config),
            f49: TypesVec::new_rng(&mut rng, config),
            f50: TypesTable0::new_rng(&mut rng, config),
            f51: TypesTable1::new_rng(&mut rng, config),
            f52: TypesTable2::new_rng(&mut rng, config),
            f53: TypesTable3::new_rng(&mut rng, config),
            f54: TypesTable4::new_rng(&mut rng, config),
            f55: TypesTable5::new_rng(&mut rng, config),
            f56: TypesTable6::new_rng(&mut rng, config),
            f57: TypesOption::new_rng(&mut rng, config),
            f58: TypesOption::new_rng(&mut rng, config),
            f59: TypesOption::new_rng(&mut rng, config),
            f60: TypesOption::new_rng(&mut rng, config),
            f61: TypesOption::new_rng(&mut rng, config),
            f62: TypesOption::new_rng(&mut rng, config),
            f63: TypesOption::new_rng(&mut rng, config),
            f64: TypesOption::new_rng(&mut rng, config),
            f65: TypesOption::new_rng(&mut rng, config),
            f66: TypesOption::new_rng(&mut rng, config),
            f67: TypesOption::new_rng(&mut rng, config),
            f68: TypesVec::new_rng(&mut rng, config),
            f69: TypesVec::new_rng(&mut rng, config),
            f70: TypesVec::new_rng(&mut rng, config),
            f71: TypesVec::new_rng(&mut rng, config),
            f72: TypesUnionA::new_rng(&mut rng, config),
            f73: TypesTableA::new_rng(&mut rng, config),
            f74: TypesTableB::new_rng(&mut rng, config),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        use crate::types_api::*;

        let builder = types_api::AllInOneBuilder::default()
            .f0(self.f0.to_mol())
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .f5(self.f5.to_mol())
            .f6(self.f6.to_mol())
            .f7(self.f7.to_mol())
            .f8(self.f8.to_mol())
            .f9(self.f9.to_mol())
            .f10(self.f10.to_mol())
            .f11(self.f11.to_mol())
            .f12(self.f12.to_mol())
            .f13(self.f13.to_mol())
            .f14(self.f14.to_mol())
            .f15(self.f15.to_mol())
            .f16(
                types_api::Word::new_builder()
                    .set(self.f16.d.map(|f| f.into()))
                    .build(),
            )
            .f17(self.f17.to_mol())
            .f18(self.f18.to_mol())
            .f19(self.f19.to_mol())
            .f20(self.f20.to_mol())
            .f21(self.f21.to_mol())
            .f22(self.f22.to_mol())
            .f23(self.f23.to_mol())
            .f24(self.f24.to_mol())
            .f25(self.f25.to_mol())
            .f26(self.f26.to_mol())
            .f27(self.f27.to_mol())
            .f28(self.f28.to_mol())
            .f29(self.f29.to_mol())
            .f30(self.f30.to_mol())
            .f31(self.f31.to_mol())
            .f32(self.f32.to_mol())
            .f33(self.f33.to_mol())
            .f34(self.f34.to_mol())
            .f35(self.f35.to_mol())
            .f36(self.f36.to_mol())
            .f37(self.f37.to_mol())
            .f38(self.f38.to_mol())
            .f39(self.f39.to_mol())
            .f40(self.f40.to_mol())
            .f41(self.f41.to_mol())
            .f42(self.f42.to_mol())
            .f43(self.f43.to_mol())
            .f44(self.f44.to_mol())
            .f45(self.f45.to_mol())
            .f46(self.f46.to_mol())
            .f47(self.f47.to_mol())
            .f48(self.f48.to_mol())
            .f49(self.f49.to_mol())
            .f50(self.f50.to_mol())
            .f51(self.f51.to_mol())
            .f52(self.f52.to_mol())
            .f53(self.f53.to_mol())
            .f54(self.f54.to_mol())
            .f55(self.f55.to_mol())
            .f56(self.f56.to_mol())
            .f57(self.f57.to_mol())
            .f58(self.f58.to_mol())
            .f59(self.f59.to_mol())
            .f60(self.f60.to_mol())
            .f61(self.f61.to_mol())
            .f62(self.f62.to_mol())
            .f63(self.f63.to_mol())
            .f64(self.f64.to_mol())
            .f65(self.f65.to_mol())
            .f66(self.f66.to_mol())
            .f67(self.f67.to_mol())
            .f68(self.f68.to_mol())
            .f69(self.f69.to_mol())
            .f70(self.f70.to_mol())
            .f71(self.f71.to_mol())
            .f72(self.f72.to_mol())
            .f73(self.f73.to_mol())
            .f74(self.f74.to_mol())
            .build();

        builder.as_reader().as_slice().to_vec()
    }

    pub fn check(&self, data: &[u8]) -> ResCheckErr {
        use crate::types_api2_mol2::Mol2Vec;
        use types_api2::*;

        let cursor = Cursor::new(data.len(), Box::new(data.to_vec()));
        let all_in_one = AllInOne { cursor };

        self.f0
            .check(&all_in_one.f0()?)
            .map_err(|f| f.to(format!("f0:{}", f.as_str())))?;
        self.f1
            .check(&all_in_one.f1()?.into())
            .map_err(|f| f.to(format!("f1:{}", f.as_str())))?;
        self.f2
            .check(&all_in_one.f2()?.into())
            .map_err(|f| f.to(format!("f2:{}", f.as_str())))?;
        self.f3
            .check(&all_in_one.f3()?.into())
            .map_err(|f| f.to(format!("f3:{}", f.as_str())))?;
        self.f4
            .check(&all_in_one.f4()?.into())
            .map_err(|f| f.to(format!("f74:{}", f.as_str())))?;
        self.f5
            .check(&all_in_one.f5()?.into())
            .map_err(|f| f.to(format!("f5:{}", f.as_str())))?;
        self.f6
            .check(&all_in_one.f6()?.into())
            .map_err(|f| f.to(format!("f6:{}", f.as_str())))?;
        self.f7
            .check(&all_in_one.f7()?.into())
            .map_err(|f| f.to(format!("f7:{}", f.as_str())))?;
        self.f8
            .check(&all_in_one.f8()?.into())
            .map_err(|f| f.to(format!("f8:{}", f.as_str())))?;
        self.f9
            .check(&all_in_one.f9()?.into())
            .map_err(|f| f.to(format!("f9:{}", f.as_str())))?;
        self.f10
            .check(&all_in_one.f10()?.into())
            .map_err(|f| f.to(format!("f10:{}", f.as_str())))?;
        self.f11
            .check(&all_in_one.f11()?.into())
            .map_err(|f| f.to(format!("f11:{}", f.as_str())))?;
        self.f12
            .check(&all_in_one.f12()?.into())
            .map_err(|f| f.to(format!("f12:{}", f.as_str())))?;
        self.f13
            .check(&all_in_one.f13()?.into())
            .map_err(|f| f.to(format!("f13:{}", f.as_str())))?;
        self.f14
            .check(&all_in_one.f14()?.into())
            .map_err(|f| f.to(format!("f14:{}", f.as_str())))?;
        self.f15
            .check(&all_in_one.f15()?.into())
            .map_err(|f| f.to(format!("f15:{}", f.as_str())))?;

        self.f16
            .check2(&all_in_one.f16()?.into())
            .map_err(|f| f.to(format!("f16:{}", f.as_str())))?;
        self.f17
            .check(&all_in_one.f17()?.into())
            .map_err(|f| f.to(format!("f17:{}", f.as_str())))?;
        self.f18
            .check(&all_in_one.f18()?.into())
            .map_err(|f| f.to(format!("f18:{}", f.as_str())))?;
        self.f19
            .check(&all_in_one.f19()?.into())
            .map_err(|f| f.to(format!("f19:{}", f.as_str())))?;
        self.f20
            .check(&all_in_one.f20()?.into())
            .map_err(|f| f.to(format!("f20:{}", f.as_str())))?;
        self.f21
            .check(&all_in_one.f21()?.into())
            .map_err(|f| f.to(format!("f21:{}", f.as_str())))?;
        self.f22
            .check(&all_in_one.f22()?.into())
            .map_err(|f| f.to(format!("f22:{}", f.as_str())))?;
        self.f23
            .check(&all_in_one.f23()?.into())
            .map_err(|f| f.to(format!("f23:{}", f.as_str())))?;

        self.f24
            .check(&all_in_one.f24()?.into())
            .map_err(|f| f.to(format!("f24:{}", f.as_str())))?;
        self.f25
            .check(&all_in_one.f25()?.into())
            .map_err(|f| f.to(format!("f25:{}", f.as_str())))?;
        self.f26
            .check(&all_in_one.f26()?.into())
            .map_err(|f| f.to(format!("f26:{}", f.as_str())))?;
        self.f27
            .check(&all_in_one.f27()?.into())
            .map_err(|f| f.to(format!("f27:{}", f.as_str())))?;

        self.f28
            .check(&all_in_one.f28()?.into())
            .map_err(|f| f.to(format!("f28:{}", f.as_str())))?;
        self.f29
            .check(&all_in_one.f29()?.into())
            .map_err(|f| f.to(format!("f29:{}", f.as_str())))?;
        self.f30
            .check(&all_in_one.f30()?.into())
            .map_err(|f| f.to(format!("f30:{}", f.as_str())))?;
        self.f31
            .check(&all_in_one.f31()?.into())
            .map_err(|f| f.to(format!("f31:{}", f.as_str())))?;
        self.f32
            .check(&all_in_one.f32()?.into())
            .map_err(|f| f.to(format!("f32:{}", f.as_str())))?;
        self.f33
            .check(&all_in_one.f33()?.into())
            .map_err(|f| f.to(format!("f33:{}", f.as_str())))?;
        self.f34
            .check(&all_in_one.f34()?.into())
            .map_err(|f| f.to(format!("f34:{}", f.as_str())))?;
        self.f35
            .check(&all_in_one.f35()?.into())
            .map_err(|f| f.to(format!("f35:{}", f.as_str())))?;
        self.f36
            .check(&all_in_one.f36()?.into())
            .map_err(|f| f.to(format!("f36:{}", f.as_str())))?;
        self.f37
            .check(&all_in_one.f37()?.into())
            .map_err(|f| f.to(format!("f37:{}", f.as_str())))?;
        self.f38
            .check(&all_in_one.f38()?.into())
            .map_err(|f| f.to(format!("f38:{}", f.as_str())))?;
        self.f39
            .check(&all_in_one.f39()?.into())
            .map_err(|f| f.to(format!("f39:{}", f.as_str())))?;
        self.f40
            .check(&all_in_one.f40()?.into())
            .map_err(|f| f.to(format!("f40:{}", f.as_str())))?;

        self.f41
            .check(&all_in_one.f41()?.try_into().unwrap())
            .map_err(|f| f.to(format!("f41:{}", f.as_str())))?;
        self.f42
            .check(&all_in_one.f42()?.into())
            .map_err(|f| f.to(format!("f42:{}", f.as_str())))?;
        self.f43
            .check(&all_in_one.f43()?.into())
            .map_err(|f| f.to(format!("f43:{}", f.as_str())))?;
        self.f44
            .check(&all_in_one.f44()?.into())
            .map_err(|f| f.to(format!("f44:{}", f.as_str())))?;
        self.f45
            .check(&all_in_one.f45()?.into())
            .map_err(|f| f.to(format!("f45:{}", f.as_str())))?;
        self.f46
            .check(&all_in_one.f46()?.into())
            .map_err(|f| f.to(format!("f46:{}", f.as_str())))?;
        self.f47
            .check(&all_in_one.f47()?.into())
            .map_err(|f| f.to(format!("f47:{}", f.as_str())))?;
        self.f48
            .check(&all_in_one.f48()?.into())
            .map_err(|f| f.to(format!("f48:{}", f.as_str())))?;
        self.f49
            .check(&all_in_one.f49()?.into())
            .map_err(|f| f.to(format!("f49:{}", f.as_str())))?;

        self.f50
            .check(&all_in_one.f50()?.into())
            .map_err(|f| f.to(format!("f50:{}", f.as_str())))?;
        self.f51
            .check(&all_in_one.f51()?.into())
            .map_err(|f| f.to(format!("f51:{}", f.as_str())))?;
        self.f52
            .check(&all_in_one.f52()?.into())
            .map_err(|f| f.to(format!("f52:{}", f.as_str())))?;
        self.f53
            .check(&all_in_one.f53()?.into())
            .map_err(|f| f.to(format!("f53:{}", f.as_str())))?;
        self.f54
            .check(&all_in_one.f54()?.into())
            .map_err(|f| f.to(format!("f54:{}", f.as_str())))?;
        self.f55
            .check(&all_in_one.f55()?.into())
            .map_err(|f| f.to(format!("f55:{}", f.as_str())))?;
        self.f56
            .check(&all_in_one.f56()?.into())
            .map_err(|f| f.to(format!("f56:{}", f.as_str())))?;
        self.f57
            .check(&all_in_one.f57()?.into())
            .map_err(|f| f.to(format!("f57:{}", f.as_str())))?;
        self.f58
            .check(&all_in_one.f58()?.map(|f| f.into()))
            .map_err(|f| f.to(format!("f58:{}", f.as_str())))?;
        self.f59
            .check(&all_in_one.f59()?)
            .map_err(|f| f.to(format!("f59:{}", f.as_str())))?;
        self.f60
            .check(&all_in_one.f60()?)
            .map_err(|f| f.to(format!("f60:{}", f.as_str())))?;
        self.f61
            .check(&all_in_one.f61()?)
            .map_err(|f| f.to(format!("f61:{}", f.as_str())))?;
        self.f62
            .check(&all_in_one.f62()?)
            .map_err(|f| f.to(format!("f62:{}", f.as_str())))?;
        self.f63
            .check(&all_in_one.f63()?)
            .map_err(|f| f.to(format!("f63:{}", f.as_str())))?;
        self.f64
            .check(&all_in_one.f64()?)
            .map_err(|f| f.to(format!("f64:{}", f.as_str())))?;
        self.f65
            .check(&all_in_one.f65()?)
            .map_err(|f| f.to(format!("f65:{}", f.as_str())))?;
        self.f66
            .check(&all_in_one.f66()?)
            .map_err(|f| f.to(format!("f66:{}", f.as_str())))?;
        self.f67
            .check(&all_in_one.f67()?)
            .map_err(|f| f.to(format!("f67:{}", f.as_str())))?;
        self.f68
            .check(&all_in_one.f68()?)
            .map_err(|f| f.to(format!("f68:{}", f.as_str())))?;
        self.f69
            .check(&all_in_one.f69()?)
            .map_err(|f| f.to(format!("f69:{}", f.as_str())))?;
        self.f70
            .check(&all_in_one.f70()?)
            .map_err(|f| f.to(format!("f70:{}", f.as_str())))?;
        self.f71
            .check(&all_in_one.f71()?)
            .map_err(|f| f.to(format!("f71:{}", f.as_str())))?;
        self.f72
            .check(&all_in_one.f72()?)
            .map_err(|f| f.to(format!("f72:{}", f.as_str())))?;
        self.f73
            .check(&all_in_one.f73()?)
            .map_err(|f| f.to(format!("f73:{}", f.as_str())))?;
        self.f74
            .check(&all_in_one.f74()?)
            .map_err(|f| f.to(format!("f74:{}", f.as_str())))?;

        types_api::AllInOneReader::verify(&data, true).expect("check data");

        check_mol(
            &types_api::AllInOne::new_unchecked(molecule::bytes::Bytes::from(data.to_vec())),
            &all_in_one,
        )
        .map_err(|f| f.to(format!("check mol and mol2 failed: {:?}", f.as_str())))?;

        Ok(())
    }
}

#[test]
fn test_base() {
    let test_data = TypesAll::default();
    let data = test_data.to_bytes();
    test_data.check(&data).expect("test base");
}

#[test]
fn test_opt_all_none() {
    let mut config: TypesConfig = TypesConfig::default();
    config.option_fill = OptionFillType::FillNone;
    let test_data = TypesAll::new_by_config(&config);
    let data = test_data.to_bytes();
    test_data.check(&data).expect("test base");

    config.option_fill = OptionFillType::FillSome;
    let test_data = TypesAll::new_by_config(&config);
    let data = test_data.to_bytes();
    test_data.check(&data).expect("test base");
}

#[test]
fn test_min_bytes() {
    let mut config: TypesConfig = TypesConfig::default();
    config.min_size = true;
    let test_data = TypesAll::new_by_config(&config);
    let data = test_data.to_bytes();
    test_data.check(&data).expect("test base");

    config.option_fill = OptionFillType::FillSome;
    let test_data = TypesAll::new_by_config(&config);
    let data = test_data.to_bytes();
    test_data.check(&data).expect("test base");

    // Min size is 1106
}

#[test]
fn test_large_buf() {
    let mut config = TypesConfig::default();
    config.large_vec = true;
    let test_data = TypesAll::new_by_config(&config);
    let data = test_data.to_bytes();
    test_data.check(&data).expect("test base");
}

#[test]
fn test_iterator() {
    use types_api2::*;

    let test_data = TypesAll::default();
    let data = test_data.to_bytes();
    let cursor = Cursor::new(data.len(), Box::new(data.to_vec()));
    let all_in_one = AllInOne { cursor };
    let f48 = all_in_one.f48().unwrap();

    let mut count: usize = 0;
    for _ in f48.iter() {
        count += 1;
    }
    assert_eq!(count, f48.len().unwrap());

    let mut count: usize = 0;
    let len = f48.len().unwrap();
    for _ in f48 {
        count += 1;
    }
    assert_eq!(count, len);
}

#[test]
fn test_verify() {
    use types_api2::*;

    let test_data = TypesAll::default();
    let data = test_data.to_bytes();
    let cursor = Cursor::new(data.len(), Box::new(data.to_vec()));
    let all_in_one = AllInOne { cursor };
    all_in_one.verify(false).unwrap();
}
