use std::io;

use case::CaseExt;
use proc_macro2 as m4;
use quote::quote;

use super::Generator;
use crate::ast::verified as ast;

const ATOM_NAME: &str = "u8";

impl Generator {
    pub(crate) fn generate_rust<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let code = quote!(
            use molecule::prelude::{Entity as _, Reader as _};
        );
        write!(writer, "{}", code)?;
        for decl in &self.ast.decls[..] {
            match decl.typ {
                ast::TopDeclType::Option_(ref info) => {
                    gen_option(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Union(ref info) => {
                    gen_union(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Array(ref info) => {
                    gen_array(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Struct(ref info) => {
                    gen_struct(writer, &decl.name, info)?;
                }
                ast::TopDeclType::FixedVector(ref info) => {
                    gen_fix_vec(writer, &decl.name, info)?;
                }
                ast::TopDeclType::DynamicVector(ref info) => {
                    gen_dyn_vec(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Table(ref info) => {
                    gen_table(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Atom => unreachable!(),
            };
        }
        Ok(())
    }
}

/*
 * Utilities
 */

fn ident_name(name: &str, suffix: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    if name == ast::ATOM_NAME {
        m4::Ident::new(ATOM_NAME, span)
    } else {
        m4::Ident::new(&format!("{}{}", name, suffix).to_camel(), span)
    }
}

fn entity_name(name: &str) -> m4::Ident {
    ident_name(name, "")
}

fn reader_name(name: &str) -> m4::Ident {
    ident_name(name, "Reader")
}

fn entity_union_name(name: &str) -> m4::Ident {
    ident_name(name, "Union")
}

fn reader_union_name(name: &str) -> m4::Ident {
    ident_name(name, "ReaderUnion")
}

fn union_item_name(name: &str) -> m4::Ident {
    ident_name(name, "")
}

fn builder_name(name: &str) -> m4::Ident {
    ident_name(name, "Builder")
}

fn usize_lit(num: usize) -> m4::Literal {
    m4::Literal::usize_unsuffixed(num)
}

fn func_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    m4::Ident::new(&name.to_snake(), span)
}

/*
 * Common
 */

fn def_entity_and_reader<W>(writer: &mut W, origin_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let code = quote!(
        #[derive(Debug, Default, Clone)]
        pub struct #entity(molecule::bytes::Bytes);
        #[derive(Debug, Clone, Copy)]
        pub struct #reader<'r>(&'r [u8]);

    );
    write!(writer, "{}", code)
}

fn def_items_for_union<W>(writer: &mut W, origin_name: &str, info: &ast::Union) -> io::Result<()>
where
    W: io::Write,
{
    let entity_union = entity_union_name(origin_name);
    let reader_union = reader_union_name(origin_name);
    let entity_unions = &vec![entity_union_name(origin_name); info.inner.len()];
    let reader_unions = &vec![reader_union_name(origin_name); info.inner.len()];
    let (ref entity_inners, ref reader_inners, ref union_items, ref union_ids) =
        info.inner.iter().enumerate().fold(
            (
                Vec::with_capacity(info.inner.len()),
                Vec::with_capacity(info.inner.len()),
                Vec::with_capacity(info.inner.len()),
                Vec::with_capacity(info.inner.len()),
            ),
            |(mut entity_inners, mut reader_inners, mut union_items, mut union_ids),
             (index, inner)| {
                let entity_name = entity_name(&inner.typ.name);
                let reader_name = reader_name(&inner.typ.name);
                let item_name = union_item_name(&inner.typ.name);
                let item_id = usize_lit(index);
                entity_inners.push(entity_name);
                reader_inners.push(reader_name);
                union_items.push(item_name);
                union_ids.push(item_id);
                (entity_inners, reader_inners, union_items, union_ids)
            },
        );
    {
        let entity_default = {
            let inner = &info.inner[0];
            let item_name = union_item_name(&inner.typ.name);
            quote!(#item_name(::std::default::Default::default()))
        };
        let code = quote!(
            #[derive(Debug, Clone)]
            pub enum #entity_union {
                #( #union_items(#entity_inners), )*
            }
            #[derive(Debug)]
            pub enum #reader_union<'r> {
                #( #union_items(#reader_inners<'r>), )*
            }

            impl ::std::default::Default for #entity_union {
                fn default() -> Self {
                    #entity_union::#entity_default
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    for (item_name, entity_name) in union_items.iter().zip(entity_inners.iter()) {
        let code = quote!(
            impl ::std::convert::From<#entity_name> for #entity_union {
                fn from(item: #entity_name) -> Self {
                    #entity_union::#item_name(item)
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    for (item_name, reader_name) in union_items.iter().zip(reader_inners.iter()) {
        let code = quote!(
            impl<'r> ::std::convert::From<#reader_name<'r>> for #reader_union<'r> {
                fn from(item: #reader_name<'r>) -> Self {
                    #reader_union::#item_name(item)
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    {
        let code = quote!(
            impl #entity_union {
                pub fn as_slice(&self) -> &[u8] {
                    match self {
                        #( #entity_unions::#union_items(item) => item.as_slice(), )*
                    }
                }
                pub fn item_id(&self) -> usize {
                    match self {
                        #( #entity_unions::#union_items(_) => #union_ids, )*
                    }
                }
            }
            impl<'r> #reader_union<'r> {
                pub fn as_slice(&self) -> &[u8] {
                    match self {
                        #( #reader_unions::#union_items(item) => item.as_slice(), )*
                    }
                }
                pub fn item_id(&self) -> usize {
                    match self {
                        #( #reader_unions::#union_items(_) => #union_ids, )*
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    Ok(())
}

fn impl_trait_entity<W>(writer: &mut W, origin_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let builder = builder_name(origin_name);
    let code = quote!(
        impl molecule::prelude::Entity for #entity {
            type Builder = #builder;
            fn new_unchecked(data: molecule::bytes::Bytes) -> Self {
                #entity(data)
            }
            fn as_slice(&self) -> &[u8] {
                &self.0[..]
            }
            fn from_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
                #reader::from_slice(slice).map(|reader| reader.to_entity())
            }
            fn new_builder() -> Self::Builder {
                ::std::default::Default::default()
            }
        }
    );
    write!(writer, "{}", code)
}

fn impl_entity<W>(writer: &mut W, origin_name: &str, funcs: Vec<m4::TokenStream>) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let code = quote!(
        impl #entity {
            pub fn as_reader(&self) -> #reader<'_> {
                #reader::new_unchecked(self.as_slice())
            }
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

fn impl_trait_reader<W>(writer: &mut W, origin_name: &str, funcs: m4::TokenStream) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let code = quote!(
        impl<'r> molecule::prelude::Reader<'r> for #reader<'r> {
            type Entity = #entity;
            fn to_entity(&self) -> Self::Entity {
                #entity::new_unchecked(self.as_slice().into())
            }
            fn new_unchecked(slice: &'r [u8]) -> Self {
                #reader(slice)
            }
            fn as_slice(&self) -> &[u8] {
                self.0
            }
            #funcs
        }
    );
    write!(writer, "{}", code)
}

fn impl_reader<W>(writer: &mut W, origin_name: &str, funcs: Vec<m4::TokenStream>) -> io::Result<()>
where
    W: io::Write,
{
    let reader = reader_name(origin_name);
    let code = quote!(
        impl<'r> #reader<'r> {
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

fn def_builder_for_option<W>(
    writer: &mut W,
    origin_name: &str,
    info: &ast::Option_,
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let inner = entity_name(&info.typ.name);
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder (Option<#inner>);
    );
    write!(writer, "{}", code)
}

fn def_builder_for_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let builder_string = builder.to_string();
    let inner = entity_name(&info.typ.name);
    let item_count = usize_lit(info.item_count);
    let code = quote!(
        pub struct #builder ([#inner; #item_count]);

        impl ::std::fmt::Debug for #builder {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{} ({:?})", #builder_string, &self.0[..])
            }
        }

    );
    write!(writer, "{}", code)?;
    let code = if info.typ.is_atom() {
        quote!(
            impl ::std::default::Default for #builder {
                fn default() -> Self {
                    #builder ([0; #item_count])
                }
            }
        )
    } else {
        let inner_array = (0..info.item_count)
            .map(|_| inner.clone())
            .collect::<Vec<_>>();
        quote!(
            impl ::std::default::Default for #builder {
                fn default() -> Self {
                    #builder([#(#inner_array::default(), )*])
                }
            }
        )
    };
    write!(writer, "{}", code)
}

fn def_builder_for_union<W>(writer: &mut W, origin_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let entity_union = entity_union_name(origin_name);
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder (#entity_union);
    );
    write!(writer, "{}", code)
}

fn def_builder_for_struct_or_table<W>(
    writer: &mut W,
    origin_name: &str,
    inner: &[ast::FieldDecl],
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let fields = inner.iter().map(|f| {
        let field_name = func_name(&f.name);
        let field_type = entity_name(&f.typ.name);
        quote!(#field_name: #field_type,)
    });
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder { #( #fields )* }
    );
    write!(writer, "{}", code)
}

fn def_builder_for_vector<W>(writer: &mut W, origin_name: &str, inner_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let inner = entity_name(&inner_name);
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder (Vec<#inner>);
    );
    write!(writer, "{}", code)
}

fn impl_trait_builder<W>(
    writer: &mut W,
    origin_name: &str,
    funcs: m4::TokenStream,
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let entity = entity_name(origin_name);
    let code = quote!(
        impl molecule::prelude::Builder for #builder {
            type Entity = #entity;
            #funcs
            fn build(&self) -> ::std::io::Result<Self::Entity> {
                let mut inner = Vec::with_capacity(self.expected_length());
                self.write(&mut inner)?;
                Ok(#entity::new_unchecked(inner.into()))
            }
        }
    );
    write!(writer, "{}", code)
}

fn impl_builder<W>(writer: &mut W, origin_name: &str, funcs: Vec<m4::TokenStream>) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let code = quote!(
        impl #builder {
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

/*
 * Core
 */

fn gen_option<W>(writer: &mut W, origin_name: &str, info: &ast::Option_) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_builder_for_option(writer, origin_name, info)?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                pub fn is_none(&self) -> bool {
                    self.0.is_empty()
                }

                pub fn is_some(&self) -> bool {
                    !self.0.is_empty()
                }
            );
            funcs.push(code);
        }
        {
            let inner = entity_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn get(&self) -> Option<#inner> {
                        if self.is_none() {
                            None
                        } else {
                            Some(self.0[0])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn get(&self) -> Option<#inner> {
                        if self.is_none() {
                            None
                        } else {
                            Some(#inner::new_unchecked(self.0.clone()))
                        }
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let inner = reader_name(&info.typ.name);
        if info.typ.is_atom() {
            quote!(
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    if slice.len() > 1 {
                        let err = VerificationError::TotalSizeNotAsExpected(
                            #reader_string.to_owned(), 0, 1, slice.len());
                        Err(err)?;
                    }
                    Ok(())
                }
            )
        } else {
            quote!(
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    if !slice.is_empty() {
                        #inner::verify(&slice[..])?;
                    }
                    Ok(())
                }
            )
        }
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                pub fn is_none(&self) -> bool {
                    self.0.is_empty()
                }

                pub fn is_some(&self) -> bool {
                    !self.0.is_empty()
                }
            );
            funcs.push(code);
        }
        {
            let inner = reader_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn get(&self) -> Option<#inner> {
                        if self.is_none() {
                            None
                        } else {
                            Some(self.0[0])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn get(&self) -> Option<#inner<'_>> {
                        if self.is_none() {
                            None
                        } else {
                            Some(#inner::new_unchecked(self.as_slice()))
                        }
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        let write_inner = if info.typ.is_atom() {
            quote!(if let Some(inner) = self.0 {
                writer.write_all(&[inner])
            } else {
                Ok(())
            })
        } else {
            quote!(if let Some(ref inner) = self.0 {
                writer.write_all(inner.as_slice())
            } else {
                Ok(())
            })
        };
        let expected_length = if info.typ.is_atom() {
            quote!(if let Some(_) = self.0 { 1 } else { 0 })
        } else {
            quote!(if let Some(ref inner) = self.0 {
                inner.as_slice().len()
            } else {
                0
            })
        };
        quote!(
            fn expected_length(&self) -> usize {
                #expected_length
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #write_inner
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        let code = quote!(
            pub fn set(mut self, v: #inner) -> Self {
                self.0 = Some(v);
                self
            }
            pub fn unset(mut self) -> Self {
                self.0 = None;
                self
            }
        );
        funcs.push(code);
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

fn gen_union<W>(writer: &mut W, origin_name: &str, info: &ast::Union) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_items_for_union(writer, origin_name, info)?;
    def_builder_for_union(writer, origin_name)?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let item_count = usize_lit(info.inner.len());
            let code = quote!(
                pub const ITEM_COUNT: usize = #item_count;

                pub fn item_id(&self) -> usize {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    u32::from_le(ptr[0]) as usize
                }
            );
            funcs.push(code);
        }
        {
            let entity_union = entity_union_name(origin_name);
            let match_stmts = info.inner.iter().enumerate().map(|(index, inner)| {
                let item_id = usize_lit(index);
                let inner = entity_name(&inner.typ.name);
                quote!(#item_id => #inner::new_unchecked(inner).into(),)
            });
            let code = quote!(
                pub fn get(&self) -> #entity_union {
                    let inner = self.0.slice_from(4);
                    match self.item_id() {
                        #( #match_stmts )*
                        _ => unreachable!(),
                    }
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let item_count = usize_lit(info.inner.len());
        let verify_inners = info.inner.iter().enumerate().map(|(index, inner)| {
            let item_id = usize_lit(index);
            let inner = reader_name(&inner.typ.name);
            quote!(#item_id => #inner::verify(&slice[4..]),)
        });
        quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        #reader_string.to_owned(), 4, slice.len());
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let item_id = u32::from_le(ptr[0]) as usize;
                match item_id {
                    #( #verify_inners )*
                    _ => {
                        let err = VerificationError::UnknownItem(
                            #reader_string.to_owned(), #item_count, item_id);
                        Err(err)
                    },
                }?;
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let item_count = usize_lit(info.inner.len());
            let code = quote!(
                pub const ITEM_COUNT: usize = #item_count;

                pub fn item_id(&self) -> usize {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    u32::from_le(ptr[0]) as usize
                }
            );
            funcs.push(code);
        }
        {
            let reader_union = reader_union_name(origin_name);
            let match_stmts = info.inner.iter().enumerate().map(|(index, inner)| {
                let item_id = usize_lit(index);
                let inner = reader_name(&inner.typ.name);
                quote!(#item_id => #inner::new_unchecked(inner).into(),)
            });
            let code = quote!(
                pub fn get(&self) -> #reader_union<'_> {
                    let inner = &self.as_slice()[4..];
                    match self.item_id() {
                        #( #match_stmts )*
                        _ => unreachable!(),
                    }
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        quote!(
            fn expected_length(&self) -> usize {
                4 + self.0.as_slice().len()
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let item_id = (self.0.item_id() as u32).to_le_bytes();
                writer.write_all(&item_id[..])?;
                writer.write_all(self.0.as_slice())
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let entity_union = entity_union_name(origin_name);
            let code = quote!(
                pub fn set<I>(mut self, v: I) -> Self
                where
                    I: ::std::convert::Into<#entity_union>
                {
                    self.0 = v.into();
                    self
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

fn gen_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_builder_for_array(writer, origin_name, info)?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        {
            let total_size = usize_lit(info.item_size * info.item_count);
            let item_size = usize_lit(info.item_size);
            let item_count = usize_lit(info.item_count);
            let code = quote!(
                pub const TOTAL_SIZE: usize = #total_size;
                pub const ITEM_SIZE: usize = #item_size;
                pub const ITEM_COUNT: usize = #item_count;
            );
            funcs.push(code);
        }
        for idx in 0..info.item_count {
            let start = usize_lit(idx * info.item_size);
            let func = func_name(&format!("nth{}", idx));
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn #func(&self) -> #inner {
                        self.0[#start]
                    }
                )
            } else {
                let end = usize_lit((idx + 1) * info.item_size);
                quote!(
                    pub fn #func(&self) -> #inner {
                        #inner::new_unchecked(self.0.slice(#start, #end))
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let inner = reader_name(&info.typ.name);
        let total_size = usize_lit(info.item_size * info.item_count);
        let verify_inners = if info.typ.is_atom() {
            Vec::new()
        } else {
            (0..info.item_count)
                .map(|i| {
                    let start = usize_lit(info.item_size * i);
                    let end = usize_lit(info.item_size * (i + 1));
                    quote!(#inner::verify(&slice[#start..#end])?;)
                })
                .collect()
        };
        quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() != #total_size {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(), #total_size, slice.len());
                    Err(err)?;
                }
                #( #verify_inners )*
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = reader_name(&info.typ.name);
        {
            let total_size = usize_lit(info.item_size * info.item_count);
            let item_size = usize_lit(info.item_size);
            let item_count = usize_lit(info.item_count);
            let code = quote!(
                pub const TOTAL_SIZE: usize = #total_size;
                pub const ITEM_SIZE: usize = #item_size;
                pub const ITEM_COUNT: usize = #item_count;
            );
            funcs.push(code);
        }
        for idx in 0..info.item_count {
            let start = usize_lit(idx * info.item_size);
            let func = func_name(&format!("nth{}", idx));
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn #func(&self) -> #inner {
                        self.as_slice()[#start]
                    }
                )
            } else {
                let end = usize_lit((idx + 1) * info.item_size);
                quote!(
                    pub fn #func(&self) -> #inner<'_> {
                        #inner::new_unchecked(&self.as_slice()[#start..#end])
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        let total_size = usize_lit(info.item_size * info.item_count);
        let write_inners = if info.typ.is_atom() {
            quote!(writer.write_all(&self.0)?;)
        } else {
            let idx = (0..info.item_count).map(usize_lit).collect::<Vec<_>>();
            quote!(#( writer.write_all(self.0[#idx].as_slice())?; )*)
        };
        quote!(
            fn expected_length(&self) -> usize {
                #total_size
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #write_inners
                Ok(())
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        for idx in 0..info.item_count {
            let index = usize_lit(idx);
            let func = func_name(&format!("nth{}", idx));
            let code = quote!(
                pub fn #func(mut self, v: #inner) -> Self {
                    self.0[#index] = v;
                    self
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

fn gen_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_builder_for_struct_or_table(writer, origin_name, &info.inner[..])?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let total_size = usize_lit(info.field_size.iter().sum());
            let field_count = usize_lit(info.inner.len());
            let fields_size = info.field_size.iter().map(|x| usize_lit(*x as usize));
            let code = quote!(
                pub const TOTAL_SIZE: usize = #total_size;
                pub const FIELD_COUNT: usize = #field_count;
                pub const FIELDS_SIZE: [usize; #field_count]= [ #( #fields_size, )* ];
            );
            funcs.push(code);
        }
        {
            let mut offset = 0;
            for (f, s) in info.inner.iter().zip(info.field_size.iter()) {
                let func = func_name(&f.name);
                let inner = entity_name(&f.typ.name);
                let start = usize_lit(offset);
                offset += s;
                let code = if f.typ.is_atom() {
                    quote!(
                        pub fn #func(&self) -> #inner {
                            self.0[#start]
                        }
                    )
                } else {
                    let end = usize_lit(offset);
                    quote!(
                        pub fn #func(&self) -> #inner {
                            #inner::new_unchecked(self.0.slice(#start, #end))
                        }
                    )
                };
                funcs.push(code);
            }
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let total_size = usize_lit(info.field_size.iter().sum());
        let verify_fields = {
            let mut offset = 0;
            let mut codes = Vec::with_capacity(info.field_size.len());
            for (f, s) in info.inner.iter().zip(info.field_size.iter()) {
                let field = reader_name(&f.typ.name);
                let start = usize_lit(offset);
                offset += s;
                let end = usize_lit(offset);
                if !f.typ.is_atom() {
                    let code = quote!(
                        #field::verify(&slice[#start..#end])?;
                    );
                    codes.push(code);
                }
            }
            codes
        };
        quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() != #total_size {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(), #total_size, slice.len());
                    Err(err)?;
                }
                #( #verify_fields )*
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let total_size = usize_lit(info.field_size.iter().sum());
            let field_count = usize_lit(info.inner.len());
            let fields_size = info.field_size.iter().map(|x| usize_lit(*x as usize));
            let code = quote!(
                pub const TOTAL_SIZE: usize = #total_size;
                pub const FIELD_COUNT: usize = #field_count;
                pub const FIELDS_SIZE: [usize; #field_count]= [ #( #fields_size, )* ];
            );
            funcs.push(code);
        }
        {
            let mut offset = 0;
            for (f, s) in info.inner.iter().zip(info.field_size.iter()) {
                let func = func_name(&f.name);
                let inner = reader_name(&f.typ.name);
                let start = usize_lit(offset);
                offset += s;
                let code = if f.typ.is_atom() {
                    quote!(
                        pub fn #func(&self) -> #inner {
                            self.as_slice()[#start]
                        }
                    )
                } else {
                    let end = usize_lit(offset);
                    quote!(
                        pub fn #func(&self) -> #inner<'_> {
                            #inner::new_unchecked(&self.as_slice()[#start..#end])
                        }
                    )
                };
                funcs.push(code);
            }
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        let total_size = usize_lit(info.field_size.iter().sum());
        let fields = info.inner.iter().map(|f| {
            let field_name = func_name(&f.name);
            if f.typ.is_atom() {
                quote!(writer.write_all(&[self.#field_name])?;)
            } else {
                quote!(writer.write_all(self.#field_name.as_slice())?;)
            }
        });
        quote!(
            fn expected_length(&self) -> usize {
                #total_size
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #( #fields )*
                Ok(())
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        for f in info.inner.iter() {
            let field_name = func_name(&f.name);
            let field_type = entity_name(&f.typ.name);
            let code = quote!(
                pub fn #field_name(mut self, v: #field_type) -> Self {
                    self.#field_name = v;
                    self
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

fn gen_fix_vec<W>(writer: &mut W, origin_name: &str, info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_builder_for_vector(writer, origin_name, &info.typ.name)?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let item_size = usize_lit(info.item_size);
        {
            let code = quote!(
                pub const ITEM_SIZE: usize = #item_size;

                pub fn len(&self) -> usize {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    u32::from_le(ptr[0]) as usize
                }
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
            );
            funcs.push(code);
        }
        {
            let item_size = usize_lit(info.item_size);
            let inner = entity_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner> {
                        if idx >= self.len() {
                            None
                        } else {
                            Some(self.0[4+idx])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner> {
                        if idx >= self.len() {
                            None
                        } else {
                            let start = 4 + idx * #item_size;
                            let end = start + #item_size;
                            Some(#inner::new_unchecked(self.0.slice(start, end)))
                        }
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let inner = reader_name(&info.typ.name);
        let item_size = usize_lit(info.item_size);
        let verify_inners = if info.typ.is_atom() {
            quote!()
        } else {
            quote!(
                for i in 0..item_count {
                    let start = #item_size * i;
                    let end = start + #item_size;
                    #inner::verify(&slice[start..end])?;
                }
            )
        };
        quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err =
                        VerificationError::HeaderIsBroken(#reader_string.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let item_count = u32::from_le(ptr[0]) as usize;
                let expected = 4 + #item_size * item_count;
                if len != expected {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(),
                        expected,
                        len,
                    );
                    Err(err)?;
                }
                #verify_inners
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let item_size = usize_lit(info.item_size);
        {
            let code = quote!(
                pub const ITEM_SIZE: usize = #item_size;

                pub fn len(&self) -> usize {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    u32::from_le(ptr[0]) as usize
                }
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
            );
            funcs.push(code);
        }
        {
            let item_size = usize_lit(info.item_size);
            let inner = reader_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner> {
                        if idx >= self.len() {
                            None
                        } else {
                            Some(self.as_slice()[4+idx])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner<'_>> {
                        if idx >= self.len() {
                            None
                        } else {
                            let start = 4 + idx * #item_size;
                            let end = start + #item_size;
                            Some(#inner::new_unchecked(&self.as_slice()[start..end]))
                        }
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        let item_size = usize_lit(info.item_size);
        let write_inners = if info.typ.is_atom() {
            quote!(writer.write_all(&self.0)?;)
        } else {
            quote!(for inner in &self.0[..] {
                writer.write_all(inner.as_slice())?;
            })
        };
        quote!(
            fn expected_length(&self) -> usize {
                4 + #item_size * self.0.len()
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let len = (self.0.len() as u32).to_le_bytes();
                writer.write_all(&len)?;
                #write_inners
                Ok(())
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        let code = quote!(
            pub fn push(mut self, v: #inner) -> Self {
                self.0.push(v);
                self
            }
        );
        funcs.push(code);
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

fn gen_dyn_vec<W>(writer: &mut W, origin_name: &str, info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_builder_for_vector(writer, origin_name, &info.typ.name)?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                pub fn offsets(&self) -> &[u32] {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    &ptr[1..]
                }
            );
            funcs.push(code);
        }
        {
            let code = quote!(
                pub fn len(&self) -> usize {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    let bytes_len = u32::from_le(ptr[0]) as usize;
                    if bytes_len == 4 {
                        0
                    } else {
                        let first = u32::from_le(ptr[1]) as usize;
                        (first - 4) / 4
                    }
                }
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
            );
            funcs.push(code);
        }
        {
            let inner = entity_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner> {
                        let len = self.len();
                        if idx >= len {
                            None
                        } else {
                            let offsets = self.offsets();
                            let offset = u32::from_le(offsets[idx]) as usize;
                            Some(self.0[offset])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner> {
                        let len = self.len();
                        if idx >= len {
                            None
                        } else {
                            let offsets = self.offsets();
                            let start = u32::from_le(offsets[idx]) as usize;
                            if idx == len - 1 {
                                Some(#inner::new_unchecked(self.0.slice_from(start)))
                            } else {
                                let end = u32::from_le(offsets[idx+1]) as usize;
                                Some(#inner::new_unchecked(self.0.slice(start, end)))
                            }
                        }
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let inner = reader_name(&info.typ.name);
        quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        #reader_string.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let total_size = u32::from_le(ptr[0]) as usize;
                if total_size != len {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(), total_size, len);
                    Err(err)?;
                }
                if total_size == 4 {
                    return Ok(());
                }
                if total_size < 4 + 4 {
                    let err = VerificationError::DataIsShort(
                        #reader_string.to_owned(), 8, total_size);
                    Err(err)?;
                }
                let offset_first = u32::from_le(ptr[1]) as usize;
                if offset_first % 4 != 0 {
                    let err = VerificationError::FirstOffsetIsBroken(
                        #reader_string.to_owned(), offset_first);
                    Err(err)?;
                }
                if offset_first < 4 + 4 {
                    let err = VerificationError::FirstOffsetIsShort(
                        #reader_string.to_owned(), 8, offset_first);
                    Err(err)?;
                }
                let item_count = offset_first / 4 - 1;
                let expected = 4 + 4 * item_count;
                if total_size < expected {
                    let err = VerificationError::DataIsShort(
                        #reader_string.to_owned(), expected, total_size);
                    Err(err)?;
                }
                let mut offsets: Vec<usize> = ptr[1..(item_count+1)]
                    .iter()
                    .map(|x| u32::from_le(*x) as usize)
                    .collect();
                offsets.push(total_size);
                if offsets.windows(2).any(|i| i[0] > i[1]) {
                    let err = VerificationError::OffsetsNotMatch(#reader_string.to_owned());
                    Err(err)?;
                }
                for i in 0..=(offsets.len()-2) {
                    let start = offsets[i];
                    let end = offsets[i+1];
                    #inner::verify(&slice[start..end])?;
                }
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                pub fn offsets(&self) -> &[u32] {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    &ptr[1..]
                }
            );
            funcs.push(code);
        }
        {
            let code = quote!(
                pub fn len(&self) -> usize {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    let bytes_len = u32::from_le(ptr[0]) as usize;
                    if bytes_len == 4 {
                        0
                    } else {
                        let first = u32::from_le(ptr[1]) as usize;
                        (first - 4) / 4
                    }
                }
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
            );
            funcs.push(code);
        }
        {
            let inner = reader_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner> {
                        let len = self.len();
                        if idx >= len {
                            None
                        } else {
                            let offsets = self.offsets();
                            let offset = u32::from_le(offsets[idx]) as usize;
                            Some(self.as_slice()[offset])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn get(&self, idx: usize) -> Option<#inner<'_>> {
                        let len = self.len();
                        if idx >= len {
                            None
                        } else {
                            let offsets = self.offsets();
                            let start = u32::from_le(offsets[idx]) as usize;
                            if idx == len - 1 {
                                Some(#inner::new_unchecked(&self.as_slice()[start..]))
                            } else {
                                let end = u32::from_le(offsets[idx+1]) as usize;
                                Some(#inner::new_unchecked(&self.as_slice()[start..end]))
                            }
                        }
                    }
                )
            };
            funcs.push(code);
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        quote!(
            fn expected_length(&self) -> usize {
                let len_header = 4 + 4 * self.0.len();
                len_header + self.0.iter().map(|inner| inner.as_slice().len()).sum::<usize>()
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let len = (self.expected_length() as u32).to_le_bytes();
                writer.write_all(&len[..])?;
                let mut offset = 4 + 4 * self.0.len();
                for inner in &self.0[..] {
                    let tmp = (offset as u32).to_le_bytes();
                    writer.write_all(&tmp[..])?;
                    offset += inner.as_slice().len();
                }
                for inner in &self.0[..] {
                    writer.write_all(inner.as_slice())?;
                }
                Ok(())
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        let code = quote!(
            pub fn push(mut self, v: #inner) -> Self {
                self.0.push(v);
                self
            }
        );
        funcs.push(code);
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

fn gen_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    def_builder_for_struct_or_table(writer, origin_name, &info.inner[..])?;
    impl_trait_entity(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let field_count = usize_lit(info.inner.len());
            let code = quote!(pub const FIELD_COUNT: usize = #field_count;);
            funcs.push(code);
        }
        {
            let code = quote!(
                pub fn field_offsets(&self) -> (usize, &[u32]) {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    let first = u32::from_le(ptr[1]) as usize;
                    let count = (first - 4) / 4;
                    (count, &ptr[1..])
                }
            );
            funcs.push(code);
        }
        {
            let field_count = usize_lit(info.inner.len());
            for (i, f) in info.inner.iter().enumerate() {
                let func = func_name(&f.name);
                let inner = entity_name(&f.typ.name);
                let start = usize_lit(i);
                let code = if f.typ.is_atom() {
                    quote!(
                        pub fn #func(&self) -> #inner {
                            let (_, offsets) = Self::field_offsets(self);
                            let offset = u32::from_le(offsets[#start]) as usize;
                            self.0[offset]
                        }
                    )
                } else if i == info.inner.len() - 1 {
                    quote!(
                        pub fn #func(&self) -> #inner {
                            let (count, offsets) = Self::field_offsets(self);
                            let start = u32::from_le(offsets[#start]) as usize;
                            if count == #field_count {
                                #inner::new_unchecked(self.0.slice_from(start))
                            } else {
                                let end = u32::from_le(offsets[#start+1]) as usize;
                                #inner::new_unchecked(self.0.slice(start, end))
                            }
                        }
                    )
                } else {
                    quote!(
                        pub fn #func(&self) -> #inner {
                            let (_, offsets) = Self::field_offsets(self);
                            let start = u32::from_le(offsets[#start]) as usize;
                            let end = u32::from_le(offsets[#start+1]) as usize;
                            #inner::new_unchecked(self.0.slice(start, end))
                        }
                    )
                };
                funcs.push(code);
            }
        }
        funcs
    };
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let field_count = usize_lit(info.inner.len());
        let verify_fields = info.inner.iter().enumerate().map(|(i, f)| {
            let field = reader_name(&f.typ.name);
            let start = usize_lit(i);
            let end = usize_lit(i + 1);
            if f.typ.is_atom() {
                quote!(
                    if offsets[#start] + 1 != offsets[#end] {
                        let err = VerificationError::FieldIsBroken(
                            #reader_string.to_owned(), #start);
                        Err(err)?;
                    }
                )
            } else {
                quote!(
                    #field::verify(&slice[offsets[#start]..offsets[#end]])?;
                )
            }
        });
        quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        #reader_string.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let total_size = u32::from_le(ptr[0]) as usize;
                if total_size != len {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(), total_size, len);
                    Err(err)?;
                }
                let expected = 4 + 4 * #field_count;
                if total_size < expected {
                    let err = VerificationError::HeaderIsBroken(
                        #reader_string.to_owned(), expected, total_size);
                    Err(err)?;
                }
                let mut offsets: Vec<usize> = ptr[1..=#field_count]
                    .iter()
                    .map(|x| u32::from_le(*x) as usize)
                    .collect();
                if offsets[0] != expected {
                    let err = VerificationError::FirstOffsetIsShort(
                        #reader_string.to_owned(), expected, offsets[0]);
                    Err(err)?;
                }
                offsets.push(total_size);
                if offsets.windows(2).any(|i| i[0] > i[1]) {
                    let err = VerificationError::OffsetsNotMatch(#reader_string.to_owned());
                    Err(err)?;
                }
                #( #verify_fields )*
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let field_count = usize_lit(info.inner.len());
            let code = quote!(pub const FIELD_COUNT: usize = #field_count;);
            funcs.push(code);
        }
        {
            let code = quote!(
                pub fn field_offsets(&self) -> (usize, &[u32]) {
                    let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                    let first = u32::from_le(ptr[1]) as usize;
                    let count = (first - 4) / 4;
                    (count, &ptr[1..])
                }
            );
            funcs.push(code);
        }
        {
            let field_count = usize_lit(info.inner.len());
            for (i, f) in info.inner.iter().enumerate() {
                let func = func_name(&f.name);
                let inner = reader_name(&f.typ.name);
                let start = usize_lit(i);
                let code = if f.typ.is_atom() {
                    quote!(
                        pub fn #func(&self) -> #inner {
                            let (_, offsets) = Self::field_offsets(self);
                            let offset = u32::from_le(offsets[#start]) as usize;
                            self.as_slice()[offset]
                        }
                    )
                } else if i == info.inner.len() - 1 {
                    quote!(
                        pub fn #func(&self) -> #inner<'_> {
                            let (count, offsets) = Self::field_offsets(self);
                            let start = u32::from_le(offsets[#start]) as usize;
                            if count == #field_count {
                                #inner::new_unchecked(&self.as_slice()[start..])
                            } else {
                                let end = u32::from_le(offsets[#start+1]) as usize;
                                #inner::new_unchecked(&self.as_slice()[start..end])
                            }
                        }
                    )
                } else {
                    quote!(
                        pub fn #func(&self) -> #inner<'_> {
                            let (_, offsets) = Self::field_offsets(self);
                            let start = u32::from_le(offsets[#start]) as usize;
                            let end = u32::from_le(offsets[#start+1]) as usize;
                            #inner::new_unchecked(&self.as_slice()[start..end])
                        }
                    )
                };
                funcs.push(code);
            }
        }
        funcs
    };
    impl_reader(writer, origin_name, funcs)?;
    let code = if info.inner.is_empty() {
        quote!(
            fn expected_length(&self) -> usize {
                4
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let len = 4u32.to_le_bytes();
                writer.write_all(&len[..])?;
                Ok(())
            }
        )
    } else {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        let mut lengths: Vec<m4::TokenStream> = Vec::new();
        let field_count = usize_lit(info.inner.len());
        for f in info.inner.iter() {
            let field_name = func_name(&f.name);
            let code = if f.typ.is_atom() {
                quote!(writer.write_all(&[self.#field_name])?;)
            } else {
                quote!(writer.write_all(self.#field_name.as_slice())?;)
            };
            fields.push(code);
            let code = if f.typ.is_atom() {
                quote!(1)
            } else {
                quote!(self.#field_name.as_slice().len())
            };
            lengths.push(code);
        }
        let lengths1 = &lengths;
        let lengths2 = &lengths;
        quote!(
            fn expected_length(&self) -> usize {
                let len_header = 4 + #field_count * 4;
                len_header #(+ #lengths1)*
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let len = (self.expected_length() as u32).to_le_bytes();
                writer.write_all(&len[..])?;
                let mut offset = 4 + #field_count * 4;
                #({
                    let tmp = (offset as u32).to_le_bytes();
                    writer.write_all(&tmp[..])?;
                    offset += #lengths2;
                })*
                let _ = offset;
                #( #fields )*
                Ok(())
            }
        )
    };
    impl_trait_builder(writer, origin_name, code)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        for f in info.inner.iter() {
            let field_name = func_name(&f.name);
            let field_type = entity_name(&f.typ.name);
            let code = quote!(
                pub fn #field_name(mut self, v: #field_type) -> Self {
                    self.#field_name = v;
                    self
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}
