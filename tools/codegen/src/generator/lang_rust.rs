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

fn reader_name(name: &str) -> m4::Ident {
    ident_name(name, "Reader")
}

fn entity_name(name: &str) -> m4::Ident {
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

fn def_molecule<W>(writer: &mut W, origin_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let builder = builder_name(origin_name);
    let code = quote!(
        #[derive(Debug, Default, Clone)]
        pub struct #entity(molecule::bytes::Bytes);
        #[derive(Debug)]
        pub struct #reader<'r>(&'r [u8]);

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
                std::default::Default::default()
            }
        }

        impl #entity {
            pub fn as_reader(&self) -> #reader<'_> {
                #reader::new_unchecked(self.as_slice())
            }
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
    let name = reader_name(origin_name);
    let code = quote!(
        impl<'r> #name<'r> {
            #( #funcs )*
        }
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
    let name = builder_name(origin_name);
    let entity = entity_name(origin_name);
    let code = quote!(
        impl molecule::prelude::Builder for #name {
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
    let name = builder_name(origin_name);
    let code = quote!(
        impl #name {
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

/*
 * Core
 */

fn gen_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    def_molecule(writer, origin_name)?;
    def_builder_for_array(writer, origin_name, info)?;
    {
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
        let code = quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() == #total_size {
                    #( #verify_inners )*
                    Ok(())
                } else {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(), #total_size, slice.len());
                    Err(err)
                }
            }
        );
        impl_trait_reader(writer, origin_name, code)?;
    }
    {
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
        impl_reader(writer, origin_name, funcs)?;
    }
    {
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
        impl_builder(writer, origin_name, funcs)?;
    }
    {
        let total_size = usize_lit(info.item_size * info.item_count);
        let write_inners = if info.typ.is_atom() {
            quote!(writer.write_all(&self.0)?;)
        } else {
            let idx = (0..info.item_count).map(usize_lit).collect::<Vec<_>>();
            quote!(#( writer.write_all(self.0[#idx].as_slice())?; )*)
        };
        let code = quote!(
            fn expected_length(&self) -> usize {
                #total_size
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #write_inners
                Ok(())
            }
        );
        impl_trait_builder(writer, origin_name, code)?;
    }
    writeln!(writer)
}

fn gen_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    def_molecule(writer, origin_name)?;
    def_builder_for_struct_or_table(writer, origin_name, &info.inner[..])?;
    {
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
        let code = quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() == #total_size {
                    #( #verify_fields )*
                    Ok(())
                } else {
                    let err = VerificationError::TotalSizeNotMatch(
                        #reader_string.to_owned(), #total_size, slice.len());
                    Err(err)
                }
            }
        );
        impl_trait_reader(writer, origin_name, code)?;
    }
    {
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
        impl_reader(writer, origin_name, funcs)?;
    }
    {
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
        impl_builder(writer, origin_name, funcs)?;
    }
    {
        let total_size = usize_lit(info.field_size.iter().sum());
        let fields = info.inner.iter().map(|f| {
            let field_name = func_name(&f.name);
            if f.typ.is_atom() {
                quote!(writer.write_all(&[self.#field_name])?;)
            } else {
                quote!(writer.write_all(self.#field_name.as_slice())?;)
            }
        });
        let code = quote!(
            fn expected_length(&self) -> usize {
                #total_size
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                #( #fields )*
                Ok(())
            }
        );
        impl_trait_builder(writer, origin_name, code)?;
    }
    writeln!(writer)
}

fn gen_fix_vec<W>(writer: &mut W, origin_name: &str, info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    def_molecule(writer, origin_name)?;
    def_builder_for_vector(writer, origin_name, &info.typ.name)?;
    {
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
                    let end = start + #item_size * i;
                    #inner::verify(&slice[start..end])?;
                }
            )
        };
        let code = quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err =
                        VerificationError::HeaderIsBroken(#reader_string.to_owned(), 4, len);
                    Err(err)
                } else {
                    let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                    let item_count = u32::from_le(ptr[0]) as usize;
                    let expected = 4 + #item_size * item_count;
                    if len == expected {
                        let err = VerificationError::TotalSizeNotMatch(
                            #reader_string.to_owned(),
                            expected,
                            len,
                        );
                        Err(err)
                    } else {
                        #verify_inners
                        Ok(())
                    }
                }
            }
        );
        impl_trait_reader(writer, origin_name, code)?;
    }
    {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let item_size = usize_lit(info.item_size);
        {
            let code = quote!(
                pub const ITEM_SIZE: usize = #item_size;

                pub fn item_count(&self) -> usize {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.as_slice()) };
                    u32::from_le(ptr[0]) as usize
                }
            );
            funcs.push(code);
        }
        {
            let item_size = usize_lit(info.item_size);
            let inner = reader_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn nth(&self, idx: usize) -> Option<#inner> {
                        if idx >= Self::item_count(self) {
                            None
                        } else {
                            Some(self.as_slice()[4+idx])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn nth(&self, idx: usize) -> Option<#inner<'_>> {
                        if idx >= Self::item_count(self) {
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
        impl_reader(writer, origin_name, funcs)?;
    }
    {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        let code = quote!(
            pub fn push(mut self, v: #inner) -> Self {
                self.0.push(v);
                self
            }
        );
        funcs.push(code);
        impl_builder(writer, origin_name, funcs)?;
    }
    {
        let item_size = usize_lit(info.item_size);
        let write_inners = if info.typ.is_atom() {
            quote!(writer.write_all(&self.0)?;)
        } else {
            quote!(for inner in &self.0[..] {
                writer.write_all(inner.as_slice())?;
            })
        };
        let code = quote!(
            fn expected_length(&self) -> usize {
                4 + #item_size * self.0.len()
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let len = (self.0.len() as u32).to_le_bytes();
                writer.write_all(&len)?;
                #write_inners
                Ok(())
            }
        );
        impl_trait_builder(writer, origin_name, code)?;
    }
    writeln!(writer)
}

fn gen_dyn_vec<W>(writer: &mut W, origin_name: &str, info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    def_molecule(writer, origin_name)?;
    def_builder_for_vector(writer, origin_name, &info.typ.name)?;
    {
        let reader = reader_name(origin_name);
        let reader_string = reader.to_string();
        let inner = reader_name(&info.typ.name);
        let code = quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        #reader_string.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
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
                if offsets.windows(2).any(|i| i[0] + 4 > i[1]) {
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
        );
        impl_trait_reader(writer, origin_name, code)?;
    }
    {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                pub fn item_offsets(&self) -> (usize, &[u32]) {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.as_slice()) };
                    let first = u32::from_le(ptr[1]) as usize;
                    let count = (first - 4) / 4;
                    (count, &ptr[1..])
                }
            );
            funcs.push(code);
        }
        {
            let inner = reader_name(&info.typ.name);
            let code = if info.typ.is_atom() {
                quote!(
                    pub fn nth(&self, idx: usize) -> Option<#inner> {
                        let (count, offsets) = Self::item_offsets(self);
                        if idx >= count {
                            None
                        } else {
                            let offset = u32::from_le(offsets[idx]) as usize;
                            Some(self.as_slice()[offset])
                        }
                    }
                )
            } else {
                quote!(
                    pub fn nth(&self, idx: usize) -> Option<#inner<'_>> {
                        let (count, offsets) = Self::item_offsets(self);
                        if idx >= count {
                            None
                        } else if idx == count - 1 {
                            let start = u32::from_le(offsets[idx]) as usize;
                            Some(#inner::new_unchecked(&self.as_slice()[start..]))
                        } else {
                            let start = u32::from_le(offsets[idx]) as usize;
                            let end = u32::from_le(offsets[idx+1]) as usize;
                            Some(#inner::new_unchecked(&self.as_slice()[start..end]))
                        }
                    }
                )
            };
            funcs.push(code);
        }
        impl_reader(writer, origin_name, funcs)?;
    }
    {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        let inner = entity_name(&info.typ.name);
        let code = quote!(
            pub fn push(mut self, v: #inner) -> Self {
                self.0.push(v);
                self
            }
        );
        funcs.push(code);
        impl_builder(writer, origin_name, funcs)?;
    }
    {
        let code = quote!(
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
        );
        impl_trait_builder(writer, origin_name, code)?;
    }
    writeln!(writer)
}

fn gen_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    def_molecule(writer, origin_name)?;
    def_builder_for_struct_or_table(writer, origin_name, &info.inner[..])?;
    {
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
        let code = quote!(
            fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        #reader_string.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
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
                if !offsets.windows(2).all(|i| i[0] < i[1]) {
                    let err = VerificationError::OffsetsNotMatch(#reader_string.to_owned());
                    Err(err)?;
                }
                #( #verify_fields )*
                Ok(())
            }
        );
        impl_trait_reader(writer, origin_name, code)?;
    }
    {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let field_count = usize_lit(info.inner.len());
            let code = quote!(pub const FIELD_COUNT: usize = #field_count;);
            funcs.push(code);
        }
        {
            let code = quote!(
                pub fn field_offsets(&self) -> (usize, &[u32]) {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.as_slice()) };
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
        impl_reader(writer, origin_name, funcs)?;
    }
    {
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
        impl_builder(writer, origin_name, funcs)?;
    }
    {
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
        let code = quote!(
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
        );
        impl_trait_builder(writer, origin_name, code)?;
    }
    writeln!(writer)
}
