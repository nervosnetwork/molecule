use std::{io, iter::FromIterator as _};

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
        for decl in &self.ast.decls[..] {
            match decl.typ {
                ast::TopDeclType::Array(ref info) => read_array(writer, &decl.name, info),
                ast::TopDeclType::Struct(ref info) => read_struct(writer, &decl.name, info),
                ast::TopDeclType::FixedVector(ref info) => read_fix_vec(writer, &decl.name, info),
                ast::TopDeclType::DynamicVector(ref info) => read_dyn_vec(writer, &decl.name, info),
                ast::TopDeclType::Table(ref info) => read_table(writer, &decl.name, info),
                ast::TopDeclType::Atom => unreachable!(),
            }?;
        }
        Ok(())
    }
}

fn reader_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    if name == ast::ATOM_NAME {
        m4::Ident::new(ATOM_NAME, span)
    } else {
        let reader_name = format!("{}Reader", name).to_camel();
        m4::Ident::new(&reader_name, span)
    }
}

fn func_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    m4::Ident::new(&name.to_snake(), span)
}

fn usize_lit(num: usize) -> m4::Literal {
    m4::Literal::usize_unsuffixed(num)
}

fn create_reader<W>(writer: &mut W, name: &m4::Ident) -> io::Result<()>
where
    W: io::Write,
{
    let code = quote!(
        #[derive(Debug)]
        pub struct #name<'m> (&'m [u8]);
    );
    write!(writer, "{}", code)
}

fn def_funcs<W>(writer: &mut W, name: &m4::Ident, defuns: Vec<m4::TokenStream>) -> io::Result<()>
where
    W: io::Write,
{
    let defuns = m4::TokenStream::from_iter(defuns);
    let code = quote!(
        impl<'m> #name<'m> {
            #defuns
        }
    );
    write!(writer, "{}", code)
}

fn impl_traits<W>(writer: &mut W, name: &m4::Ident) -> io::Result<()>
where
    W: io::Write,
{
    let code = quote!(
        impl<'m> molecule::prelude::Reader<'m> for #name<'m> {
            fn as_slice(&self) -> &[u8] {
                &self.0[..]
            }
            fn from_slice<'a: 'm>(slice: &'a [u8]) -> Option<Self> {
                use molecule::prelude::Verifiable;
                if Self::verify(slice) {
                    Some(#name(slice))
                } else {
                    None
                }
            }
        }
    );
    write!(writer, "{}", code)
}

fn read_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    let name = reader_name(origin_name);
    create_reader(writer, &name)?;
    let inner = reader_name(&info.typ.name);
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let total_size = usize_lit(info.item_size * info.item_count);
        let item_size = usize_lit(info.item_size);
        let item_count = usize_lit(info.item_count);
        let code = quote!(
            pub const TOTAL_SIZE: usize = #total_size;
            pub const ITEM_SIZE: usize = #item_size;
            pub const ITEM_COUNT: usize = #item_count;
        );
        defuns.push(code);
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
                    #inner(&self.0[#start..#end])
                }
            )
        };
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> bool {
                    slice.len() == Self::TOTAL_SIZE
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_traits(writer, &name)?;
    writeln!(writer)
}

fn read_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    let name = reader_name(origin_name);
    create_reader(writer, &name)?;
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let total_size = usize_lit(info.field_size.iter().sum());
        let field_count = usize_lit(info.field_size.len());
        let code = quote!(
            pub const TOTAL_SIZE: usize = #total_size;
            pub const FIELD_COUNT: usize = #field_count;
        );
        defuns.push(code);
    }
    let mut offset = 0;
    for (field, size) in info.inner.iter().zip(info.field_size.iter()) {
        let field_name = func_name(&field.name);
        let field_type = reader_name(&field.typ.name);
        let start = usize_lit(offset);
        offset += size;
        let code = if field.typ.is_atom() {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    self.0[#start]
                }
            )
        } else {
            let end = usize_lit(offset);
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    #field_type(&self.0[#start..#end])
                }
            )
        };
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> bool {
                    slice.len() == Self::TOTAL_SIZE
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_traits(writer, &name)?;
    writeln!(writer)
}

fn read_fix_vec<W>(writer: &mut W, origin_name: &str, info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = reader_name(origin_name);
    create_reader(writer, &name)?;
    let inner = reader_name(&info.typ.name);
    let item_size = usize_lit(info.item_size);
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let code = quote!(pub const ITEM_SIZE: usize = #item_size;);
        defuns.push(code);
    }
    {
        let code = quote!(
            pub fn item_count(&self) -> usize {
                let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                u32::from_le(ptr[0]) as usize
            }
        );
        defuns.push(code);
    }
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn nth(&self, idx: usize) -> Option<#inner> {
                    if idx >= self.item_count() {
                        None
                    } else {
                        Some(self.0[4+idx])
                    }
                }
            )
        } else {
            quote!(
                pub fn nth(&self, idx: usize) -> Option<#inner> {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let item_count = u32::from_le(ptr[0]) as usize;
                    if idx >= self.item_count() {
                        None
                    } else {
                        let start = 4 + idx * #item_size;
                        let end = start + #item_size;
                        Some(#inner(&self.0[start..end]))
                    }
                }
            )
        };
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> bool {
                    let len = slice.len();
                    if len >= 4 {
                        let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                        let item_count = u32::from_le(ptr[0]) as usize;
                        len == 4 + Self::ITEM_SIZE * item_count
                    } else {
                        false
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_traits(writer, &name)?;
    writeln!(writer)
}

fn read_dyn_vec<W>(writer: &mut W, origin_name: &str, info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = reader_name(origin_name);
    create_reader(writer, &name)?;
    let inner = reader_name(&info.typ.name);
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn nth(&self, idx: usize) -> Option<#inner> {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let item_count = u32::from_le(ptr[1]) as usize;
                    if idx >= item_count {
                        None
                    } else {
                        let offset = u32::from_le(ptr[idx+2]) as usize;
                        Some(self.0[offset])
                    }
                }
            )
        } else {
            quote!(
                pub fn nth(&self, idx: usize) -> Option<#inner> {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let item_count = u32::from_le(ptr[1]) as usize;
                    if idx >= item_count {
                        None
                    } else if idx == item_count - 1 {
                        let start = u32::from_le(ptr[idx+2]) as usize;
                        let end = u32::from_le(ptr[0]) as usize;
                        Some(#inner(&self.0[start..end]))
                    } else {
                        let start = u32::from_le(ptr[idx+2]) as usize;
                        let end = u32::from_le(ptr[idx+3]) as usize;
                        Some(#inner(&self.0[start..end]))
                    }
                }
            )
        };
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> bool {
                    let len = slice.len();
                    if len < 4 {
                        return false;
                    }
                    let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                    let total_size = u32::from_le(ptr[0]) as usize;
                    if total_size != slice.len() {
                        return false;
                    }
                    if total_size == 4 {
                        return true;
                    }
                    if total_size < 4 + 4 {
                        return false;
                    }
                    let offset_first = u32::from_le(ptr[1]) as usize;
                    if offset_first % 4 != 0 {
                        return false;
                    }
                    if offset_first < 4 + 4 {
                        return false;
                    }
                    let item_count = offset_first / 4 - 1;
                    if total_size < 4 + 4 * item_count {
                        return false;
                    }
                    let mut offsets: Vec<usize> = ptr[1..(item_count+1)]
                        .iter()
                        .map(|x| u32::from_le(*x) as usize)
                        .collect();
                    offsets.push(total_size);
                    if offsets.windows(2).any(|i| i[0] + 4 > i[1]) {
                        return false;
                    }
                    if offsets.windows(2).any(|i| !#inner::verify(&slice[i[0]..i[1]])) {
                        return false;
                    }
                    true
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_traits(writer, &name)?;
    writeln!(writer)
}

fn read_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    let name = reader_name(origin_name);
    create_reader(writer, &name)?;
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let item_count = usize_lit(info.inner.len());
        let code = quote!(pub const FIELD_COUNT: usize = #item_count;);
        defuns.push(code);
    }
    for (i, field) in info.inner.iter().enumerate() {
        let field_name = func_name(&field.name);
        let field_type = reader_name(&field.typ.name);
        let start = usize_lit(i + 1);
        let end = usize_lit(i + 2);
        let code = if field.typ.is_atom() {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let offset = u32::from_le(ptr[#start]) as usize;
                    self.0[offset]
                }
            )
        } else if i == info.inner.len() - 1 {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let start = u32::from_le(ptr[#start]) as usize;
                    let end = u32::from_le(ptr[0]) as usize;
                    #field_type(&self.0[start..end])
                }
            )
        } else {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let start = u32::from_le(ptr[#start]) as usize;
                    let end = u32::from_le(ptr[#end]) as usize;
                    #field_type(&self.0[start..end])
                }
            )
        };
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    {
        let mut verify_fields: Vec<m4::TokenStream> = Vec::new();
        for (i, field) in info.inner.iter().enumerate() {
            let field_type = reader_name(&field.typ.name);
            let start = usize_lit(i);
            let end = usize_lit(i + 1);
            let code = if field.typ.is_atom() {
                quote!(
                    if offsets[#start] + 1 != offsets[#end] {
                        return false;
                    }
                )
            } else {
                quote!(
                    if !#field_type::verify(&slice[offsets[#start]..offsets[#end]]) {
                        return false;
                    }
                )
            };
            verify_fields.push(code);
        }
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> bool {
                    let len = slice.len();
                    if len < 4 {
                        return false;
                    }
                    let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                    let total_size = u32::from_le(ptr[0]) as usize;
                    if total_size != slice.len() {
                        return false;
                    }
                    if total_size < 4 + 4 * Self::FIELD_COUNT {
                        return false;
                    }
                    let mut offsets: Vec<usize> = ptr[1..(Self::FIELD_COUNT+1)]
                        .iter()
                        .map(|x| u32::from_le(*x) as usize)
                        .collect();
                    if offsets[0] != 4 + 4 * Self::FIELD_COUNT {
                        return false;
                    }
                    offsets.push(total_size);
                    if offsets.windows(2).any(|i| i[0] >= i[1]) {
                        return false;
                    }
                    #( #verify_fields )*
                    true
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_traits(writer, &name)?;
    writeln!(writer)
}
