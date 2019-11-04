use std::io;

use crate::ast::verified::{self as ast, HasName};

pub(super) const API_DECORATOR: &str = "MOLECULE_API_DECORATOR";

macro_rules! w {
    ($writer:ident, $( $args:tt )*) => {
        writeln!($writer, "{}", format!($( $args )*).trim_end())?;
    }
}

pub(super) trait IdentPrefix: HasName {
    fn reader_prefix(&self) -> String {
        format!("MolReader_{}", self.name())
    }

    fn builder_prefix(&self) -> String {
        format!("MolBuilder_{}", self.name())
    }

    fn default_constant(&self) -> String {
        format!("MolDefault_{}", self.name())
    }

    fn api_decorator(&self) -> &str {
        API_DECORATOR
    }

    fn define_reader_macro<W: io::Write>(
        &self,
        writer: &mut W,
        macro_sig_tail: &str,
        macro_content: &str,
    ) -> io::Result<()> {
        self.define_macro(true, writer, macro_sig_tail, macro_content)
    }

    fn define_builder_macro<W: io::Write>(
        &self,
        writer: &mut W,
        macro_sig_tail: &str,
        macro_content: &str,
    ) -> io::Result<()> {
        self.define_macro(false, writer, macro_sig_tail, macro_content)
    }

    fn define_reader_function<W: io::Write>(
        &self,
        writer: &mut W,
        func_sig_tail: &str,
        func_args: &str,
        func_ret: &str,
    ) -> io::Result<()> {
        self.define_function(true, writer, func_sig_tail, func_args, func_ret)
    }

    fn define_builder_function<W: io::Write>(
        &self,
        writer: &mut W,
        func_sig_tail: &str,
        func_args: &str,
        func_ret: &str,
    ) -> io::Result<()> {
        self.define_function(false, writer, func_sig_tail, func_args, func_ret)
    }

    fn define_macro<W: io::Write>(
        &self,
        is_reader: bool,
        writer: &mut W,
        macro_sig_tail: &str,
        macro_content: &str,
    ) -> io::Result<()> {
        let prefix = if is_reader {
            self.reader_prefix()
        } else {
            self.builder_prefix()
        };
        let macro_sig = format!("{}{}", prefix, macro_sig_tail);
        writeln!(
            writer,
            "{:39} {:47} {}",
            "#define", macro_sig, macro_content
        )
    }

    fn define_function<W: io::Write>(
        &self,
        is_reader: bool,
        writer: &mut W,
        func_sig_tail: &str,
        func_args: &str,
        func_ret: &str,
    ) -> io::Result<()> {
        let prefix = if is_reader {
            self.reader_prefix()
        } else {
            self.builder_prefix()
        };
        let func_name = format!("{}{}", prefix, func_sig_tail);
        writeln!(
            writer,
            "{:23} {:15} {:47} {};",
            self.api_decorator(),
            func_ret,
            func_name,
            func_args
        )
    }
}

impl IdentPrefix for ast::Option_ {}

impl IdentPrefix for ast::Union {}

impl IdentPrefix for ast::Array {}

impl IdentPrefix for ast::Struct {}

impl IdentPrefix for ast::FixVec {}

impl IdentPrefix for ast::DynVec {}

impl IdentPrefix for ast::Table {}

impl IdentPrefix for ast::TopDecl {}
