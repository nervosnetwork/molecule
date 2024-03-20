use super::{
    BaseTypes, ResCheckErr, TypesArray, TypesArrayWord, TypesCheckErr, TypesConfig, TypesOption,
    TypesStructA, TypesUnionA, TypesVec,
};
use crate::{types_api, types_api2};
use molecule::prelude::{Builder, Entity};
use rand::{prelude::*, rngs::ThreadRng, thread_rng};

pub struct TypesTable0 {}
impl BaseTypes for TypesTable0 {
    fn new_rng(_rng: &mut ThreadRng, _config: &TypesConfig) -> Self {
        Self {}
    }
}
impl Default for TypesTable0 {
    fn default() -> Self {
        Self {}
    }
}
impl TypesTable0 {
    pub fn to_mol(&self) -> types_api::Table0 {
        types_api::Table0::new_builder().build()
    }
    pub fn check(&self, _d: &types_api2::Table0) -> ResCheckErr {
        // TODO
        Ok(())
    }
}

pub struct TypesTable1 {
    f1: TypesArray<u8, 1>,
}
impl BaseTypes for TypesTable1 {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
        }
    }
}
impl Default for TypesTable1 {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTable1 {
    pub fn to_mol(&self) -> types_api::Table1 {
        types_api::Table1::new_builder()
            .f1(self.f1.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::Table1) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        Ok(())
    }
}

pub struct TypesTable2 {
    f1: TypesArray<u8, 1>,
    f2: TypesArray<TypesArrayWord, 2>,
}
impl BaseTypes for TypesTable2 {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
            f2: TypesArray::new_rng(rng, config),
        }
    }
}
impl Default for TypesTable2 {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTable2 {
    pub fn to_mol(&self) -> types_api::Table2 {
        types_api::Table2::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::Table2) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        self.f2.check(&d.f2()?)?;
        Ok(())
    }
}

pub struct TypesTable3 {
    f1: TypesArray<u8, 1>,
    f2: TypesArray<TypesArrayWord, 2>,
    f3: TypesStructA,
}
impl BaseTypes for TypesTable3 {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
            f2: TypesArray::new_rng(rng, config),
            f3: TypesStructA::new_rng(rng, config),
        }
    }
}
impl Default for TypesTable3 {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTable3 {
    pub fn to_mol(&self) -> types_api::Table3 {
        types_api::Table3::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::Table3) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        self.f2.check(&d.f2()?)?;
        self.f3.check(&d.f3()?)?;
        Ok(())
    }
}

pub struct TypesTable4 {
    f1: TypesArray<u8, 1>,
    f2: TypesArray<TypesArrayWord, 2>,
    f3: TypesStructA,
    f4: TypesVec<u8>,
}
impl BaseTypes for TypesTable4 {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
            f2: TypesArray::new_rng(rng, config),
            f3: TypesStructA::new_rng(rng, config),
            f4: TypesVec::new_rng(rng, config),
        }
    }
}
impl Default for TypesTable4 {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTable4 {
    pub fn to_mol(&self) -> types_api::Table4 {
        types_api::Table4::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::Table4) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        self.f2.check(&d.f2()?)?;
        self.f3.check(&d.f3()?)?;
        self.f4.check(&d.f4()?.try_into().unwrap())?;
        Ok(())
    }
}

pub struct TypesTable5 {
    f1: TypesArray<u8, 1>,
    f2: TypesArray<TypesArrayWord, 2>,
    f3: TypesStructA,
    f4: TypesVec<u8>,
    f5: TypesVec<TypesVec<u8>>,
}
impl BaseTypes for TypesTable5 {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
            f2: TypesArray::new_rng(rng, config),
            f3: TypesStructA::new_rng(rng, config),
            f4: TypesVec::new_rng(rng, config),
            f5: TypesVec::new_rng(rng, config),
        }
    }
}
impl Default for TypesTable5 {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTable5 {
    pub fn to_mol(&self) -> types_api::Table5 {
        types_api::Table5::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .f5(self.f5.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::Table5) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        self.f2.check(&d.f2()?)?;
        self.f3.check(&d.f3()?)?;
        self.f4.check(&d.f4()?.try_into().unwrap())?;
        self.f5.check(&d.f5()?.try_into().unwrap())?;
        Ok(())
    }
}

pub struct TypesTable6 {
    f1: TypesArray<u8, 1>,
    f2: TypesArray<TypesArrayWord, 2>,
    f3: TypesStructA,
    f4: TypesVec<u8>,
    f5: TypesVec<TypesVec<u8>>,
    f6: TypesTable5,
}
impl BaseTypes for TypesTable6 {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
            f2: TypesArray::new_rng(rng, config),
            f3: TypesStructA::new_rng(rng, config),
            f4: TypesVec::new_rng(rng, config),
            f5: TypesVec::new_rng(rng, config),
            f6: TypesTable5::new_rng(rng, config),
        }
    }
}
impl Default for TypesTable6 {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTable6 {
    pub fn to_mol(&self) -> types_api::Table6 {
        types_api::Table6::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .f5(self.f5.to_mol())
            .f6(self.f6.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::Table6) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        self.f2.check(&d.f2()?)?;
        self.f3.check(&d.f3()?)?;
        self.f4.check(&d.f4()?.try_into().unwrap())?;
        self.f5.check(&d.f5()?.try_into().unwrap())?;
        self.f6.check(&d.f6()?.try_into().unwrap())?;
        Ok(())
    }
}
pub struct TypesTableA {
    f1: TypesArray<TypesArrayWord, 2>,
    f2: TypesStructA,
    f3: TypesVec<u8>,
    f4: TypesVec<TypesVec<u8>>,
    f5: TypesTable1,
    f6: TypesOption<TypesVec<u8>>,
    f7: TypesUnionA,
    f8: TypesArray<u8, 1>,
}
impl BaseTypes for TypesTableA {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: TypesArray::new_rng(rng, config),
            f2: TypesStructA::new_rng(rng, config),
            f3: TypesVec::new_rng(rng, config),
            f4: TypesVec::new_rng(rng, config),
            f5: TypesTable1::new_rng(rng, config),
            f6: TypesOption::new_rng(rng, config),
            f7: TypesUnionA::new_rng(rng, config),
            f8: TypesArray::new_rng(rng, config),
        }
    }
}
impl Default for TypesTableA {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTableA {
    pub fn to_mol(&self) -> types_api::TableA {
        types_api::TableA::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .f5(self.f5.to_mol())
            .f6(self.f6.to_mol())
            .f7(self.f7.to_mol())
            .f8(self.f8.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::TableA) -> ResCheckErr {
        self.f1.check(&d.f1()?)?;
        self.f2.check(&d.f2()?)?;
        self.f3.check(&d.f3()?.try_into().unwrap())?;
        self.f4.check(&d.f4()?.try_into().unwrap())?;
        self.f5.check(&d.f5()?.try_into().unwrap())?;
        self.f6.check(&d.f6()?.try_into().unwrap())?;
        Ok(())
    }
}

pub struct TypesTableB {
    f1: u8,
    f2: i8,
    f3: u16,
    f4: i16,
    f5: u32,
    f6: i32,
    f7: u64,
    f8: i64,
}
impl BaseTypes for TypesTableB {
    fn new_rng(rng: &mut ThreadRng, _config: &TypesConfig) -> Self {
        Self {
            f1: rng.gen(),
            f2: rng.gen(),
            f3: rng.gen(),
            f4: rng.gen(),
            f5: rng.gen(),
            f6: rng.gen(),
            f7: rng.gen(),
            f8: rng.gen(),
        }
    }
}
impl Default for TypesTableB {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesTableB {
    pub fn to_mol(&self) -> types_api::TableB {
        types_api::TableB::new_builder()
            .f1(types_api::Uint8::new_builder()
                .set([self.f1.into()])
                .build())
            .f2(types_api::Int8::new_builder()
                .set([(self.f2 as u8).into()])
                .build())
            .f3(types_api::Uint16::new_builder()
                .set(self.f3.to_le_bytes().map(|f| f.into()))
                .build())
            .f4(types_api::Int16::new_builder()
                .set(self.f4.to_le_bytes().map(|f| f.into()))
                .build())
            .f5(types_api::Uint32::new_builder()
                .set(self.f5.to_le_bytes().map(|f| f.into()))
                .build())
            .f6(types_api::Int32::new_builder()
                .set(self.f6.to_le_bytes().map(|f| f.into()))
                .build())
            .f7(types_api::Uint64::new_builder()
                .set(self.f7.to_le_bytes().map(|f| f.into()))
                .build())
            .f8(types_api::Int64::new_builder()
                .set(self.f8.to_le_bytes().map(|f| f.into()))
                .build())
            .build()
    }
    pub fn check(&self, d: &types_api2::TableB) -> ResCheckErr {
        TypesCheckErr::check_1_data(&self.f1, &d.f1()?)?;
        TypesCheckErr::check_1_data(&self.f2, &d.f2()?)?;
        TypesCheckErr::check_1_data(&self.f3, &d.f3()?)?;
        TypesCheckErr::check_1_data(&self.f4, &d.f4()?)?;
        TypesCheckErr::check_1_data(&self.f5, &d.f5()?)?;
        TypesCheckErr::check_1_data(&self.f6, &d.f6()?)?;
        TypesCheckErr::check_1_data(&self.f7, &d.f7()?)?;
        TypesCheckErr::check_1_data(&self.f8, &d.f8()?)?;

        Ok(())
    }
}
