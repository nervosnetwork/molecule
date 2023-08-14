use super::*;
use crate::{types_api, types_api2};
use molecule::prelude::{Builder, Entity};
use rand::{rngs::ThreadRng, thread_rng, Rng};

pub struct TypesVec<T: BaseTypes> {
    pub d: Vec<T>,
}

impl<T: BaseTypes> BaseTypes for TypesVec<T> {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let mut d = Vec::new();

        let size = if config.large_vec {
            rng.gen_range(128..1024)
        } else {
            rng.gen_range(1..128)
        };
        for _i in 0..size {
            d.push(T::new_rng(rng, config));
        }

        Self { d }
    }
}
impl<T: BaseTypes> Default for TypesVec<T> {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}

impl TypesVec<u8> {
    pub fn to_mol(&self) -> types_api::Bytes {
        types_api::Bytes::new_builder()
            .set(self.d.iter().map(|f| f.clone().into()).collect())
            .build()
    }
    pub fn check(&self, d: &Vec<u8>) -> ResCheckErr {
        TypesCheckErr::check_data(d, &self.d)
    }
}
impl TypesVec<TypesArrayWord> {
    pub fn to_mol(&self) -> types_api::Words {
        types_api::Words::new_builder()
            .set(self.d.iter().map(|f| f.to_mol2()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::Words) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesArray<u8, 3>> {
    pub fn to_mol(&self) -> types_api::Byte3Vec {
        types_api::Byte3Vec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte3Vec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesArray<u8, 7>> {
    pub fn to_mol(&self) -> types_api::Byte7Vec {
        types_api::Byte7Vec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::Byte7Vec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesStructI> {
    pub fn to_mol(&self) -> types_api::StructIVec {
        types_api::StructIVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructIVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?)?;
        }
        Ok(())
    }
}
impl TypesVec<TypesStructJ> {
    pub fn to_mol(&self) -> types_api::StructJVec {
        types_api::StructJVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructJVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?)?;
        }
        Ok(())
    }
}
impl TypesVec<TypesStructP> {
    pub fn to_mol(&self) -> types_api::StructPVec {
        types_api::StructPVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::StructPVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?)?;
        }
        Ok(())
    }
}
impl TypesVec<TypesVec<u8>> {
    pub fn to_mol(&self) -> types_api::BytesVec {
        types_api::BytesVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::BytesVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.try_into().unwrap())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesVec<TypesArrayWord>> {
    pub fn to_mol(&self) -> types_api::WordsVec {
        types_api::WordsVec::new_builder()
            .set(
                self.d
                    .iter()
                    .map(|f| {
                        types_api::Words::new_builder()
                            .set(f.d.iter().map(|ff| ff.to_mol2()).collect())
                            .build()
                    })
                    .collect(),
            )
            .build()
    }
    pub fn check(&self, d: &types_api2::WordsVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesOption<u8>> {
    pub fn to_mol(&self) -> types_api::ByteOptVec {
        types_api::ByteOptVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::ByteOptVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesOption<TypesArrayWord>> {
    pub fn to_mol(&self) -> types_api::WordOptVec {
        types_api::WordOptVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::WordOptVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesOption<TypesVec<TypesArrayWord>>> {
    pub fn to_mol(&self) -> types_api::WordsOptVec {
        types_api::WordsOptVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::WordsOptVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
impl TypesVec<TypesOption<TypesVec<u8>>> {
    pub fn to_mol(&self) -> types_api::BytesOptVec {
        types_api::BytesOptVec::new_builder()
            .set(self.d.iter().map(|f| f.to_mol()).collect())
            .build()
    }
    pub fn check(&self, d: &types_api2::BytesOptVec) -> ResCheckErr {
        TypesCheckErr::check_lenght(d.len()?, self.d.len())?;
        for i in 0..d.len()? {
            self.d[i].check(&d.get(i)?.into())?;
        }
        Ok(())
    }
}
