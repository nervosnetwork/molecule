use super::{
    super::{
        integers,
        rust::{module_path_tokens, utilities::RustNamePlan},
    },
    LazyReaderGenerator,
};
use crate::ast::{self, HasName, *};
use crate::generator::{field_name, ident_name, ident_new};
use crate::integer::{ByteOrder, IntegerSpec};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::io;

fn lazy_type_path(typ: &TopDecl, plan: &RustNamePlan) -> TokenStream {
    let name = ident_name(typ.name(), "");
    if let Some(module) = plan.qualified_lazy.get(typ.name()) {
        let module = module_path_tokens(module);
        quote!(#module::#name)
    } else {
        quote!(#name)
    }
}

impl ast::Union {
    fn get_item_name(typ: &TopDecl, plan: &RustNamePlan) -> TokenStream {
        let item_name = ident_name(typ.name(), "");
        let item_type_name = Self::get_type_name(typ, plan);

        quote! {
            #item_name(#item_type_name),
        }
    }

    fn get_type_name(typ: &TopDecl, plan: &RustNamePlan) -> TokenStream {
        match typ {
            TopDecl::Primitive(v) => {
                let name = ident_new(match v.name().to_lowercase().as_str() {
                    "byte" => "u8",
                    "uint8" => "u8",
                    "int8" => "i8",
                    "uint16" => "u16",
                    "int16" => "i16",
                    "uint32" => "u32",
                    "int32" => "i32",
                    "uint64" => "u64",
                    "int64" => "i64",
                    _ => {
                        panic!("unknown type: {}", v.name())
                    }
                });
                quote!(#name)
            }
            TopDecl::Option_(o) => {
                let name = Self::get_type_name(o.item().typ(), plan);
                quote!(::core::option::Option<#name>)
            }
            TopDecl::FixVec(v) => {
                if let TopDecl::Primitive(_) = v.item().typ().as_ref() {
                    quote!(molecule::lazy_reader::Cursor)
                } else {
                    lazy_type_path(typ, plan)
                }
            }
            _ => lazy_type_path(typ, plan),
        }
    }
}

impl LazyReaderGenerator for ast::Union {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        _ast: &ast::Ast,
        plan: &RustNamePlan,
    ) -> io::Result<()> {
        let name = ident_name(self.name(), "");

        // generate enum:
        let q = self.items().iter().map(|item| {
            let item_name = Self::get_item_name(item.typ().as_ref(), plan);
            quote! {
                #item_name
            }
        });
        writeln!(
            output,
            "{}",
            quote! {
                pub enum #name {
                    #( #q )*
                }
            }
        )?;

        // generate enum try_from
        let q = self.items().iter().map(|item| {
            let item_id = item.id();
            let item_name_str = item.typ().name();
            let item_name = ident_name(item_name_str, "");
            let item_type = Self::get_type_name(item.typ(), plan);

            let q = match item.typ().as_ref() {
                TopDecl::Primitive(a) => match a.name().to_lowercase().as_str() {
                    "byte" | "uint8" | "int8" | "uint16" | "int16" | "uint32" | "int32"
                    | "uint64" | "int64" => {
                        quote! {{
                            cur.verify_fixed_size(::core::mem::size_of::<#item_type>())?;
                            cur.try_into()?
                        }}
                    }
                    _ => {
                        quote!(cur.into())
                    }
                },
                TopDecl::Option_(o) => {
                    let inner_type = lazy_type_path(o.item().typ(), plan);
                    quote! {
                        if cur.option_is_none() {
                            ::core::option::Option::None
                        } else {
                            ::core::option::Option::Some(<#inner_type as ::core::convert::From<molecule::lazy_reader::Cursor>>::from(cur))
                        }
                    }
                }
                TopDecl::FixVec(v) => {
                    if let TopDecl::Primitive(_) = v.item().typ().as_ref() {
                        quote! {{
                            if cur.fixvec_length()? != cur.size - NUMBER_SIZE {
                                return ::core::result::Result::Err(molecule::lazy_reader::Error::TotalSize(
                                    cur.fixvec_length()?,
                                    cur.size
                                ));
                            }

                            cur.add_offset(NUMBER_SIZE)?;
                            cur.sub_size(NUMBER_SIZE)?;
                            cur
                        }}
                    } else {
                        quote! {
                            cur.into()
                        }
                    }
                }
                _ => {
                    quote! {
                        cur.into()
                    }
                }
            };

            quote! {
                #item_id => {
                    ::core::result::Result::Ok(Self::#item_name(#q))
                }
            }
        });
        let q = quote! {
            impl ::core::convert::TryFrom<molecule::lazy_reader::Cursor> for #name {
                type Error = molecule::lazy_reader::Error;
                fn try_from(cur: molecule::lazy_reader::Cursor) -> ::core::result::Result<Self, Self::Error> {
                    use molecule::lazy_reader::NUMBER_SIZE;
                    let item = cur.union_unpack()?;

                    let mut cur = cur;
                    cur.add_offset(NUMBER_SIZE)?;
                    cur.sub_size(NUMBER_SIZE)?;

                    match item.item_id {
                        #( #q )*
                        _ => ::core::result::Result::Err(molecule::lazy_reader::Error::UnknownItem)
                    }
                }
            }
        };
        writeln!(output, "{}", q)?;

        // generate verify
        let verify_items = self.items().iter().map(|item| {
            let item_name = ident_name(item.typ().name(), "");
            match item.typ().as_ref() {
                TopDecl::Primitive(_) => {
                    quote!( Self::#item_name(_v) => ::core::result::Result::Ok(()), )
                }
                TopDecl::Option_(_) => {
                    quote! {
                        Self::#item_name(v) => {
                            if v.is_some() {
                                v.as_ref().unwrap().verify(compatible)?;
                            }
                            ::core::result::Result::Ok(())
                        },
                    }
                }
                TopDecl::FixVec(v) => {
                    if let TopDecl::Primitive(_) = v.item().typ().as_ref() {
                        quote! {
                            Self::#item_name(_v) => ::core::result::Result::Ok(()),
                        }
                    } else {
                        quote! {
                            Self::#item_name(v) => {
                                v.verify(compatible)?;
                                ::core::result::Result::Ok(())
                            },
                        }
                    }
                }
                _ => quote! {
                    Self::#item_name(v) => {
                        v.verify(compatible)?;
                        ::core::result::Result::Ok(())
                    },
                },
            }
        });

        let q = quote! {
            impl #name {
                pub fn verify(&self, compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    match self {
                        #( #verify_items )*
                    }
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::Array {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        ast: &ast::Ast,
        plan: &RustNamePlan,
    ) -> io::Result<()> {
        let (transformed_name, tc) = cursor_type(self.item().typ(), ast, plan);
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
        let name = ident_name(self.name(), "");
        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    self.cursor.verify_fixed_size(#total_size)?;
                    ::core::result::Result::Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}
impl LazyReaderGenerator for ast::Option_ {}

impl LazyReaderGenerator for ast::Struct {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        ast: &ast::Ast,
        plan: &RustNamePlan,
    ) -> io::Result<()> {
        generate_rust_common_table(
            output,
            ast,
            plan,
            self.name(),
            self.fields(),
            Some(self.field_sizes()),
        )?;
        let name = ident_name(self.name(), "");
        let total_size: usize = self.field_sizes().iter().sum();

        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    self.cursor.verify_fixed_size(#total_size)?;
                    ::core::result::Result::Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

// in FixVec, all item size is same and known already, the count is unknown.
impl LazyReaderGenerator for ast::FixVec {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        ast: &ast::Ast,
        plan: &RustNamePlan,
    ) -> io::Result<()> {
        let (transformed_name, tc) = cursor_type(self.item().typ(), ast, plan);
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            None,
            Some(self),
            None,
        )?;

        let name = ident_name(self.name(), "");
        let item_size = self.item_size();

        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    self.cursor.verify_fixvec(#item_size)?;
                    ::core::result::Result::Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::DynVec {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        ast: &ast::Ast,
        plan: &RustNamePlan,
    ) -> io::Result<()> {
        let (transformed_name, tc) = cursor_type(self.item().typ(), ast, plan);
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            None,
            None,
            Some(self),
        )?;

        let item_typ = self.item().typ().as_ref();
        let verify_item = verify_cursor(
            item_typ,
            quote!(self.cursor.dynvec_slice_by_index(i)?),
            plan,
        );
        let val_compatible = if verify_uses_compatible(item_typ) {
            quote!(compatible)
        } else {
            quote!(_compatible)
        };

        let name = ident_name(self.name(), "");
        let q = quote! {
            impl #name {
                pub fn verify(&self, #val_compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    self.cursor.verify_dynvec()?;
                    for i in 0..self.len()? {
                        #verify_item
                    }
                    ::core::result::Result::Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::Table {
    fn gen_rust<W: io::Write>(
        &self,
        output: &mut W,
        ast: &ast::Ast,
        plan: &RustNamePlan,
    ) -> io::Result<()> {
        generate_rust_common_table(output, ast, plan, self.name(), self.fields(), None)?;
        let field_count = self.fields().len();
        let name = ident_name(self.name(), "");

        let verify_fields = self.fields().iter().enumerate().map(|(index, f)| {
            verify_cursor(
                f.typ().as_ref(),
                quote!(self.cursor.table_slice_by_index(#index)?),
                plan,
            )
        });

        let q = quote! {
            impl #name {
                pub fn verify(&self, compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    self.cursor.verify_table(#field_count, compatible)?;
                    #( #verify_fields )*
                    ::core::result::Result::Ok(())
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
    tc: CursorValue,
    array: Option<&ast::Array>,
    fixvec: Option<&ast::FixVec>,
    _dynvec: Option<&ast::DynVec>,
) -> io::Result<()> {
    let name = ident_name(plain_name, "");
    let q = quote! {
        #[derive(Clone)]
        pub struct #name {
            pub cursor: molecule::lazy_reader::Cursor,
        }

        impl ::core::convert::From<molecule::lazy_reader::Cursor> for #name {
            fn from(cursor: molecule::lazy_reader::Cursor) -> Self {
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
                pub fn len(&self) -> ::core::result::Result<usize, molecule::lazy_reader::Error> { self.cursor.fixvec_length() }
            }
        };
        writeln!(output, "{}", q)?;
    } else {
        let q = quote! {
            impl #name {
                pub fn len(&self) -> ::core::result::Result<usize, molecule::lazy_reader::Error> { self.cursor.dynvec_length() }
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
    tc: CursorValue,
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
    let value = tc.expression();
    let name = ident_name(plain_name, "");
    let iterator_name = ident_name(plain_name, "Iterator");
    let iterator_ref_name = ident_name(plain_name, "IteratorRef");
    let q = quote! {
        impl #name {
            pub fn get(&self, index: usize) -> ::core::result::Result<#item_name, molecule::lazy_reader::Error> {
                let cur = self.cursor.#slice_by?;
                ::core::result::Result::Ok(#value)
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
        impl ::core::iter::Iterator for #iterator_name {
            type Item = #item_name;
            fn next(&mut self) -> ::core::option::Option<Self::Item> {
                if self.index >= self.len {
                    ::core::option::Option::None
                } else {
                    let res = self.cur.get(self.index).unwrap();
                    self.index += 1;
                    ::core::option::Option::Some(res)
                }
            }
        }
        impl ::core::iter::IntoIterator for #name {
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
        impl<'a> ::core::iter::Iterator for #iterator_ref_name<'a> {
            type Item = #item_name;
            fn next(&mut self) -> ::core::option::Option<Self::Item> {
                if self.index >= self.len {
                    ::core::option::Option::None
                } else {
                    let res = self.cur.get(self.index).unwrap();
                    self.index += 1;
                    ::core::option::Option::Some(res)
                }
            }
        }
        impl #name {
            pub fn iter(&self) -> #iterator_ref_name<'_> {
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
    ast: &ast::Ast,
    plan: &RustNamePlan,
    plain_name: &str,
    fields: &[FieldDecl],
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let name = ident_name(plain_name, "");
    let q = quote! {
        #[derive(Clone)]
        pub struct #name {
            pub cursor: molecule::lazy_reader::Cursor,
        }

        impl ::core::convert::From<molecule::lazy_reader::Cursor> for #name {
            fn from(cursor: molecule::lazy_reader::Cursor) -> Self {
                #name { cursor }
            }
        }
    };
    writeln!(output, "{}", q)?;
    for (index, field) in fields.iter().enumerate() {
        generate_rust_common_table_impl(output, ast, plan, plain_name, field, index, field_sizes)?;
    }
    Ok(())
}

fn generate_rust_common_table_impl<W: io::Write>(
    output: &mut W,
    ast: &ast::Ast,
    plan: &RustNamePlan,
    plain_name: &str,
    field: &FieldDecl,
    index: usize,
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let (transformed_name, tc) = cursor_type(field.typ(), ast, plan);
    let slice_by = generate_rust_slice_by(index, &field_sizes);
    let value = tc.expression();
    let name = ident_name(plain_name, "");
    let field_name = field_name(field.name());
    let q = quote! {
        impl #name {
            pub fn #field_name(&self) -> ::core::result::Result<#transformed_name, molecule::lazy_reader::Error> {
                let cur = self.cursor.#slice_by?;
                ::core::result::Result::Ok(#value)
             }
         }
    };
    writeln!(output, "{}", q)?;
    Ok(())
}

// The legacy lazy API recognizes these array names even in an unannotated schema.
fn legacy_integer(name: &str) -> Option<IntegerSpec> {
    let name = name.to_ascii_lowercase();
    let (signed, width) = if let Some(width) = name.strip_prefix("uint") {
        (false, width)
    } else {
        (true, name.strip_prefix("int")?)
    };
    let bits = match width {
        "8" => 8,
        "16" => 16,
        "32" => 32,
        "64" => 64,
        _ => return None,
    };
    Some(IntegerSpec {
        bits,
        signed,
        byte_order: ByteOrder::Little,
    })
}

pub(super) fn check_legacy_integer_types(ast: &Ast) -> io::Result<()> {
    for (name, declared) in ast.integers() {
        let Some(legacy) = legacy_integer(name) else {
            continue;
        };
        if declared.bits != legacy.bits || declared.signed != legacy.signed {
            let expected = integers::native(legacy);
            let actual = integers::native(*declared);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("E_INT_LAZY_TYPE: {name} has an existing lazy getter type {expected}; choose a different name for {actual}"),
            ));
        }
    }
    Ok(())
}

enum CursorValue {
    Into,
    TryInto,
    Bytes,
    Integer(IntegerSpec),
    Optional(Box<Self>),
}

impl CursorValue {
    fn expression(&self) -> TokenStream {
        match self {
            Self::Into => quote!(cur.into()),
            Self::TryInto => quote!(cur.try_into()?),
            Self::Bytes => quote!(cur.convert_to_rawbytes()?),
            Self::Integer(spec) => {
                let width = spec.byte_width();
                let value = integers::decode(*spec, quote!(cur.try_into()?));
                quote!({
                    cur.verify_fixed_size(#width)?;
                    #value
                })
            }
            Self::Optional(inner) => {
                let value = inner.expression();
                quote!(if cur.option_is_none() { ::core::option::Option::None } else { ::core::option::Option::Some(#value) })
            }
        }
    }
}

fn cursor_type(typ: &TopDecl, ast: &Ast, plan: &RustNamePlan) -> (TokenStream, CursorValue) {
    match typ {
        TopDecl::Array(array) => {
            if let Some(legacy) = legacy_integer(typ.name()).filter(|legacy| {
                array.item().typ().is_byte() && array.item_count() == legacy.byte_width()
            }) {
                let value = ast
                    .integer(typ.name())
                    .copied()
                    .map_or(CursorValue::TryInto, CursorValue::Integer);
                (integers::native(legacy), value)
            } else if array.item().typ().is_byte() {
                let width = array.item_count();
                (quote!([u8; #width]), CursorValue::TryInto)
            } else {
                (lazy_type_path(typ, plan), CursorValue::Into)
            }
        }
        TopDecl::Primitive(_) => (quote!(u8), CursorValue::TryInto),
        TopDecl::FixVec(vector) if vector.item().typ().is_byte() => {
            (quote!(molecule::lazy_reader::Cursor), CursorValue::Bytes)
        }
        TopDecl::Option_(option) => {
            let (native, value) = cursor_type(option.item().typ(), ast, plan);
            (
                quote!(::core::option::Option<#native>),
                CursorValue::Optional(Box::new(value)),
            )
        }
        TopDecl::Union(_) => (lazy_type_path(typ, plan), CursorValue::TryInto),
        _ => (lazy_type_path(typ, plan), CursorValue::Into),
    }
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

fn verify_uses_compatible(typ: &TopDecl) -> bool {
    match typ {
        TopDecl::Option_(v) => verify_uses_compatible(v.item().typ().as_ref()),
        TopDecl::Union(_) | TopDecl::DynVec(_) | TopDecl::Table(_) => true,
        TopDecl::Primitive(_) | TopDecl::Array(_) | TopDecl::Struct(_) | TopDecl::FixVec(_) => {
            false
        }
    }
}

fn verify_cursor(typ: &TopDecl, cur: TokenStream, plan: &RustNamePlan) -> TokenStream {
    match typ {
        TopDecl::Primitive(_) => quote!((#cur).verify_fixed_size(1)?;),
        TopDecl::Option_(v) => {
            let inner = verify_cursor(v.item().typ().as_ref(), quote!(val), plan);
            quote!({
                let val = #cur;
                if !val.option_is_none() {
                    #inner
                }
            })
        }
        TopDecl::Union(_) => {
            let type_path = lazy_type_path(typ, plan);
            quote!(<#type_path as ::core::convert::TryFrom<molecule::lazy_reader::Cursor>>::try_from(#cur)?.verify(compatible)?;)
        }
        TopDecl::Array(array) => {
            let total_size = array.total_size();
            quote!((#cur).verify_fixed_size(#total_size)?;)
        }
        TopDecl::Struct(record) => {
            let total_size = record.total_size();
            quote!((#cur).verify_fixed_size(#total_size)?;)
        }
        TopDecl::FixVec(v) => {
            let item_size = v.item_size();
            quote!((#cur).verify_fixvec(#item_size)?;)
        }
        TopDecl::DynVec(_) | TopDecl::Table(_) => {
            let type_path = lazy_type_path(typ, plan);
            quote!(<#type_path as ::core::convert::From<molecule::lazy_reader::Cursor>>::from(#cur).verify(compatible)?;)
        }
    }
}
