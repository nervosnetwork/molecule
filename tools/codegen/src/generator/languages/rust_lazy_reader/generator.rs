use super::{ident_new, LazyReaderGenerator};
use crate::ast::{self, HasName, *};
use case::CaseExt;
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
        writeln!(output, "{}", q)?;

        for (item_index, item) in self.items().iter().enumerate() {
            let item_type_name = item.typ().name();
            let item_type_name = ident_new(&format!("as_{}", item_type_name.to_snake()));
            let (transformed_name, tc) = get_rust_type_category(item.typ());
            let convert_code = tc.gen_convert_code();
            let q = quote! {
                impl #name {
                    pub fn #item_type_name(&self) -> Result<#transformed_name, Error> {
                        let item = self.cursor.union_unpack()?;
                        if item.item_id != #item_index {
                            return Err(Error::Header(format!("invalid item type id in union: {}", item.item_id)));
                        }
                        let cur = item.cursor.clone();
                        #convert_code
                    }
                }
            };
            writeln!(output, "{}", q)?;
        }
        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
                    Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::Array {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let (transformed_name, tc) = get_rust_type_category(self.item().typ());
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            Some(self),
            None,
            None,
        )?;
        let total_size = self.item_count() * self.item_size();
        let name = ident_new(self.name());
        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
                    self.cursor.verify_fixed_size(#total_size)
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}
impl LazyReaderGenerator for ast::Option_ {}

impl LazyReaderGenerator for ast::Struct {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        generate_rust_common_table(output, self.name(), self.fields(), Some(self.field_sizes()))?;
        let name = ident_new(self.name());
        let total_size: usize = self.field_sizes().iter().sum();
        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
                    self.cursor.verify_fixed_size(#total_size)
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

// in FixVec, all item size is same and known already, the count is unknown.
impl LazyReaderGenerator for ast::FixVec {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let (transformed_name, tc) = get_rust_type_category(self.item().typ());
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            None,
            Some(self),
            None,
        )?;
        let name = ident_new(self.name());
        let item_size = self.item_size();
        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
                    self.cursor.verify_fixvec(#item_size)
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::DynVec {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        let (transformed_name, tc) = get_rust_type_category(self.item().typ());
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            None,
            None,
            Some(self),
        )?;
        let name = ident_new(self.name());
        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> Result<(), Error> {
                    self.cursor.verify_dynvec()
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::Table {
    fn gen_rust<W: io::Write>(&self, output: &mut W) -> io::Result<()> {
        generate_rust_common_table(output, self.name(), self.fields(), None)?;
        let field_count = self.fields().len();
        let name = ident_new(self.name());
        let q = quote! {
            impl #name {
                pub fn verify(&self, compatible: bool) -> Result<(), Error> {
                    self.cursor.verify_table(#field_count, compatible)
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

fn generate_rust_common_array<W: io::Write>(
    output: &mut W,
    plain_name: &str,
    type_name: TokenStream,
    tc: TypeCategory,
    array: Option<&ast::Array>,
    fixvec: Option<&ast::FixVec>,
    _dynvec: Option<&ast::DynVec>,
) -> io::Result<()> {
    let name = ident_new(plain_name);
    let q = quote! {
        pub struct #name {
            pub cursor: Cursor,
        }

        impl From<Cursor> for #name {
            fn from(cursor: Cursor) -> Self {
                Self { cursor }
            }
        }
    };

    writeln!(output, "{}", q)?;

    // len
    if let Some(arr) = array {
        let item_count = Literal::usize_unsuffixed(arr.item_count());
        let q = quote! {
            impl #name {
                pub fn len(&self) -> usize { #item_count }
             }
        };
        writeln!(output, "{}", q)?;
    } else if fixvec.is_some() {
        let q = quote! {
            impl #name {
                pub fn len(&self) -> Result<usize, Error> {  self.cursor.fixvec_length()  }
            }
        };
        writeln!(output, "{}", q)?;
    } else {
        let q = quote! {
            impl #name {
                pub fn len(&self) -> Result<usize, Error> { self.cursor.dynvec_length() }
            }
        };
        writeln!(output, "{}", q)?;
    }
    generate_rust_common_array_impl(output, plain_name, type_name, tc, array, fixvec)?;
    Ok(())
}

fn generate_rust_common_array_impl<W: io::Write>(
    output: &mut W,
    plain_name: &str,
    item_name: TokenStream,
    tc: TypeCategory,
    array: Option<&Array>,
    fixvec: Option<&FixVec>,
) -> io::Result<()> {
    let slice_by = if let Some(fv) = fixvec {
        let size = fv.item_size();
        quote! { fixvec_slice_by_index(#size, index) }
    } else if let Some(arr) = array {
        let index = arr.item_size();
        quote! { slice_by_offset(#index*index, #index) }
    } else {
        quote! { dynvec_slice_by_index(index) }
    };
    let convert_code = tc.gen_convert_code();
    let name = ident_new(plain_name);
    let iterator_name = ident_new(&format!("{}Iterator", name));
    let iterator_ref_name = ident_new(&format!("{}IteratorRef", name));
    let q = quote! {
        impl #name {
            pub fn get(&self, index: usize) -> Result<#item_name, Error> {
                let cur = self.cursor.#slice_by?;
                #convert_code
            }
        }
    };
    writeln!(output, "{}", q)?;

    if array.is_some() {
        return Ok(());
    }
    let q = quote! {
        pub struct #iterator_name {
            cur: #name,
            index: usize,
            len: usize,
        }
        impl core::iter::Iterator for #iterator_name {
            type Item = #item_name;
            fn next(&mut self) -> Option<Self::Item> {
                if self.index >= self.len {
                    None
                } else {
                    let res = self.cur.get(self.index).unwrap();
                    self.index += 1;
                    Some(res)
                }
            }
        }
        impl core::iter::IntoIterator for #name {
            type Item = #item_name;
            type IntoIter = #iterator_name;
            fn into_iter(self) -> Self::IntoIter {
                let len = self.len().unwrap();
                Self::IntoIter {
                    cur: self,
                    index: 0,
                    len
                }
            }
        }
        pub struct #iterator_ref_name<'a> {
            cur: &'a #name,
            index: usize,
            len: usize
        }
        impl<'a> core::iter::Iterator for #iterator_ref_name<'a> {
            type Item = #item_name;
            fn next(&mut self) -> Option<Self::Item> {
                if self.index >= self.len {
                    None
                } else {
                    let res = self.cur.get(self.index).unwrap();
                    self.index += 1;
                    Some(res)
                }
            }
        }
        impl #name {
            pub fn iter(&self) -> #iterator_ref_name {
                let len = self.len().unwrap();
                #iterator_ref_name {
                    cur: &self,
                    index: 0,
                    len
                }
            }
        }
    };
    writeln!(output, "{}", q)?;
    Ok(())
}

fn generate_rust_common_table<W: io::Write>(
    output: &mut W,
    plain_name: &str,
    fields: &[FieldDecl],
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let name = ident_new(plain_name);
    let q = quote! {

        pub struct #name {
            pub cursor: Cursor,
        }

        impl From<Cursor> for #name {
            fn from(cursor: Cursor) -> Self {
                #name { cursor }
            }
        }
    };
    writeln!(output, "{}", q)?;
    for (index, field) in fields.iter().enumerate() {
        generate_rust_common_table_impl(output, plain_name, field, index, field_sizes)?;
    }
    Ok(())
}

fn generate_rust_common_table_impl<W: io::Write>(
    output: &mut W,
    plain_name: &str,
    field: &FieldDecl,
    index: usize,
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let field_name = field.name();
    let (transformed_name, tc) = get_rust_type_category(field.typ());
    let slice_by = generate_rust_slice_by(index, &field_sizes);
    let convert_code = tc.gen_convert_code();
    let name = ident_new(plain_name);
    let field_name = ident_new(field_name);
    let q = quote! {
        impl #name {
            pub fn #field_name(&self) -> Result<#transformed_name, Error> {
                let cur = self.cursor.#slice_by?;
                #convert_code
             }
         }
    };
    writeln!(output, "{}", q)?;
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
fn get_rust_type_category(typ: &TopDecl) -> (TokenStream, TypeCategory) {
    let name = typ.name();
    let mut tc = TypeCategory::Primitive;
    let token_name = ident_new(name);
    let mut transformed_name = quote! { #token_name };
    match typ {
        // if the field type is array and the field type name is "uint8", "int8" ...
        // then it's primitive
        TopDecl::Array(a) => {
            let field_type_name = name.to_lowercase();
            let new_name = match field_type_name.as_ref() {
                // see https://github.com/XuJiandong/moleculec-c2#extra-support-for-known-types
                "uint8" => quote! { u8 },
                "int8" => quote! { i8 },
                "uint16" => quote! { u16 },
                "int16" => quote! { i16 },
                "uint32" => quote! { u32 },
                "int32" => quote! { i32 },
                "uint64" => quote! { u64 },
                "int64" => quote! { i64 },
                _ => {
                    if let TopDecl::Primitive(_) = a.item().typ().as_ref() {
                        // array of byte
                        tc = TypeCategory::Array;
                        quote! { Cursor }
                    } else {
                        // array of Types
                        tc = TypeCategory::Type;
                        transformed_name
                    }
                }
            };
            transformed_name = new_name;
        }
        TopDecl::Primitive(_) => {
            transformed_name = quote! { u8 };
        }
        TopDecl::FixVec(v) => {
            // FixVec is different than Array: it has a header.
            if let TopDecl::Primitive(_) = v.item().typ().as_ref() {
                // array of byte
                transformed_name = quote! { Cursor };
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
                    transformed_name = quote! { Option<#inner_name> };
                }
                _ => {
                    tc = TypeCategory::Option(1, inner_tc.is_fixvec(), inner_tc.has_from());
                    transformed_name = quote! { Option<#inner_name> };
                }
            }
        }
        _ => {
            tc = TypeCategory::Type;
        }
    }
    (transformed_name, tc)
}

fn generate_rust_slice_by(index: usize, fields_sizes: &Option<&[usize]>) -> TokenStream {
    if let Some(fs) = fields_sizes {
        let size = fs[index];
        let mut offset = 0;
        for i in (0..index).rev() {
            offset += fs[i];
        }
        quote! {
            slice_by_offset(#offset, #size)
        }
    } else {
        quote! {
            table_slice_by_index(#index)
        }
    }
}
