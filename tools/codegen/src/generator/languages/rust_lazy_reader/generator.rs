use super::{
    super::{integers, rust::module_path_tokens},
    LazyReaderGenerator,
};
use crate::ast::{self, HasName, *};
use crate::generator::{field_name, ident_name};
use crate::integer::{ByteOrder, IntegerSpec};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::io;

fn lazy_type_path(typ: &TopDecl, ast: &Ast) -> TokenStream {
    let name = ident_name(typ.name(), "");
    if let Some(module) = ast.decl_origin(typ.name()) {
        let module = module_path_tokens(module);
        quote!(#module::#name)
    } else {
        quote!(#name)
    }
}

impl LazyReaderGenerator for ast::Union {
    fn gen_rust<W: io::Write>(&self, output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        let name = ident_name(self.name(), "");
        let exact = !ast.integers().is_empty();
        let unpack = if exact {
            quote!(cur.union_unpack_exact()?)
        } else {
            quote!(cur.union_unpack()?)
        };
        let mut variants = Vec::new();
        let mut decode = Vec::new();
        let mut verify = Vec::new();
        for item in self.items() {
            let typ = item.typ();
            let variant = ident_name(typ.name(), "");
            let id = item.id();
            let (native, value) = cursor_type(typ, ast, true);
            let check = value.verify_union_value();
            let value = value.expression(exact);
            variants.push(quote!(#variant(#native)));
            decode.push(quote!(#id => ::core::result::Result::Ok(Self::#variant(#value))));
            verify.push(quote!(Self::#variant(_value) => #check));
        }
        writeln!(
            output,
            "{}",
            quote! {
                pub enum #name { #(#variants,)* }
                impl ::core::convert::TryFrom<molecule::lazy_reader::Cursor> for #name {
                    type Error = molecule::lazy_reader::Error;
                    fn try_from(cur: molecule::lazy_reader::Cursor) -> ::core::result::Result<Self, molecule::lazy_reader::Error> {
                        let item = #unpack;
                        let cur = item.cursor;
                        match item.item_id {
                            #(#decode,)*
                            _ => ::core::result::Result::Err(molecule::lazy_reader::Error::UnknownItem),
                        }
                    }
                }
                impl #name {
                    pub fn verify(&self, _compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                        match self { #(#verify,)* }
                    }
                }
            }
        )
    }
}

enum SequenceKind {
    Array { count: usize, item_size: usize },
    FixVec { item_size: usize },
    DynVec,
}

impl LazyReaderGenerator for ast::Array {
    fn gen_rust<W: io::Write>(&self, output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        let (transformed_name, tc) = cursor_type(self.item().typ(), ast, false);
        let exact = !ast.integers().is_empty();
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            exact,
            SequenceKind::Array {
                count: self.item_count(),
                item_size: self.item_size(),
            },
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
    fn gen_rust<W: io::Write>(&self, output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        generate_rust_common_table(
            output,
            ast,
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
    fn gen_rust<W: io::Write>(&self, output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        let (transformed_name, tc) = cursor_type(self.item().typ(), ast, false);
        let exact = !ast.integers().is_empty();
        let item_size = self.item_size();
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            exact,
            SequenceKind::FixVec { item_size },
        )?;

        let name = ident_name(self.name(), "");
        let verify_expr = if exact {
            quote!(self.cursor.fixvec_len_exact(#item_size)?;)
        } else {
            quote!(self.cursor.verify_fixvec(#item_size)?;)
        };

        let q = quote! {
            impl #name {
                pub fn verify(&self, _compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    #verify_expr
                    ::core::result::Result::Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::DynVec {
    fn gen_rust<W: io::Write>(&self, output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        let (transformed_name, tc) = cursor_type(self.item().typ(), ast, false);
        let item_typ = self.item().typ().as_ref();
        let exact = !ast.integers().is_empty();
        generate_rust_common_array(
            output,
            self.name(),
            transformed_name,
            tc,
            exact,
            SequenceKind::DynVec,
        )?;

        let val_compatible = if verify_uses_compatible(item_typ) {
            quote!(compatible)
        } else {
            quote!(_compatible)
        };
        let verify_body = if exact {
            let verify_item = verify_cursor(
                item_typ,
                quote!(self.cursor.dynvec_item_exact(count, i)?),
                ast,
                true,
            );
            quote! {
                let count = self.cursor.verify_dynvec_exact()?;
                for i in 0..count {
                    #verify_item
                }
            }
        } else {
            let verify_item = verify_cursor(
                item_typ,
                quote!(self.cursor.dynvec_slice_by_index(i)?),
                ast,
                false,
            );
            quote! {
                self.cursor.verify_dynvec()?;
                for i in 0..self.len()? {
                    #verify_item
                }
            }
        };

        let name = ident_name(self.name(), "");
        let q = quote! {
            impl #name {
                pub fn verify(&self, #val_compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    #verify_body
                    ::core::result::Result::Ok(())
                }
            }
        };
        writeln!(output, "{}", q)?;
        Ok(())
    }
}

impl LazyReaderGenerator for ast::Table {
    fn gen_rust<W: io::Write>(&self, output: &mut W, ast: &ast::Ast) -> io::Result<()> {
        generate_rust_common_table(output, ast, self.name(), self.fields(), None)?;
        let field_count = self.fields().len();
        let name = ident_name(self.name(), "");
        let exact = !ast.integers().is_empty();

        let verify_body = if exact {
            if self.fields().is_empty() {
                quote!(self.cursor.verify_table_exact(0, compatible)?;)
            } else {
                let verify_fields = self.fields().iter().enumerate().map(|(index, f)| {
                    verify_cursor(
                        f.typ().as_ref(),
                        quote!(self.cursor.dynvec_item_exact(count, #index)?),
                        ast,
                        true,
                    )
                });
                quote! {
                    let count = self.cursor.verify_table_exact(#field_count, compatible)?;
                    #( #verify_fields )*
                }
            }
        } else {
            let verify_fields = self.fields().iter().enumerate().map(|(index, f)| {
                verify_cursor(
                    f.typ().as_ref(),
                    quote!(self.cursor.table_slice_by_index(#index)?),
                    ast,
                    false,
                )
            });
            quote! {
                self.cursor.verify_table(#field_count, compatible)?;
                #( #verify_fields )*
            }
        };

        let q = quote! {
            impl #name {
                pub fn verify(&self, compatible: bool) -> ::core::result::Result<(), molecule::lazy_reader::Error> {
                    #verify_body
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
    exact: bool,
    kind: SequenceKind,
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

    let len_impl = match (&kind, exact) {
        (SequenceKind::Array { count, .. }, _) => {
            let item_count = Literal::usize_unsuffixed(*count);
            quote!(pub fn len(&self) -> usize { #item_count })
        }
        (SequenceKind::FixVec { item_size }, true) => quote! {
            pub fn len(&self) -> ::core::result::Result<usize, molecule::lazy_reader::Error> {
                ::core::result::Result::Ok(self.cursor.fixvec_len_exact(#item_size)?)
            }
        },
        (SequenceKind::FixVec { .. }, false) => quote! {
            pub fn len(&self) -> ::core::result::Result<usize, molecule::lazy_reader::Error> {
                self.cursor.fixvec_length()
            }
        },
        (SequenceKind::DynVec, true) => quote! {
            pub fn len(&self) -> ::core::result::Result<usize, molecule::lazy_reader::Error> {
                ::core::result::Result::Ok(self.cursor.dynvec_len_exact()?)
            }
        },
        (SequenceKind::DynVec, false) => quote! {
            pub fn len(&self) -> ::core::result::Result<usize, molecule::lazy_reader::Error> {
                self.cursor.dynvec_length()
            }
        },
    };
    writeln!(output, "{}", quote!(impl #name { #len_impl }))?;
    generate_rust_common_array_impl(output, plain_name, type_name, tc, exact, kind)?;
    Ok(())
}

fn generate_rust_common_array_impl<W: io::Write>(
    output: &mut W,
    plain_name: &str,
    item_name: TokenStream,
    tc: CursorValue,
    exact: bool,
    kind: SequenceKind,
) -> io::Result<()> {
    let slice_by = match (&kind, exact) {
        (SequenceKind::FixVec { item_size }, true) => quote!(fixvec_item_exact(#item_size, index)),
        (SequenceKind::FixVec { item_size }, false) => {
            quote!(fixvec_slice_by_index(#item_size, index))
        }
        (SequenceKind::Array { count, item_size }, true) => {
            quote!(array_item_exact(#count, #item_size, index))
        }
        (SequenceKind::Array { item_size, .. }, false) => {
            quote!(slice_by_offset(#item_size * index, #item_size))
        }
        (SequenceKind::DynVec, true) => quote!(table_field_exact(0, index)),
        (SequenceKind::DynVec, false) => quote!(dynvec_slice_by_index(index)),
    };
    let value = tc.expression(exact);
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

    if matches!(kind, SequenceKind::Array { .. }) {
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
    let field_count = fields.len();
    for (index, field) in fields.iter().enumerate() {
        generate_rust_common_table_impl(
            output,
            ast,
            plain_name,
            field,
            index,
            field_count,
            field_sizes,
        )?;
    }
    Ok(())
}

fn generate_rust_common_table_impl<W: io::Write>(
    output: &mut W,
    ast: &ast::Ast,
    plain_name: &str,
    field: &FieldDecl,
    index: usize,
    field_count: usize,
    field_sizes: Option<&[usize]>,
) -> io::Result<()> {
    let (transformed_name, tc) = cursor_type(field.typ(), ast, false);
    let exact = !ast.integers().is_empty();
    let slice_by = generate_rust_slice_by(index, field_count, field_sizes, exact);
    let value = tc.expression(exact);
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
    Byte,
    Into,
    TryInto,
    Bytes,
    FixedBytes(usize),
    Integer(IntegerSpec),
    Optional(Box<Self>),
}

impl CursorValue {
    // Byte and Bytes lose their framing during conversion. Only payloads that
    // retain a generated reader still have wire data to verify.
    fn verify_union_value(&self) -> TokenStream {
        match self {
            Self::Into | Self::TryInto => quote!(_value.verify(_compatible)),
            Self::Optional(inner) => {
                let check = inner.verify_union_value();
                quote!(match _value {
                    ::core::option::Option::Some(_value) => #check,
                    ::core::option::Option::None => ::core::result::Result::Ok(()),
                })
            }
            Self::Byte | Self::Bytes | Self::FixedBytes(_) | Self::Integer(_) => {
                quote!(::core::result::Result::Ok(()))
            }
        }
    }

    fn expression(&self, exact: bool) -> TokenStream {
        match self {
            Self::Byte if exact => quote!(cur.read_array::<1>()?[0]),
            Self::Byte => quote!({
                cur.verify_fixed_size(1)?;
                ::core::convert::TryInto::try_into(cur)?
            }),
            Self::Into => quote!(::core::convert::Into::into(cur)),
            Self::TryInto => quote!(::core::convert::TryInto::try_into(cur)?),
            Self::Bytes if exact => quote!(cur.fixvec_raw_bytes_exact()?),
            Self::Bytes => quote!({
                let raw = cur.convert_to_rawbytes()?;
                cur.verify_fixed_size(molecule::lazy_reader::NUMBER_SIZE + raw.size)?;
                raw
            }),
            Self::FixedBytes(width) => quote!(cur.read_array::<#width>()?),
            Self::Integer(spec) => {
                let width = spec.byte_width();
                integers::decode(*spec, quote!(cur.read_array::<#width>()?))
            }
            Self::Optional(inner) => {
                let value = inner.expression(exact);
                quote!(if cur.option_is_none() { ::core::option::Option::None } else { ::core::option::Option::Some(#value) })
            }
        }
    }
}

fn cursor_type(typ: &TopDecl, ast: &Ast, union_payload: bool) -> (TokenStream, CursorValue) {
    let exact = !ast.integers().is_empty();
    match typ {
        TopDecl::Array(array) if !union_payload => {
            if let Some(legacy) = legacy_integer(typ.name()).filter(|legacy| {
                array.item().typ().is_byte() && array.item_count() == legacy.byte_width()
            }) {
                let value = match (ast.integer(typ.name()).copied(), exact) {
                    (Some(spec), _) => CursorValue::Integer(spec),
                    (None, true) => CursorValue::Integer(legacy),
                    (None, false) => CursorValue::TryInto,
                };
                (integers::native(legacy), value)
            } else if array.item().typ().is_byte() {
                let width = array.item_count();
                let value = if exact {
                    CursorValue::FixedBytes(width)
                } else {
                    CursorValue::TryInto
                };
                (quote!([u8; #width]), value)
            } else {
                (lazy_type_path(typ, ast), CursorValue::Into)
            }
        }
        TopDecl::Primitive(_) => (quote!(u8), CursorValue::Byte),
        TopDecl::FixVec(vector) if vector.item().typ().is_byte() => {
            (quote!(molecule::lazy_reader::Cursor), CursorValue::Bytes)
        }
        TopDecl::Option_(option) => {
            let (native, value) = cursor_type(option.item().typ(), ast, union_payload);
            (
                quote!(::core::option::Option<#native>),
                CursorValue::Optional(Box::new(value)),
            )
        }
        TopDecl::Union(_) => (lazy_type_path(typ, ast), CursorValue::TryInto),
        _ => (lazy_type_path(typ, ast), CursorValue::Into),
    }
}

fn generate_rust_slice_by(
    index: usize,
    field_count: usize,
    field_sizes: Option<&[usize]>,
    exact: bool,
) -> TokenStream {
    match (field_sizes, exact) {
        (Some(fs), true) => {
            let total: usize = fs.iter().sum();
            let offset: usize = fs[..index].iter().sum();
            let size = fs[index];
            quote!(fixed_field_exact(#total, #offset, #size))
        }
        (Some(fs), false) => {
            let offset: usize = fs[..index].iter().sum();
            let size = fs[index];
            quote!(slice_by_offset(#offset, #size))
        }
        (None, true) => quote!(table_field_exact(#field_count, #index)),
        (None, false) => quote!(table_slice_by_index(#index)),
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

fn verify_cursor(typ: &TopDecl, cur: TokenStream, ast: &Ast, exact: bool) -> TokenStream {
    match typ {
        TopDecl::Primitive(_) => quote!((#cur).verify_fixed_size(1)?;),
        TopDecl::Option_(v) => {
            let inner = verify_cursor(v.item().typ().as_ref(), quote!(val), ast, exact);
            quote!({
                let val = #cur;
                if !val.option_is_none() {
                    #inner
                }
            })
        }
        TopDecl::Union(_) => {
            let type_path = lazy_type_path(typ, ast);
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
            if exact {
                quote!((#cur).fixvec_len_exact(#item_size)?;)
            } else {
                quote!((#cur).verify_fixvec(#item_size)?;)
            }
        }
        TopDecl::DynVec(_) | TopDecl::Table(_) => {
            let type_path = lazy_type_path(typ, ast);
            quote!(<#type_path as ::core::convert::From<molecule::lazy_reader::Cursor>>::from(#cur).verify(compatible)?;)
        }
    }
}
