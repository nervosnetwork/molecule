use std::path::Path;

use crate::{ast, utils::ParserUtils as _};

mod inner;
pub(crate) use inner::{Parser as InnerParser, Rule};

pub struct Parser;

impl Parser {
    pub fn parse<P: AsRef<Path>>(path: &P) -> ast::Ast {
        let ast_raw = Self::preprocess(path).unwrap();
        ast::Ast::complete(ast_raw)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{HasName, TopDecl};
    use crate::*;
    use std::io::Write;

    #[test]
    fn test_parse_custom_union_id() {
        let mut schema_file = tempfile::NamedTempFile::new().unwrap();
        schema_file
            .write_all(
                b"
array a0  [byte;1];
array a1  [byte;1];
array a2  [byte;1];
array a3  [byte;1];
array a4  [byte;1];

union UnionWithoutCustomId {
    a0,
    a1,
    a2,
    a3,
}

union UninoWithFullContinuousCustomIdFrom0 {
    a0 : 0,
    a1 : 1,
    a2 : 2,
    a3 : 3,
}

union UninoWithFullContinuousCustomIdFrom5 {
    a0 : 5,
    a1 : 6,
    a2 : 7,
    a3 : 8,
}

union UninoWithFullDiscontinuousCustomId {
    a0 : 2,
    a1 : 3,
    a2 : 7,
    a3 : 8,
}

union UninoWithPartialCustomId_0 {
    a0 : 3,
    a1,
    a2,
    a3,
}

union UninoWithPartialCustomId_1 {
    a0,
    a1 : 3,
    a2,
    a3,
}

union UninoWithPartialCustomId_2 {
    a0 : 3,
    a1,
    a2 : 5,
    a3,
}

union UninoWithPartialCustomId_Reverse {
    a0 : 5,
    a1,
    a2 : 3,
    a3,
}


",
            )
            .unwrap();
        schema_file.flush().unwrap();

        let ast = Parser::parse(&schema_file.into_temp_path());
        ast.decls().iter().for_each(|decl| {
            if let TopDecl::Union(union) = decl.as_ref() {
                match union.name() {
                    "UnionWithoutCustomId" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 0),
                                "a1" => assert_eq!(union_item_decl.id(), 1),
                                "a2" => assert_eq!(union_item_decl.id(), 2),
                                "a3" => assert_eq!(union_item_decl.id(), 3),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithFullContinuousCustomIdFrom0" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 0),
                                "a1" => assert_eq!(union_item_decl.id(), 1),
                                "a2" => assert_eq!(union_item_decl.id(), 2),
                                "a3" => assert_eq!(union_item_decl.id(), 3),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithFullContinuousCustomIdFrom5" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 5),
                                "a1" => assert_eq!(union_item_decl.id(), 6),
                                "a2" => assert_eq!(union_item_decl.id(), 7),
                                "a3" => assert_eq!(union_item_decl.id(), 8),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithFullDiscontinuousCustomId" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 2),
                                "a1" => assert_eq!(union_item_decl.id(), 3),
                                "a2" => assert_eq!(union_item_decl.id(), 7),
                                "a3" => assert_eq!(union_item_decl.id(), 8),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithPartialCustomId_0" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 3),
                                "a1" => assert_eq!(union_item_decl.id(), 4),
                                "a2" => assert_eq!(union_item_decl.id(), 5),
                                "a3" => assert_eq!(union_item_decl.id(), 6),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithPartialCustomId_1" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 0),
                                "a1" => assert_eq!(union_item_decl.id(), 3),
                                "a2" => assert_eq!(union_item_decl.id(), 4),
                                "a3" => assert_eq!(union_item_decl.id(), 5),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithPartialCustomId_2" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 3),
                                "a1" => assert_eq!(union_item_decl.id(), 4),
                                "a2" => assert_eq!(union_item_decl.id(), 5),
                                "a3" => assert_eq!(union_item_decl.id(), 6),
                                _ => unreachable!(),
                            }
                        }
                    }
                    "UninoWithPartialCustomId_Reverse" => {
                        assert_eq!(union.items().len(), 4);
                        for union_item_decl in union.items() {
                            match union_item_decl.typ().name() {
                                "a0" => assert_eq!(union_item_decl.id(), 5),
                                "a1" => assert_eq!(union_item_decl.id(), 6),
                                "a2" => assert_eq!(union_item_decl.id(), 3),
                                "a3" => assert_eq!(union_item_decl.id(), 4),
                                _ => unreachable!(),
                            }
                        }
                    }

                    _ => unreachable!(),
                }
            }
        });
    }

    #[test]
    fn test_union_items_should_ordered_by_custom_id() {
        let mut schema_file0 = tempfile::NamedTempFile::new().unwrap();
        schema_file0
            .write_all(
                b"
array a0  [byte;1];
array a1  [byte;2];
array a2  [byte;3];
array a3  [byte;4];
union Foo {
    a0 : 1,
    a1,
    a2 : 10,
    a3,
}
",
            )
            .unwrap();

        schema_file0.flush().unwrap();

        let mut schema_file1 = tempfile::NamedTempFile::new().unwrap();
        schema_file1
            .write_all(
                b"
array a0  [byte;1];
array a1  [byte;2];
array a2  [byte;3];
array a3  [byte;4];
union Foo {
    a2 : 10,
    a3,
    a0 : 1,
    a1,
}
",
            )
            .unwrap();

        schema_file1.flush().unwrap();

        let ast0 = Parser::parse(&schema_file0.into_temp_path());
        let ast1 = Parser::parse(&schema_file1.into_temp_path());

        for ast in [ast0, ast1] {
            // get union items
            if let TopDecl::Union(union) = ast
                .decls()
                .iter()
                .find(|decl| decl.name() == "Foo")
                .unwrap()
                .as_ref()
            {
                let custom_ids: Vec<usize> = union.items().iter().map(|item| item.id()).collect();
                assert_eq!(custom_ids, vec![1, 2, 10, 11]);
            }
        }
    }

    #[should_panic]
    #[test]
    fn test_bad_explicit_duplicate_union_schema() {
        let mut schema_file = tempfile::NamedTempFile::new().unwrap();
        schema_file
            .write_all(
                b"
array a0  [byte;1];
array a1  [byte;1];
array a2  [byte;1];
array a3  [byte;1];
union UninoWithPartialDuplicateCustomId {
    a0,
    a1 : 3,
    a2 : 3,
    a3,
}
",
            )
            .unwrap();

        schema_file.flush().unwrap();

        let _should_panic = Parser::parse(&schema_file.into_temp_path());
    }

    #[should_panic]
    #[test]
    fn test_bad_implicit_duplicate_union_schema() {
        let mut schema_file = tempfile::NamedTempFile::new().unwrap();
        schema_file
            .write_all(
                b"
array a0  [byte;1];
array a1  [byte;1];
array a2  [byte;1];
array a3  [byte;1];
array a4  [byte;1];

union UninoWithPartialDuplicateCustomIdInReverseOrder {
    a0 : 10,
    a1,
    a2,
    a3 : 11,
    a4,
}
",
            )
            .unwrap();

        schema_file.flush().unwrap();

        let _should_panic = Parser::parse(&schema_file.into_temp_path());
    }
}
