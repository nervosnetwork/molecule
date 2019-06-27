use std::{io, iter::FromIterator as _};

use case::CaseExt;
use proc_macro2 as m4;
use quote::quote;

use super::Generator;
use crate::ast::verified as ast;

impl Generator {
    pub fn generate_rust<W>(&self, writer: &mut W) -> io::Result<()>
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

fn ident_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    if name == ast::ATOM_NAME {
        m4::Ident::new("u8", span)
    } else {
        m4::Ident::new(&name.to_camel(), span)
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

fn create_defuns<W>(
    writer: &mut W,
    name: &m4::Ident,
    defuns: Vec<m4::TokenStream>,
) -> io::Result<()>
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

fn read_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    let name = ident_name(origin_name);
    create_reader(writer, &name)?;
    let inner = ident_name(&info.typ.name);
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
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
            quote!(
                pub fn #func(&self) -> #inner {
                    #inner(&self.0[#start..])
                }
            )
        };
        defuns.push(code);
    }
    create_defuns(writer, &name, defuns)?;
    writeln!(writer)
}

fn read_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    let name = ident_name(origin_name);
    create_reader(writer, &name)?;
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    let mut offset = 0;
    for (field, size) in info.inner.iter().zip(info.field_size.iter()) {
        let field_name = func_name(&field.name);
        let field_type = ident_name(&field.typ.name);
        let idx = usize_lit(offset);
        offset += size;
        let code = if field.typ.is_atom() {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    self.0[#idx]
                }
            )
        } else {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    #field_type(&self.0[#idx..])
                }
            )
        };
        defuns.push(code);
    }
    create_defuns(writer, &name, defuns)?;
    writeln!(writer)
}

fn read_fix_vec<W>(writer: &mut W, origin_name: &str, info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = ident_name(origin_name);
    create_reader(writer, &name)?;
    let inner = ident_name(&info.typ.name);
    let item_size = usize_lit(info.item_size);
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn nth(&self, idx: usize) -> Option<#inner> {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let item_count = u32::from_le(ptr[0]) as usize;
                    if idx >= item_count {
                        None
                    } else {
                        Some(self.0[idx])
                    }
                }
            )
        } else {
            quote!(
                pub fn nth(&self, idx: usize) -> Option<#inner> {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let item_count = u32::from_le(ptr[0]) as usize;
                    if idx >= item_count {
                        None
                    } else {
                        Some(#inner(&self.0[(4+idx*#item_size)..]))
                    }
                }
            )
        };
        defuns.push(code);
    }
    create_defuns(writer, &name, defuns)?;
    writeln!(writer)
}

fn read_dyn_vec<W>(writer: &mut W, origin_name: &str, info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = ident_name(origin_name);
    create_reader(writer, &name)?;
    let inner = ident_name(&info.typ.name);
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
                    } else {
                        let offset = u32::from_le(ptr[idx+2]) as usize;
                        Some(#inner(&self.0[offset..]))
                    }
                }
            )
        };
        defuns.push(code);
    }
    create_defuns(writer, &name, defuns)?;
    writeln!(writer)
}

fn read_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    let name = ident_name(origin_name);
    create_reader(writer, &name)?;
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    for (i, field) in info.inner.iter().enumerate() {
        let field_name = func_name(&field.name);
        let field_type = ident_name(&field.typ.name);
        let idx = usize_lit(i + 1);
        let code = if field.typ.is_atom() {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let offset = u32::from_le(ptr[#idx]) as usize;
                    self.0[offset]
                }
            )
        } else {
            quote!(
                pub fn #field_name(&self) -> #field_type {
                    let ptr: &[u32] = unsafe { std::mem::transmute(self.0) };
                    let offset = u32::from_le(ptr[#idx]) as usize;
                    #field_type(&self.0[offset..])
                }
            )
        };
        defuns.push(code);
    }
    create_defuns(writer, &name, defuns)?;
    writeln!(writer)
}
