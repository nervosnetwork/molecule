use std::io;

use proc_macro2 as m4;
use quote::quote;

use crate::ast::verified as ast;

use super::builder::*;
use super::common::*;
use super::getter::*;
use super::utilities::*;

pub(super) fn gen_option<W>(
    writer: &mut W,
    origin_name: &str,
    info: &ast::Option_,
) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        quote!({
            if let Some(v) = self.to_opt() {
                write!(f, "{}(Some({}))", Self::NAME, v)
            } else {
                write!(f, "{}(None)", Self::NAME)
            }
        })
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_builder_for_option(writer, origin_name, info)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder().set(self.to_opt())
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = def_getter_for_option(true, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let inner = reader_name(&info.typ.name);
        if info.typ.is_atom() {
            quote!(
                fn verify(
                    slice: &[u8],
                    _compatible: bool,
                ) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    if slice.len() > 1 {
                        let err = VerificationError::TotalSizeNotAsExpected(
                            Self::NAME.to_owned(),
                            0,
                            1,
                            slice.len(),
                        );
                        Err(err)?;
                    }
                    Ok(())
                }
            )
        } else {
            quote!(
                fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                    if !slice.is_empty() {
                        #inner::verify(&slice[..], compatible)?;
                    }
                    Ok(())
                }
            )
        }
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = def_getter_for_option(false, info);
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
            pub fn set(mut self, v: Option<#inner>) -> Self {
                self.0 = v;
                self
            }
        );
        funcs.push(code);
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    writeln!(writer)
}

pub(super) fn gen_union<W>(writer: &mut W, origin_name: &str, info: &ast::Union) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        quote!(
            write!(f, "{}(", Self::NAME)?;
            self.to_enum().display_inner(f)?;
            write!(f, ")")
        )
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_items_for_union(writer, origin_name, info)?;
    def_builder_for_union(writer, origin_name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder().set(self.to_enum())
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = def_getter_for_union(true, origin_name, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let item_count = usize_lit(info.inner.len());
        let verify_inners = info.inner.iter().enumerate().map(|(index, inner)| {
            let item_id = usize_lit(index + 1);
            let inner = reader_name(&inner.typ.name);
            quote!(#item_id => #inner::verify(&slice[molecule::ITEM_ID_SIZE..], _compatible),)
        });
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() < molecule::ITEM_ID_SIZE {
                    let err = VerificationError::HeaderIsBroken(
                        Self::NAME.to_owned(), molecule::ITEM_ID_SIZE, slice.len());
                    Err(err)?;
                }
                let item_id = molecule::extract_item_id(slice);
                match item_id {
                    #( #verify_inners )*
                    _ => {
                        let err = VerificationError::UnknownItem(
                            Self::NAME.to_owned(), #item_count, item_id);
                        Err(err)
                    },
                }?;
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = def_getter_for_union(false, origin_name, info);
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        quote!(
            fn expected_length(&self) -> usize {
                molecule::ITEM_ID_SIZE + self.0.as_slice().len()
            }
            fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                let item_id = self.0.item_id().to_le_bytes();
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

pub(super) fn gen_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        if info.typ.is_atom() {
            quote!(write!(
                f,
                "{}(0x{})",
                Self::NAME,
                hex_string(&self.raw_data()).unwrap()
            ))
        } else {
            let display_items = (0..info.item_count).map(|idx| {
                let func = snake_name(&format!("nth{}", idx));
                if idx == 0 {
                    quote!(write!(f, "{}", self.#func())?;)
                } else {
                    quote!(write!(f, ", {}", self.#func())?;)
                }
            });
            quote!(
                write!(f, "{} [", Self::NAME)?;
                #( #display_items )*
                write!(f, "]")
            )
        }
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_builder_for_array(writer, origin_name, info)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let items = (0..info.item_count)
                .map(|idx| snake_name(&format!("nth{}", idx)))
                .map(|func| quote!(self.#func()));
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder().set([ #( #items, )* ])
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = def_getter_for_array(true, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let inner = reader_name(&info.typ.name);
        let total_size = usize_lit(info.item_size * info.item_count);
        let verify_inners = if info.typ.is_atom() {
            Vec::new()
        } else {
            (0..info.item_count)
                .map(|i| {
                    let start = usize_lit(info.item_size * i);
                    let end = usize_lit(info.item_size * (i + 1));
                    quote!(#inner::verify(&slice[#start..#end], _compatible)?;)
                })
                .collect()
        };
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() != #total_size {
                    let err = VerificationError::TotalSizeNotMatch(
                        Self::NAME.to_owned(), #total_size, slice.len());
                    Err(err)?;
                }
                #( #verify_inners )*
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = def_getter_for_array(false, info);
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
        let item_count = usize_lit(info.item_count);
        {
            let code = quote!(
                pub fn set(mut self, v: [#inner; #item_count]) -> Self {
                    self.0 = v;
                    self
                }
            );
            funcs.push(code);
        }
        for idx in 0..info.item_count {
            let index = usize_lit(idx);
            let func = snake_name(&format!("nth{}", idx));
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

pub(super) fn gen_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        let display_fields = info.inner.iter().enumerate().map(|(i, f)| {
            let field = f.name.clone();
            let func = snake_name(&f.name);
            if i == 0 {
                quote!(write!(f, "{}: {}", #field, self.#func())?;)
            } else {
                quote!(write!(f, ", {}: {}", #field, self.#func())?;)
            }
        });
        quote!(
            write!(f, "{} {{ ", Self::NAME)?;
            #( #display_fields )*
            write!(f, " }}")
        )
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_builder_for_struct_or_table(writer, origin_name, &info.inner[..])?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let fields = info.inner.iter().map(|f| snake_name(&f.name));
            let fields_func = fields.clone();
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder()
                        #( .#fields(self.#fields_func()) )*
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = def_getter_for_struct(true, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
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
                        #field::verify(&slice[#start..#end], _compatible)?;
                    );
                    codes.push(code);
                }
            }
            codes
        };
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                if slice.len() != #total_size {
                    let err = VerificationError::TotalSizeNotMatch(
                        Self::NAME.to_owned(), #total_size, slice.len());
                    Err(err)?;
                }
                #( #verify_fields )*
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = def_getter_for_struct(false, info);
    impl_reader(writer, origin_name, funcs)?;
    let code = {
        let total_size = usize_lit(info.field_size.iter().sum());
        let fields = info.inner.iter().map(|f| {
            let field_name = snake_name(&f.name);
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
            let field_name = snake_name(&f.name);
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

pub(super) fn gen_fix_vec<W>(
    writer: &mut W,
    origin_name: &str,
    info: &ast::FixedVector,
) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        if info.typ.is_atom() {
            quote!(write!(
                f,
                "{}(0x{})",
                Self::NAME,
                hex_string(&self.raw_data()).unwrap()
            ))
        } else {
            quote!(
                write!(f, "{} [", Self::NAME)?;
                for i in 0..self.len() {
                    if i == 0 {
                        write!(f, "{}", self.get_unchecked(i))?;
                    } else {
                        write!(f, ", {}", self.get_unchecked(i))?;
                    }
                }
                write!(f, "]")
            )
        }
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_builder_for_vector(writer, origin_name, &info.typ.name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder().extend(self.into_iter())
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = def_getter_for_fix_vec(true, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let inner = reader_name(&info.typ.name);
        let item_size = usize_lit(info.item_size);
        let verify_inners = if info.typ.is_atom() {
            quote!()
        } else {
            quote!(
                for i in 0..item_count {
                    let start = #item_size * i;
                    let end = start + #item_size;
                    #inner::verify(&slice[start..end], _compatible)?;
                }
            )
        };
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err =
                        VerificationError::HeaderIsBroken(Self::NAME.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let item_count = u32::from_le(ptr[0]) as usize;
                let expected = 4 + #item_size * item_count;
                if len != expected {
                    let err = VerificationError::TotalSizeNotMatch(
                        Self::NAME.to_owned(),
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
    let funcs = def_getter_for_fix_vec(false, info);
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
            pub fn set(mut self, v: Vec<#inner>) -> Self {
                self.0 = v;
                self
            }
            pub fn push(mut self, v: #inner) -> Self {
                self.0.push(v);
                self
            }
            pub fn extend<T: ::std::iter::IntoIterator<Item=#inner>>(mut self, iter: T) -> Self {
                for elem in iter {
                    self.0.push(elem);
                }
                self
            }
        );
        funcs.push(code);
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    def_iterator_for_vector(writer, origin_name, &info.typ.name, info.typ.is_atom())?;
    writeln!(writer)
}

pub(super) fn gen_dyn_vec<W>(
    writer: &mut W,
    origin_name: &str,
    info: &ast::DynamicVector,
) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        quote!(
            write!(f, "{} [", Self::NAME)?;
            for i in 0..self.len() {
                if i == 0 {
                    write!(f, "{}", self.get_unchecked(i))?;
                } else {
                    write!(f, ", {}", self.get_unchecked(i))?;
                }
            }
            write!(f, "]")
        )
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_builder_for_vector(writer, origin_name, &info.typ.name)?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder().extend(self.into_iter())
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = def_getter_for_dyn_vec(true, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let inner = reader_name(&info.typ.name);
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        Self::NAME.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let total_size = u32::from_le(ptr[0]) as usize;
                if total_size != len {
                    let err = VerificationError::TotalSizeNotMatch(
                        Self::NAME.to_owned(), total_size, len);
                    Err(err)?;
                }
                if total_size == 4 {
                    return Ok(());
                }
                if total_size < 4 + 4 {
                    let err = VerificationError::DataIsShort(
                        Self::NAME.to_owned(), 8, total_size);
                    Err(err)?;
                }
                let offset_first = u32::from_le(ptr[1]) as usize;
                if offset_first % 4 != 0 {
                    let err = VerificationError::FirstOffsetIsBroken(
                        Self::NAME.to_owned(), offset_first);
                    Err(err)?;
                }
                if offset_first < 4 + 4 {
                    let err = VerificationError::FirstOffsetIsShort(
                        Self::NAME.to_owned(), 8, offset_first);
                    Err(err)?;
                }
                let item_count = offset_first / 4 - 1;
                let expected = 4 + 4 * item_count;
                if total_size < expected {
                    let err = VerificationError::DataIsShort(
                        Self::NAME.to_owned(), expected, total_size);
                    Err(err)?;
                }
                let mut offsets: Vec<usize> = ptr[1..(item_count+1)]
                    .iter()
                    .map(|x| u32::from_le(*x) as usize)
                    .collect();
                offsets.push(total_size);
                if offsets.windows(2).any(|i| i[0] > i[1]) {
                    let err = VerificationError::OffsetsNotMatch(Self::NAME.to_owned());
                    Err(err)?;
                }
                for i in 0..=(offsets.len()-2) {
                    let start = offsets[i];
                    let end = offsets[i+1];
                    #inner::verify(&slice[start..end], _compatible)?;
                }
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = def_getter_for_dyn_vec(false, info);
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
            pub fn set(mut self, v: Vec<#inner>) -> Self {
                self.0 = v;
                self
            }
            pub fn push(mut self, v: #inner) -> Self {
                self.0.push(v);
                self
            }
            pub fn extend<T: ::std::iter::IntoIterator<Item=#inner>>(mut self, iter: T) -> Self {
                for elem in iter {
                    self.0.push(elem);
                }
                self
            }
        );
        funcs.push(code);
        funcs
    };
    impl_builder(writer, origin_name, funcs)?;
    def_iterator_for_vector(writer, origin_name, &info.typ.name, info.typ.is_atom())?;
    writeln!(writer)
}

pub(super) fn gen_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    def_entity_and_reader(writer, origin_name)?;
    let stmts = {
        let display_fields = info.inner.iter().enumerate().map(|(i, f)| {
            let field = f.name.clone();
            let func = snake_name(&f.name);
            if i == 0 {
                quote!(write!(f, "{}: {}", #field, self.#func())?;)
            } else {
                quote!(write!(f, ", {}: {}", #field, self.#func())?;)
            }
        });
        let field_count = usize_lit(info.inner.len());
        let display_unresolved = if info.inner.is_empty() {
            quote!(write!(f, "..")?;)
        } else {
            quote!(write!(f, ", ..")?;)
        };
        quote!(
            write!(f, "{} {{ ", Self::NAME)?;
            #( #display_fields )*
            let (_, count, _) = Self::field_offsets(&self);
            if count != #field_count {
                #display_unresolved
            }
            write!(f, " }}")
        )
    };
    impl_display_for_entity_and_reader(writer, origin_name, stmts)?;
    def_builder_for_struct_or_table(writer, origin_name, &info.inner[..])?;
    impl_default_for_entity(writer, origin_name, info.default_content())?;
    let funcs = {
        let mut funcs: Vec<m4::TokenStream> = Vec::new();
        {
            let fields = info.inner.iter().map(|f| snake_name(&f.name));
            let fields_func = fields.clone();
            let code = quote!(
                fn as_builder(self) -> Self::Builder {
                    Self::new_builder()
                        #( .#fields(self.#fields_func()) )*
                }
            );
            funcs.push(code);
        }
        funcs
    };
    impl_trait_entity(writer, origin_name, funcs)?;
    let funcs = def_getter_for_table(true, info);
    impl_entity(writer, origin_name, funcs)?;
    let code = {
        let field_count = usize_lit(info.inner.len());
        let verify_fields = info.inner.iter().enumerate().map(|(i, f)| {
            let field = reader_name(&f.typ.name);
            let start = usize_lit(i);
            let end = usize_lit(i + 1);
            if f.typ.is_atom() {
                quote!(
                    if offsets[#start] + 1 != offsets[#end] {
                        let err = VerificationError::FieldIsBroken(
                            Self::NAME.to_owned(), #start);
                        Err(err)?;
                    }
                )
            } else {
                quote!(
                    #field::verify(&slice[offsets[#start]..offsets[#end]], compatible)?;
                )
            }
        });
        quote!(
            fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::error::VerificationError;
                let len = slice.len();
                if len < 4 {
                    let err = VerificationError::HeaderIsBroken(
                        Self::NAME.to_owned(), 4, len);
                    Err(err)?;
                }
                let ptr: &[u32] = unsafe { ::std::mem::transmute(slice) };
                let total_size = u32::from_le(ptr[0]) as usize;
                if total_size != len {
                    let err = VerificationError::TotalSizeNotMatch(
                        Self::NAME.to_owned(), total_size, len);
                    Err(err)?;
                }
                if #field_count == 0 && total_size == 4 {
                    return Ok(());
                }
                let expected = 4 + 4 * #field_count;
                if total_size < expected {
                    let err = VerificationError::HeaderIsBroken(
                        Self::NAME.to_owned(), expected, total_size);
                    Err(err)?;
                }
                let offset_first = u32::from_le(ptr[1]) as usize;
                if offset_first % 4 != 0 {
                    let err = VerificationError::FirstOffsetIsBroken(
                        Self::NAME.to_owned(), offset_first);
                    Err(err)?;
                }
                if offset_first < expected {
                    let err = VerificationError::FirstOffsetIsShort(
                        Self::NAME.to_owned(), expected, offset_first);
                    Err(err)?;
                }
                let real_field_count = if compatible {
                    let real_field_count = offset_first / 4 - 1;
                    let real_expected = 4 + 4 * real_field_count;
                    if total_size < real_expected {
                        let err = VerificationError::DataIsShort(
                            Self::NAME.to_owned(), real_expected, total_size);
                        Err(err)?;
                    }
                    real_field_count
                } else {
                    if offset_first > expected {
                        let err = VerificationError::FirstOffsetIsOverflow(
                            Self::NAME.to_owned(), expected, offset_first);
                        Err(err)?;
                    }
                    #field_count
                };
                let mut offsets: Vec<usize> = ptr[1..=real_field_count]
                    .iter()
                    .map(|x| u32::from_le(*x) as usize)
                    .collect();
                offsets.push(total_size);
                if offsets.windows(2).any(|i| i[0] > i[1]) {
                    let err = VerificationError::OffsetsNotMatch(Self::NAME.to_owned());
                    Err(err)?;
                }
                #( #verify_fields )*
                Ok(())
            }
        )
    };
    impl_trait_reader(writer, origin_name, code)?;
    let funcs = def_getter_for_table(false, info);
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
            let field_name = snake_name(&f.name);
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
            let field_name = snake_name(&f.name);
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
