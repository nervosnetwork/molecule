use std::{collections::HashMap, rc::Rc};

use codegen::ast::{self, HasName};

use crate::types;

type Ast = HashMap<String, Rc<ast::TopDecl>>;

mod utils {
    use crate::bytes::Bytes;

    pub(crate) fn is_byte(ident: &str) -> bool {
        ident == "byte"
    }

    pub(crate) fn entity_name(ident: &str) -> String {
        ident.to_owned()
    }

    pub(crate) fn stmts_start(name: &str, expected: &Bytes) -> Vec<String> {
        let mut stmts = Vec::new();
        stmts.push(expected.c_array("expected"));
        stmts.push("mol_builder_t b;".to_owned());
        stmts.push("mol_seg_res_t res;".to_owned());
        stmts.push(format!("MolBuilder_{}_init(&b);", name));
        stmts
    }

    pub(crate) fn stmts_build(name: &str) -> Vec<String> {
        let mut stmts = Vec::new();
        stmts.push(format!("res = MolBuilder_{}_build(b); ;", name));
        stmts.push(format!("test_build_for({});", name));
        stmts
    }

    pub(crate) fn stmts_if(name: &str, cond: &str, errmsg: &str) -> Vec<String> {
        let mut stmts = Vec::new();
        stmts.push(format!("if ({}) {{", cond));
        stmts.push(format!("    printf(\"Error {}: \");", name));
        stmts.push(format!("    printf({});", errmsg));
        stmts.push("    printf(\"\\n\");".to_owned());
        stmts.push("    free(res.seg.ptr);".to_owned());
        stmts.push("    return 1;".to_owned());
        stmts.push("}".to_owned());
        stmts
    }

    pub(crate) fn stmts_end() -> Vec<String> {
        let mut stmts = Vec::new();
        stmts.push("free(res.seg.ptr);".to_owned());
        stmts.push("return 0;".to_owned());
        stmts
    }
}

pub trait GenTest {
    fn gen_test(&self, _: &Ast) -> Vec<String> {
        Default::default()
    }
}

impl GenTest for types::Any {
    fn gen_test(&self, ast: &Ast) -> Vec<String> {
        match self {
            Self::Option_(inner) => inner.gen_test(ast),
            Self::Union(inner) => inner.gen_test(ast),
            Self::Array(inner) => inner.gen_test(ast),
            Self::StructOrTable(inner) => inner.gen_test(ast),
            Self::Vector(inner) => inner.gen_test(ast),
        }
    }
}

impl GenTest for types::Option_ {
    fn gen_test(&self, ast: &Ast) -> Vec<String> {
        let mut stmts = Vec::new();
        let name = &utils::entity_name(self.name());
        let mut is_none = "true";
        stmts.append(&mut utils::stmts_start(name, self.expected()));
        if let Some(item) = self.item() {
            let decl = ast.get(self.name()).unwrap();
            if let ast::TopDecl::Option_(_) = decl.as_ref() {
            } else {
                panic!("Error: type for {} is incorrect", self.name());
            };
            stmts.push(item.c_array("item"));
            let stmt = format!("MolBuilder_{}_set(&b, item, {});", name, item.len());
            stmts.push(stmt);
            if !item.is_empty() {
                is_none = "false";
            }
        }
        stmts.append(&mut utils::stmts_build(name));
        if self.item().is_some() {
            stmts.push(format!(
                "bool is_none = MolReader_{}_is_none(&res.seg);",
                name
            ));
            let cond = &format!("is_none != {}", is_none);
            let errmsg = "\"failed to check inner item\"";
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
        }
        stmts.append(&mut utils::stmts_end());
        stmts
    }
}

impl GenTest for types::Union {
    fn gen_test(&self, ast: &Ast) -> Vec<String> {
        let mut stmts = Vec::new();
        let name = &utils::entity_name(self.name());
        stmts.append(&mut utils::stmts_start(name, self.expected()));
        if let Some(item) = self.item() {
            let item_type = utils::entity_name(item.typ());
            let stmt = if utils::is_byte(&item_type) {
                stmts.push(item.data().c_byte("item"));
                format!("MolBuilder_{}_set_{}(&b, item);", name, item_type)
            } else {
                stmts.push(item.data().c_array("item"));
                format!(
                    "MolBuilder_{}_set_{}(&b, item, {});",
                    name,
                    item_type,
                    item.data().len()
                )
            };
            stmts.push(stmt);
        }
        stmts.append(&mut utils::stmts_build(name));
        if let Some(item) = self.item() {
            let item_type = utils::entity_name(item.typ());
            let decl = ast.get(self.name()).unwrap();
            let item_id = if let ast::TopDecl::Union(union) = decl.as_ref() {
                union
                    .items()
                    .iter()
                    .enumerate()
                    .find(|(_, inner_item)| inner_item.typ().name() == item.typ())
                    .map(|(index, _)| index)
                    .unwrap()
            } else {
                panic!("Error: type for {} is incorrect", self.name());
            };
            stmts.push(format!(
                "mol_union_t inner = MolReader_{}_unpack(&res.seg);",
                name
            ));
            let cond = &format!("inner.item_id != {}", item_id);
            let errmsg = &format!(
                "\"item id is not match (%d != {})\", inner.item_id",
                item_id
            );
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
            let cond = &format!("inner.seg.size != {}", item.data().len());
            let errmsg = &format!(
                "\"item size is not match (%d != {})\", inner.seg.size",
                item.data().len()
            );
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
            let cond = &if utils::is_byte(&item_type) {
                "*inner.seg.ptr != item".to_owned()
            } else {
                format!("memcmp(inner.seg.ptr, item, {}) != 0", item.data().len())
            };
            let errmsg = "\"item is not match\"";
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
        }
        stmts.append(&mut utils::stmts_end());
        stmts
    }
}

impl GenTest for types::Array {
    fn gen_test(&self, ast: &Ast) -> Vec<String> {
        let mut stmts = Vec::new();
        let name = &utils::entity_name(self.name());
        let decl = ast.get(self.name()).unwrap();
        let item_type = if let ast::TopDecl::Array(ref decl) = decl.as_ref() {
            utils::entity_name(decl.item().typ().name())
        } else {
            panic!("Error: type for {} is incorrect", self.name());
        };
        stmts.append(&mut utils::stmts_start(name, self.expected()));
        for (index, data) in self.data().iter() {
            let item_name = format!("item_{}", index);
            let stmt = if utils::is_byte(&item_type) {
                stmts.push(data.c_byte(&item_name));
                format!("MolBuilder_{}_set_nth{}(&b, {});", name, index, item_name)
            } else {
                stmts.push(data.c_array(&item_name));
                format!("MolBuilder_{}_set_nth{}(&b, {});", name, index, item_name)
            };
            stmts.push(stmt);
        }
        stmts.append(&mut utils::stmts_build(name));
        for (index, data) in self.data().iter() {
            let item_name = format!("item_{}", index);
            let seg_name = format!("seg_{}", index);
            stmts.push(format!(
                "mol_seg_t {} = MolReader_{}_get_nth{}(&res.seg);",
                seg_name, name, index
            ));
            let cond = &format!("{}.size != {}", seg_name, data.len());
            let errmsg = &format!(
                "\"item[{}] size is not match (%d != {})\", {}.size",
                index,
                data.len(),
                seg_name
            );
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
            let cond = &if utils::is_byte(&item_type) {
                format!("*{}.ptr != {}", seg_name, item_name)
            } else {
                format!(
                    "memcmp({}.ptr, {}, {}) != 0",
                    seg_name,
                    item_name,
                    data.len()
                )
            };
            let errmsg = &format!("\"item[{}] is not match\"", index);
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
        }
        stmts.append(&mut utils::stmts_end());
        stmts
    }
}

impl GenTest for types::StructOrTable {
    fn gen_test(&self, ast: &Ast) -> Vec<String> {
        let mut stmts = Vec::new();
        let name = &utils::entity_name(self.name());
        let decl = ast.get(self.name()).unwrap();
        let mut is_struct = false;
        let field_type_dict = if let ast::TopDecl::Struct(ref decl) = decl.as_ref() {
            is_struct = true;
            decl.fields()
                .iter()
                .map(|field| (field.name(), field.typ().name()))
                .collect::<HashMap<_, _>>()
        } else if let ast::TopDecl::Table(ref decl) = decl.as_ref() {
            decl.fields()
                .iter()
                .map(|field| (field.name(), field.typ().name()))
                .collect::<HashMap<_, _>>()
        } else {
            panic!("Error: type for {} is incorrect", self.name());
        };
        stmts.append(&mut utils::stmts_start(name, self.expected()));
        for (field_name, data) in self.data().iter() {
            let field_type = field_type_dict.get(&field_name.as_str()).unwrap();
            let stmt = if utils::is_byte(&field_type) {
                stmts.push(data.c_byte(&field_name));
                format!(
                    "MolBuilder_{}_set_{}(&b, {});",
                    name, field_name, field_name
                )
            } else {
                stmts.push(data.c_array(&field_name));
                if is_struct {
                    format!(
                        "MolBuilder_{}_set_{}(&b, {});",
                        name, field_name, field_name
                    )
                } else {
                    format!(
                        "MolBuilder_{}_set_{}(&b, {}, {});",
                        name,
                        field_name,
                        field_name,
                        data.len()
                    )
                }
            };
            stmts.push(stmt);
        }
        stmts.append(&mut utils::stmts_build(name));
        for (field_name, data) in self.data().iter() {
            let field_type = field_type_dict.get(&field_name.as_str()).unwrap();
            let seg_name = format!("seg_{}", field_name);
            stmts.push(format!(
                "mol_seg_t {} = MolReader_{}_get_{}(&res.seg);",
                seg_name, name, field_name
            ));
            let cond = &format!("{}.size != {}", seg_name, data.len());
            let errmsg = &format!(
                "\"field[{}] size is not match (%d != {})\", {}.size",
                field_name,
                data.len(),
                seg_name
            );
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
            let cond = &if utils::is_byte(&field_type) {
                format!("*{}.ptr != {}", seg_name, field_name)
            } else {
                format!(
                    "memcmp({}.ptr, {}, {}) != 0",
                    seg_name,
                    field_name,
                    data.len(),
                )
            };
            let errmsg = &format!("\"field[{}] is not match\"", field_name);
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
        }
        stmts.append(&mut utils::stmts_end());
        stmts
    }
}

impl GenTest for types::Vector {
    fn gen_test(&self, ast: &Ast) -> Vec<String> {
        let mut stmts = Vec::new();
        let name = &utils::entity_name(self.name());
        let decl = ast.get(self.name()).unwrap();
        let mut is_fixed = false;
        let item_type = if let ast::TopDecl::FixVec(ref decl) = decl.as_ref() {
            is_fixed = true;
            utils::entity_name(decl.item().typ().name())
        } else if let ast::TopDecl::DynVec(ref decl) = decl.as_ref() {
            utils::entity_name(decl.item().typ().name())
        } else {
            panic!("Error: type for {} is incorrect", self.name());
        };
        stmts.append(&mut utils::stmts_start(name, self.expected()));
        for (index, data) in self.data().iter().enumerate() {
            let item_name = format!("item_{}", index);
            let stmt = if utils::is_byte(&item_type) {
                stmts.push(data.c_byte(&item_name));
                format!("MolBuilder_{}_push(&b, {});", name, item_name)
            } else {
                stmts.push(data.c_array(&item_name));
                if is_fixed {
                    format!("MolBuilder_{}_push(&b, {});", name, item_name)
                } else {
                    format!(
                        "MolBuilder_{}_push(&b, {}, {});",
                        name,
                        item_name,
                        data.len()
                    )
                }
            };
            stmts.push(stmt);
        }
        stmts.append(&mut utils::stmts_build(name));
        for (index, data) in self.data().iter().enumerate() {
            let item_name = format!("item_{}", index);
            let res_name = format!("res_seg_{}", index);
            let seg_name = format!("seg_{}", index);
            stmts.push(format!(
                "mol_seg_res_t {} = MolReader_{}_get(&res.seg, {});",
                res_name, name, index
            ));
            let cond = &format!("{}.errno != MOL_OK", res_name);
            let errmsg = &format!("\"item[{}] is not existed\"", index);
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
            stmts.push(format!("mol_seg_t {} = {}.seg;", seg_name, res_name));
            let cond = &format!("{}.size != {}", seg_name, data.len());
            let errmsg = &format!(
                "\"item[{}] size is not match (%d != {})\", {}.size",
                index,
                data.len(),
                seg_name
            );
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
            let cond = &if utils::is_byte(&item_type) {
                format!("*{}.ptr != {}", seg_name, item_name)
            } else {
                format!(
                    "memcmp({}.ptr, {}, {}) != 0",
                    seg_name,
                    item_name,
                    data.len(),
                )
            };
            let errmsg = &format!("\"item[{}] is not match\"", index);
            stmts.append(&mut utils::stmts_if(name, cond, errmsg));
        }
        stmts.append(&mut utils::stmts_end());
        stmts
    }
}
