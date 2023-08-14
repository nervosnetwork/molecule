use super::{BaseTypes, ResCheckErr, TypesCheckErr, TypesConfig};
use crate::{types_api, types_api2};
use molecule::prelude::{Builder, Byte, Entity};
use rand::{rngs::ThreadRng, thread_rng};

#[derive(Clone, Copy)]
pub struct TypesArray<T: BaseTypes, const N: usize> {
    pub d: [T; N],
}

impl<T: BaseTypes + Copy, const N: usize> BaseTypes for TypesArray<T, N> {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let mut buf = [T::new_rng(rng, config); N];

        for i in 0..N {
            buf[i] = T::new_rng(rng, config);
        }

        Self { d: buf }
    }
}
impl<T: BaseTypes, const N: usize> TypesArray<T, N> {
    pub fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let mut d = Vec::new();
        for _ in 0..N {
            d.push(T::new_rng(rng, config));
        }

        Self {
            d: d.try_into().ok().unwrap(),
        }
    }
}

impl<T: BaseTypes, const N: usize> Default for TypesArray<T, N> {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}

impl TypesArray<u8, 1> {
    pub fn to_mol(&self) -> Byte {
        self.d[0].into()
    }

    pub fn check(&self, d: &u8) -> ResCheckErr {
        TypesCheckErr::check_1_data(d, &self.d[0])
    }
}
impl TypesArray<u8, 2> {
    pub fn to_mol(&self) -> types_api::Byte2 {
        types_api::Byte2::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn to_mol2(&self) -> types_api::Word {
        types_api::Word::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }

    pub fn check(&self, d: &types_api2::Byte2) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
    pub fn check2(&self, d: &types_api2::Word) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 3> {
    pub fn to_mol(&self) -> types_api::Byte3 {
        types_api::Byte3::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }

    pub fn check(&self, d: &types_api2::Byte3) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 4> {
    pub fn to_mol(&self) -> types_api::Byte4 {
        types_api::Byte4::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }

    pub fn check(&self, d: &types_api2::Byte4) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 5> {
    pub fn to_mol(&self) -> types_api::Byte5 {
        types_api::Byte5::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }

    pub fn check(&self, d: &types_api2::Byte5) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 6> {
    pub fn to_mol(&self) -> types_api::Byte6 {
        types_api::Byte6::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }

    pub fn check(&self, d: &types_api2::Byte6) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 7> {
    pub fn to_mol(&self) -> types_api::Byte7 {
        types_api::Byte7::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte7) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 8> {
    pub fn to_mol(&self) -> types_api::Byte8 {
        types_api::Byte8::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte8) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 9> {
    pub fn to_mol(&self) -> types_api::Byte9 {
        types_api::Byte9::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte9) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 10> {
    pub fn to_mol(&self) -> types_api::Byte10 {
        types_api::Byte10::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte10) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 11> {
    pub fn to_mol(&self) -> types_api::Byte11 {
        types_api::Byte11::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte11) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 12> {
    pub fn to_mol(&self) -> types_api::Byte12 {
        types_api::Byte12::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte12) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 13> {
    pub fn to_mol(&self) -> types_api::Byte13 {
        types_api::Byte13::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte13) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 14> {
    pub fn to_mol(&self) -> types_api::Byte14 {
        types_api::Byte14::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte14) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 15> {
    pub fn to_mol(&self) -> types_api::Byte15 {
        types_api::Byte15::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte15) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesArray<u8, 16> {
    pub fn to_mol(&self) -> types_api::Byte16 {
        types_api::Byte16::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte16) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
pub type TypesArrayWord = TypesArray<u8, 2>;
impl TypesArray<TypesArrayWord, 2> {
    pub fn to_mol(&self) -> types_api::Word2 {
        types_api::Word2::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word2) -> ResCheckErr {
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArrayWord, 3> {
    pub fn to_mol(&self) -> types_api::Word3 {
        types_api::Word3::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word3) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArrayWord, 4> {
    pub fn to_mol(&self) -> types_api::Word4 {
        types_api::Word4::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word4) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArrayWord, 5> {
    pub fn to_mol(&self) -> types_api::Word5 {
        types_api::Word5::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word5) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArrayWord, 6> {
    pub fn to_mol(&self) -> types_api::Word6 {
        types_api::Word6::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word6) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArrayWord, 7> {
    pub fn to_mol(&self) -> types_api::Word7 {
        types_api::Word7::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word7) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArrayWord, 8> {
    pub fn to_mol(&self) -> types_api::Word8 {
        types_api::Word8::new_builder()
            .set(self.d.clone().map(|f| f.to_mol2()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Word8) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArray<u8, 3>, 3> {
    pub fn to_mol(&self) -> types_api::Byte3x3 {
        types_api::Byte3x3::new_builder()
            .set(self.d.clone().map(|f| f.to_mol()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte3x3) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArray<u8, 5>, 3> {
    pub fn to_mol(&self) -> types_api::Byte5x3 {
        types_api::Byte5x3::new_builder()
            .set(self.d.clone().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &types_api2::Byte5x3) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArray<u8, 7>, 3> {
    pub fn to_mol(&self) -> types_api::Byte7x3 {
        types_api::Byte7x3::new_builder()
            .set(self.d.clone().map(|f| f.to_mol()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte7x3) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesArray<TypesArray<u8, 9>, 3> {
    pub fn to_mol(&self) -> types_api::Byte9x3 {
        types_api::Byte9x3::new_builder()
            .set(self.d.clone().map(|f| f.to_mol()))
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte9x3) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len(), self.d.len())?;
        for i in 0..d.len() {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
