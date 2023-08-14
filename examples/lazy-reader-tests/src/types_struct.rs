use super::*;
use crate::{types_api, types_api2};
use molecule::prelude::{Builder, Entity};
use rand::rngs::ThreadRng;

#[derive(Clone, Copy)]
pub struct TypesStructBytes<T1: BaseTypes, T2: BaseTypes, T3: BaseTypes, T4: BaseTypes> {
    pub f1: T1,
    pub f2: T2,
    pub f3: T3,
    pub f4: T4,
}

impl<T1: BaseTypes, T2: BaseTypes, T3: BaseTypes, T4: BaseTypes> BaseTypes
    for TypesStructBytes<T1, T2, T3, T4>
{
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        Self {
            f1: T1::new_rng(rng, config),
            f2: T2::new_rng(rng, config),
            f3: T3::new_rng(rng, config),
            f4: T4::new_rng(rng, config),
        }
    }
}

impl<T1: BaseTypes, T2: BaseTypes, T3: BaseTypes, T4: BaseTypes> Default
    for TypesStructBytes<T1, T2, T3, T4>
{
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let config = TypesConfig::default();
        Self {
            f1: T1::new_rng(&mut rng, &config),
            f2: T2::new_rng(&mut rng, &config),
            f3: T3::new_rng(&mut rng, &config),
            f4: T4::new_rng(&mut rng, &config),
        }
    }
}

pub type TypesStructA =
    TypesStructBytes<TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 2>, TypesArray<u8, 2>>;
impl TypesStructA {
    pub fn to_mol(&self) -> types_api::StructA {
        types_api::StructA::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructA) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}
pub type TypesStructB =
    TypesStructBytes<TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 2>, TypesArray<u8, 3>>;
impl TypesStructB {
    pub fn to_mol(&self) -> types_api::StructB {
        types_api::StructB::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructB) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}

pub type TypesStructC =
    TypesStructBytes<TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 2>, TypesArray<u8, 4>>;
impl TypesStructC {
    pub fn to_mol(&self) -> types_api::StructC {
        types_api::StructC::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructC) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}

pub type TypesStructD =
    TypesStructBytes<TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 2>, TypesArray<u8, 5>>;
impl TypesStructD {
    pub fn to_mol(&self) -> types_api::StructD {
        types_api::StructD::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructD) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}

pub type TypesStructE =
    TypesStructBytes<TypesArray<u8, 1>, TypesArray<u8, 2>, TypesArray<u8, 1>, TypesArray<u8, 2>>;
impl TypesStructE {
    pub fn to_mol(&self) -> types_api::StructE {
        types_api::StructE::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructE) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}

pub type TypesStructF =
    TypesStructBytes<TypesArray<u8, 1>, TypesArray<u8, 3>, TypesArray<u8, 1>, TypesArray<u8, 1>>;
impl TypesStructF {
    pub fn to_mol(&self) -> types_api::StructF {
        types_api::StructF::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructF) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        Ok(())
    }
}

pub type TypesStructG = TypesStructBytes<
    TypesArray<u8, 3>,
    TypesArray<u8, 1>,
    TypesArray<u8, 2>,
    TypesArray<TypesArrayWord, 2>,
>;
impl TypesStructG {
    pub fn to_mol(&self) -> types_api::StructG {
        types_api::StructG::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructG) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}

pub type TypesStructH =
    TypesStructBytes<TypesArray<u8, 3>, TypesArray<u8, 1>, TypesArray<u8, 2>, TypesArray<u8, 4>>;
impl TypesStructH {
    pub fn to_mol(&self) -> types_api::StructH {
        types_api::StructH::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .f3(self.f3.to_mol())
            .f4(self.f4.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructH) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        self.f3.check(&d.f3()?.into())?;
        self.f4.check(&d.f4()?.into())?;
        Ok(())
    }
}

pub type TypesStructI =
    TypesStructBytes<TypesArray<u8, 3>, TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 1>>;
impl TypesStructI {
    pub fn to_mol(&self) -> types_api::StructI {
        types_api::StructI::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructI) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        Ok(())
    }
}

pub type TypesStructJ =
    TypesStructBytes<TypesArray<u8, 6>, TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 1>>;
impl TypesStructJ {
    pub fn to_mol(&self) -> types_api::StructJ {
        types_api::StructJ::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructJ) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        Ok(())
    }
}

pub type TypesStructIx3 = TypesArray<TypesStructI, 3>;
impl TypesStructIx3 {
    pub fn to_mol(&self) -> types_api::StructIx3 {
        types_api::StructIx3::new_builder()
            .set(self.d.clone().map(|f| f.to_mol()))
            .build()
    }
    pub fn check(&self, d: &types_api2::StructIx3) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            let dd = d.get(i)?;
            self.d[i].check(&dd)?;
        }
        Ok(())
    }
}

pub type TypesStructO =
    TypesStructBytes<TypesStructIx3, TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 1>>;
impl TypesStructO {
    pub fn to_mol(&self) -> types_api::StructO {
        types_api::StructO::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructO) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        Ok(())
    }
}

pub type TypesStructP =
    TypesStructBytes<TypesStructJ, TypesArray<u8, 1>, TypesArray<u8, 1>, TypesArray<u8, 1>>;
impl TypesStructP {
    pub fn to_mol(&self) -> types_api::StructP {
        types_api::StructP::new_builder()
            .f1(self.f1.to_mol())
            .f2(self.f2.to_mol())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructP) -> ResCheckErr {
        self.f1.check(&d.f1()?.into())?;
        self.f2.check(&d.f2()?.into())?;
        Ok(())
    }
}
