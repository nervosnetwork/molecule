use super::TypesCheckErr;
use crate::types_api;
use crate::types_api2::*;

pub trait Mol2Vec {
    type RetType;
    fn mol_len(&self) -> Result<usize, TypesCheckErr>;
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr>;
}
impl Mol2Vec for Vec<u8> {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self[index])
    }
}
impl Mol2Vec for Byte2 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte3 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte4 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte5 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte6 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte7 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte8 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte9 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte10 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte11 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte12 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte13 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte14 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte15 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Byte16 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Word {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for StructIx3 {
    type RetType = StructI;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Bytes {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for StructIVec {
    type RetType = StructI;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for StructJVec {
    type RetType = StructJ;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for StructPVec {
    type RetType = StructP;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?)
    }
}
impl Mol2Vec for Word2 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Word3 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Word4 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Word5 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Word6 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Word7 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Word8 {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Byte3x3 {
    type RetType = Byte3;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Byte5x3 {
    type RetType = Byte5;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Byte7x3 {
    type RetType = Byte7;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Byte9x3 {
    type RetType = Byte9;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Words {
    type RetType = Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Byte3Vec {
    type RetType = Byte3;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for Byte7Vec {
    type RetType = Byte7;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}
impl Mol2Vec for BytesVec {
    type RetType = Vec<u8>;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index)?.try_into()?)
    }
}
impl Mol2Vec for WordsVec {
    type RetType = Words;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len()?)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(Self::RetType::from(self.get(index)?))
    }
}

impl Mol2Vec for types_api::Byte2 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte3 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte4 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte5 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte6 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte7 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte8 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte9 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte10 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte11 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte12 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte13 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte14 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte15 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Byte16 {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Word {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.raw_data().len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.raw_data().get(index).unwrap().clone())
    }
}
impl Mol2Vec for types_api::Word2 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(2)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Word3 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(3)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Word4 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(4)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            3 => self.nth3(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Word5 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(5)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            3 => self.nth3(),
            4 => self.nth4(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Word6 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(6)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            3 => self.nth3(),
            4 => self.nth4(),
            5 => self.nth5(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Word7 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(7)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            3 => self.nth3(),
            4 => self.nth4(),
            5 => self.nth5(),
            6 => self.nth6(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Word8 {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(8)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            3 => self.nth3(),
            4 => self.nth4(),
            5 => self.nth5(),
            6 => self.nth6(),
            7 => self.nth7(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Byte3x3 {
    type RetType = types_api::Byte3;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(3)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Byte5x3 {
    type RetType = types_api::Byte5;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(3)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Byte7x3 {
    type RetType = types_api::Byte7;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(3)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Byte9x3 {
    type RetType = types_api::Byte9;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(3)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::StructIx3 {
    type RetType = types_api::StructI;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(3)
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(match index {
            0 => self.nth0(),
            1 => self.nth1(),
            2 => self.nth2(),
            _ => panic!("out of bound"),
        })
    }
}
impl Mol2Vec for types_api::Bytes {
    type RetType = u8;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap().into())
    }
}
impl Mol2Vec for types_api::Words {
    type RetType = types_api::Word;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::Byte3Vec {
    type RetType = types_api::Byte3;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::Byte7Vec {
    type RetType = types_api::Byte7;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::StructIVec {
    type RetType = types_api::StructI;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::StructJVec {
    type RetType = types_api::StructJ;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::StructPVec {
    type RetType = types_api::StructP;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::BytesVec {
    type RetType = types_api::Bytes;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::WordsVec {
    type RetType = types_api::Words;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::ByteOptVec {
    type RetType = types_api::ByteOpt;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::WordOptVec {
    type RetType = types_api::WordOpt;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::WordsOptVec {
    type RetType = types_api::WordsOpt;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
impl Mol2Vec for types_api::BytesOptVec {
    type RetType = types_api::BytesOpt;
    fn mol_len(&self) -> Result<usize, TypesCheckErr> {
        Ok(self.len())
    }
    fn mol_get(&self, index: usize) -> Result<Self::RetType, TypesCheckErr> {
        Ok(self.get(index).unwrap())
    }
}
