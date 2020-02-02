use pest_derive::Parser as ParserDerive;

#[derive(ParserDerive)]
#[grammar = "grammar.pest"]
pub(crate) struct Parser;
