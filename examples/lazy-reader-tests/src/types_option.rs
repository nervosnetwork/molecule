use super::{
    BaseTypes, ResCheckErr, TypesArrayWord, TypesCheckErr, TypesConfig, TypesStructA, TypesStructP,
    TypesTable0, TypesTable6, TypesVec,
};
use crate::{types_api, types_api2};
use molecule::lazy_reader::Cursor;
use molecule::prelude::{Builder, Entity};
use rand::{rngs::ThreadRng, thread_rng, Rng};

pub struct TypesOption<T> {
    d: Option<T>,
}
impl<T: BaseTypes> BaseTypes for TypesOption<T> {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let fill = if config.min_size {
            false
        } else {
            match config.option_fill {
                super::OptionFillType::FillRand => rng.gen(),
                super::OptionFillType::FillSome => true,
                super::OptionFillType::FillNone => false,
            }
        };
        if fill {
            Self {
                d: Some(T::new_rng(rng, config)),
            }
        } else {
            Self { d: None }
        }
    }
}
impl<T: BaseTypes> Default for TypesOption<T> {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}

impl<T> TypesOption<T> {
    pub fn new_none() -> Self {
        Self { d: None }
    }
}

impl TypesOption<u8> {
    pub fn to_mol(&self) -> types_api::ByteOpt {
        types_api::ByteOpt::new_builder()
            .set(self.d.map(|f| f.into()))
            .build()
    }

    pub fn check(&self, d: &Option<u8>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        TypesCheckErr::check_1_data(d.as_ref().unwrap(), self.d.as_ref().unwrap())
    }
}
impl TypesOption<TypesArrayWord> {
    pub fn to_mol(&self) -> types_api::WordOpt {
        types_api::WordOpt::new_builder()
            .set(self.d.map(|f| f.to_mol2()))
            .build()
    }

    pub fn check(&self, d: &Option<Cursor>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d
            .as_ref()
            .unwrap()
            .check2(&d.as_ref().unwrap().clone().into())
    }
}
impl TypesOption<TypesStructA> {
    pub fn to_mol(&self) -> types_api::StructAOpt {
        types_api::StructAOpt::new_builder()
            .set(self.d.map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::StructA>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
impl TypesOption<TypesStructP> {
    pub fn to_mol(&self) -> types_api::StructPOpt {
        types_api::StructPOpt::new_builder()
            .set(self.d.map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::StructP>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
impl TypesOption<TypesVec<u8>> {
    pub fn to_mol(&self) -> types_api::BytesOpt {
        types_api::BytesOpt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<Cursor>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d
            .as_ref()
            .unwrap()
            .check(&d.as_ref().unwrap().clone().try_into().unwrap())
    }
}
impl TypesOption<TypesVec<TypesArrayWord>> {
    pub fn to_mol(&self) -> types_api::WordsOpt {
        types_api::WordsOpt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::Words>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(&d.as_ref().unwrap())
    }
}
impl TypesOption<TypesVec<TypesVec<u8>>> {
    pub fn to_mol(&self) -> types_api::BytesVecOpt {
        types_api::BytesVecOpt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::BytesVec>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
impl TypesOption<TypesVec<TypesVec<TypesArrayWord>>> {
    pub fn to_mol(&self) -> types_api::WordsVecOpt {
        types_api::WordsVecOpt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::WordsVec>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
impl TypesOption<TypesTable0> {
    pub fn to_mol(&self) -> types_api::Table0Opt {
        types_api::Table0Opt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::Table0>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
impl TypesOption<TypesTable6> {
    pub fn to_mol(&self) -> types_api::Table6Opt {
        types_api::Table6Opt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<types_api2::Table6>) -> ResCheckErr {
        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
impl TypesOption<TypesOption<TypesTable6>> {
    pub fn to_mol(&self) -> types_api::Table6OptOpt {
        types_api::Table6OptOpt::new_builder()
            .set(self.d.as_ref().map(|f| f.to_mol()))
            .build()
    }

    pub fn check(&self, d: &Option<Option<types_api2::Table6>>) -> ResCheckErr {
        let f1_flag = d.is_some() && d.as_ref().is_none();
        let f2_flag = self.d.is_some() && self.d.as_ref().is_none();
        if f1_flag != f2_flag {
            return Err(TypesCheckErr::Opt(format!(
                "different option: {:?}  {:?}",
                f1_flag, f2_flag
            )));
        }
        if !f1_flag {
            return Ok(());
        }

        TypesCheckErr::check_option(d, &self.d)?;
        if d.is_none() {
            return Ok(());
        }
        self.d.as_ref().unwrap().check(d.as_ref().unwrap())
    }
}
