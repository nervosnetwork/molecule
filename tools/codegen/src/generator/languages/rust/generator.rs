use std::io;

use super::{
    builder::GenBuilder, entity::GenEntity, enumerator::GenEnumerator, iterator::GenIterator,
    reader::GenReader,
};
use crate::ast;

pub(super) trait Generator {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
}

impl Generator for ast::Option_ {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        writeln!(writer, "{}", self.gen_from())?;
        Ok(())
    }
}

impl Generator for ast::Union {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        writeln!(writer, "{}", self.gen_enumerator())?;
        writeln!(writer, "{}", self.gen_from())?;
        Ok(())
    }
}

impl Generator for ast::Array {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        writeln!(writer, "{}", self.gen_from())?;
        Ok(())
    }
}

impl Generator for ast::Struct {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        Ok(())
    }
}

impl Generator for ast::FixVec {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        writeln!(writer, "{}", self.gen_iterator())?;
        writeln!(writer, "{}", self.gen_from_iter())?;
        Ok(())
    }
}

impl Generator for ast::DynVec {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        writeln!(writer, "{}", self.gen_iterator())?;
        writeln!(writer, "{}", self.gen_from_iter())?;
        Ok(())
    }
}

impl Generator for ast::Table {
    fn generate<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "{}", self.gen_entity())?;
        writeln!(writer, "{}", self.gen_reader())?;
        writeln!(writer, "{}", self.gen_builder())?;
        Ok(())
    }
}
