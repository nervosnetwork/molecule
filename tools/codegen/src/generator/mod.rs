use crate::{Ast, Parser};

#[cfg(feature = "lang-c")]
mod lang_c;
#[cfg(feature = "lang-rust")]
mod lang_rust;

#[derive(Debug)]
pub struct Generator {
    ast: Ast,
}

impl Generator {
    pub fn new(input: &str) -> Self {
        let ast = Parser::parse(input);
        Self { ast }
    }
}
