use super::{ident_new, LazyReaderGenerator};
/// ## Terminology
/// - field, existing in "struct", "table"
/// - field name, name of field
/// - field type, type of field
/// - field type name, name of field type
/// - item, existing in "dynvec", "union" and "option"
/// - item name, there is no such thing, the item is without name
/// - item type, type of item
/// - item type name, name of item type
/// - type name, item type name or field type name
/// - raw name, a struct name without "Type", same as name
/// - type category, field type or item type are classified into 3 categories, see TypeCategory.
///   Different category has different implementation.
/// - C/Rust transformed name: transformed from "field type name" or "item type name",
///   according to their type category, e.g. uint32_t, mol2_cursor_t, XXXType
/// - name, used for class name only
/// - class type name, name with "Type" suffix, e.g. SampleTableType
/// - common array, the set of Array, FixVec and DynVec, they share method like "len", "get"
/// - common table, the set of Table, Struct, they share same method like ".t->XXX"
use crate::ast::{self, HasName, *};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::io;

impl LazyReaderGenerator for ast::Union {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let name = ident_new(self.name());

        let q = quote! {
            pub struct #name {
                pub cursor: Cursor,
            }

            impl From<Cursor> for #name {
                fn from(cursor: Cursor) -> Self {
                    Self { cursor }
                }
            }

            impl #name {
                pub fn item_id(&self) -> Result<usize, Error> {
                    let item = self.cursor.union_unpack()?;
                    Ok(item.item_id)
                }
            }
        };
        write!(output, "{}", q)?;

        for item in self.items() {
            let item_type_name = item.typ().name();
            let item_type_name = ident_new(&format!("as_{}", item_type_name.to_lowercase()));
            let (transformed_name, tc) = get_rust_type_category(item.typ());
            let transformed_name = syn::parse_str::<syn::Type>(&transformed_name).unwrap();
            let convert_code = tc.gen_convert_code();
            let q = quote! {
                impl #name {
                    pub fn #item_type_name(&self) -> Result<#transformed_name, Error> {
                        let item = self.cursor.union_unpack()?;
                        let cur = item.cursor.clone();
                        #convert_code
                    }
                }
            };
            write!(output, "{}", q)?;
        }

        Ok(())
    }
}

impl LazyReaderGenerator for ast::Array {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let (transformed_name, tc) = get_rust_type_category(self.item().typ());
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name.as_str(),
            tc,
            Some(self),
            None,
            None,
        )
    }
}
impl LazyReaderGenerator for ast::Option_ {}

impl LazyReaderGenerator for ast::Struct {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        generate_rust_common_table(output, self.name(), self.fields(), Some(self.field_sizes()))
    }
}

// in FixVec, all item size is same and known already, the count is unknown.
impl LazyReaderGenerator for ast::FixVec {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let (transformed_name, tc) = get_rust_type_category(self.item().typ());
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name.as_str(),
            tc,
            None,
            Some(self),
            None,
        )
    }
}

impl LazyReaderGenerator for ast::DynVec {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let (transformed_name, tc) = get_rust_type_category(self.item().typ());
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name.as_str(),
            tc,
            None,
            None,
            Some(self),
        )
    }
}

impl LazyReaderGenerator for ast::Table {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        generate_rust_common_table(output, self.name(), self.fields(), None)
    }
}

fn generate_rust_common_array<W: io::Write>(
    output: &mut W,
    name: &str,
    type_name: &str,
    tc: TypeCategory,
    array: Option<&ast::Array>,
    fixvec: Option<&ast::FixVec>,
    _dynvec: Option<&ast::DynVec>,
) -> io::Result<()> {
    let n = ident_new(name);
    let q = quote! {
        pub struct #n {
            pub cursor: Cursor,
        }

        impl From<Cursor> for #n {
            fn from(cursor: Cursor) -> Self {
                Self { cursor }
            }
        }
    };

    write!(output, "{}", q)?;

    // len
    if let Some(arr) = array {
        let item_count = Literal::usize_unsuffixed(arr.item_count());
        let q = quote! {
            impl #n {
                pub fn len(&self) -> usize { #item_count }
             }
        };
        write!(output, "{}", q)?;
    } else if fixvec.is_some() {
        let q = quote! {
            impl #n {
                pub fn len(&self) -> Result<usize, Error> {  self.cursor.fixvec_length()  }
            }
        };
        write!(output, "{}", q)?;
    } else {
        let q = quote! {
            impl #n {
                pub fn len(&self) -> Result<usize, Error> { self.cursor.dynvec_length() }
            }
        };
        write!(output, "{}", q)?;
    }

    generate_rust_common_array_impl(output, name, type_name, tc, array, fixvec)?;
    Ok(())
}

fn generate_rust_common_array_impl<W: io::Write>(
    output: &mut W,
    name: &str,
    type_name: &str,
    tc: TypeCategory,
    array: Option<&Array>,
    fixvec: Option<&FixVec>,
) -> io::Result<()> {
    let slice_by = if let Some(fv) = fixvec {
        format!("fixvec_slice_by_index({}, index)", fv.item_size())
    } else if let Some(arr) = array {
        format!("slice_by_offset({0}*index, {0})", arr.item_size())
    } else {
        "dynvec_slice_by_index(index)".to_string()
    };
    let convert_code = tc.gen_convert_code();
    write!(
        output,
        r#"
        impl {0} {{
            pub fn get(&self, index: usize) -> Result<{1}, Error> {{
                let cur = self.cursor.{2}?;
                {3}
            }}
        }}
        "#,
        name, type_name, slice_by, convert_code
    )?;
    Ok(())
}

fn generate_rust_common_table<W: io::Write>(
    output: &mut W,
    name: &str,
    fields: &[FieldDecl],
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let n = ident_new(name);
    let q = quote! {

        pub struct #n {
            pub cursor: Cursor,
        }

        impl From<Cursor> for #n {
            fn from(cursor: Cursor) -> Self {
                #n { cursor }
            }
        }
    };
    write!(output, "{}", q)?;
    for (index, field) in fields.iter().enumerate() {
        generate_rust_common_table_impl(output, name, field, index, field_sizes)?;
    }
    Ok(())
}

fn generate_rust_common_table_impl<W: io::Write>(
    output: &mut W,
    name: &str,
    field: &FieldDecl,
    index: usize,
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let field_name = field.name();
    let (transformed_name, tc) = get_rust_type_category(field.typ());
    let slice_by = generate_rust_slice_by(index, &field_sizes);
    let convert_code = tc.gen_convert_code();
    write!(
        output,
        r#"
        impl {0} {{
            pub fn {1}(&self) -> Result<{2}, Error> {{
                let cur = self.cursor.{3}?;
                {4}
             }}
         }}
        "#,
        name, field_name, transformed_name, slice_by, convert_code
    )?;
    Ok(())
}

// 1. category 1, primitive
// uint8, int8
// uint16, int16
// uint32, int32
// uint64, int64

// 2. category 2, array/fixvec
// <byte>
// array

// 3. category 3, normal type

// 4. category 4, Option
enum TypeCategory {
    Primitive,
    Array,
    FixVec,
    Type,
    // 1st: nested level
    // 2nd: is nested type is FixVec or not
    // 3rd: has From<T>
    Option(u32, bool, bool),
}

impl TypeCategory {
    pub fn is_fixvec(&self) -> bool {
        matches!(self, Self::FixVec)
    }
    pub fn has_from(&self) -> bool {
        matches!(self, Self::Type | Self::Array)
    }
    pub fn gen_convert_code(&self) -> TokenStream {
        match self {
            &TypeCategory::Option(level, flag, has_from) => {
                if level == 1 {
                    if flag {
                        quote! {
                            if cur.option_is_none() {
                                Ok(None)
                            } else {
                                let cur = cur.convert_to_rawbytes()?;
                                Ok(Some(cur.into()))
                            }
                        }
                    } else if has_from {
                        quote! {
                            if cur.option_is_none() {
                                Ok(None)
                            } else {
                                Ok(Some(cur.into()))
                            }
                        }
                    } else {
                        quote! {
                            if cur.option_is_none() {
                                Ok(None)
                            } else {
                                Ok(Some(cur.try_into()?))
                            }
                        }
                    }
                } else if level == 2 {
                    if flag {
                        quote! {
                            if cur.option_is_none() {
                                Ok(None)
                            } else {
                                let cur = cur.convert_to_rawbytes()?;
                                Ok(Some(Some(cur.try_into())))
                           }
                        }
                    } else {
                        quote! {
                            if cur.option_is_none() {
                                Ok(None)
                            } else {
                                Ok(Some(Some(cur.into())))
                            }
                        }
                    }
                } else {
                    panic!("can't support")
                }
            }
            TypeCategory::Type => quote! { Ok(cur.into()) },
            TypeCategory::Primitive => quote! { cur.try_into() },
            TypeCategory::Array => quote! { Ok(cur) },
            TypeCategory::FixVec => {
                quote! {
                    cur.convert_to_rawbytes()
                }
            }
        }
    }
}

// see TypeCategory
fn get_rust_type_category(typ: &TopDecl) -> (String, TypeCategory) {
    let name = typ.name();
    let mut tc = TypeCategory::Primitive;
    let mut transformed_name = String::from(name);
    match typ {
        // if the field type is array and the field type name is "uint8", "int8" ...
        // then it's primitive
        TopDecl::Array(a) => {
            let field_type_name = name.to_lowercase();
            let new_name = match field_type_name.as_ref() {
                // see https://github.com/XuJiandong/moleculec-c2#extra-support-for-known-types
                "uint8" => "u8",
                "int8" => "i8",
                "uint16" => "u16",
                "int16" => "i16",
                "uint32" => "u32",
                "int32" => "i32",
                "uint64" => "u64",
                "int64" => "i64",
                _ => {
                    if let TopDecl::Primitive(_) = a.item().typ().as_ref() {
                        // array of byte
                        tc = TypeCategory::Array;
                        "Cursor"
                    } else {
                        // array of Types
                        tc = TypeCategory::Type;
                        transformed_name.as_str()
                    }
                }
            };
            transformed_name = String::from(new_name);
        }
        TopDecl::Primitive(_) => {
            transformed_name = String::from("u8");
        }
        TopDecl::FixVec(v) => {
            // FixVec is different than Array: it has a header.
            if let TopDecl::Primitive(_) = v.item().typ().as_ref() {
                // array of byte
                transformed_name = String::from("Cursor");
                tc = TypeCategory::FixVec;
            } else {
                tc = TypeCategory::Type;
            }
        }
        TopDecl::Option_(o) => {
            // Option<Script>, etc
            let (inner_name, inner_tc) = get_rust_type_category(o.item().typ());
            match inner_tc {
                TypeCategory::Option(level, flag, has_from) => {
                    tc = TypeCategory::Option(level + 1, flag, has_from);
                    transformed_name = format!("Option<{}>", inner_name);
                }
                _ => {
                    transformed_name = format!("Option<{}>", inner_name);
                    tc = TypeCategory::Option(1, inner_tc.is_fixvec(), inner_tc.has_from());
                }
            }
        }
        _ => {
            tc = TypeCategory::Type;
        }
    }
    (transformed_name, tc)
}

fn generate_rust_slice_by(index: usize, fields_sizes: &Option<&[usize]>) -> String {
    if let Some(fs) = fields_sizes {
        let size = fs[index];
        let mut offset = 0;
        for i in (0..index).rev() {
            offset += fs[i];
        }
        format!("slice_by_offset({}, {})", offset, size)
    } else {
        format!("table_slice_by_index({})", index)
    }
}
