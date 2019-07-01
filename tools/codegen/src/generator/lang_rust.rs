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
        for decl in &self.ast.decls[..] {
            match decl.typ {
                ast::TopDeclType::Array(ref info) => {
                    read_array(writer, &decl.name, info)?;
                    build_array(writer, &decl.name, info)?;
                    owned_array(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Struct(ref info) => {
                    read_struct(writer, &decl.name, info)?;
                    build_struct(writer, &decl.name, info)?;
                    owned_struct(writer, &decl.name, info)?;
                }
                ast::TopDeclType::FixedVector(ref info) => {
                    read_fix_vec(writer, &decl.name, info)?;
                    build_fix_vec(writer, &decl.name, info)?;
                    owned_fix_vec(writer, &decl.name, info)?;
                }
                ast::TopDeclType::DynamicVector(ref info) => {
                    read_dyn_vec(writer, &decl.name, info)?;
                    build_dyn_vec(writer, &decl.name, info)?;
                    owned_dyn_vec(writer, &decl.name, info)?;
                }
                ast::TopDeclType::Table(ref info) => {
                    read_table(writer, &decl.name, info)?;
                    build_table(writer, &decl.name, info)?;
                    owned_table(writer, &decl.name, info)?;
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

fn func_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    m4::Ident::new(&name.to_snake(), span)
}

fn usize_lit(num: usize) -> m4::Literal {
    m4::Literal::usize_unsuffixed(num)
}

fn def_funcs_with_lifetime<W>(
    writer: &mut W,
    name: &m4::Ident,
    defuns: Vec<m4::TokenStream>,
) -> io::Result<()>
where
    W: io::Write,
{
    let code = quote!(
        impl<'m> #name<'m> {
            #( #defuns )*
        }
    );
    write!(writer, "{}", code)
}

fn def_funcs<W>(writer: &mut W, name: &m4::Ident, defuns: Vec<m4::TokenStream>) -> io::Result<()>
where
    W: io::Write,
{
    let code = quote!(
        impl #name {
            #( #defuns )*
        }
    );
    write!(writer, "{}", code)
}

/*
 * Reader
 */

fn reader_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    if name == ast::ATOM_NAME {
        m4::Ident::new(ATOM_NAME, span)
    } else {
        m4::Ident::new(&format!("{}Reader", name).to_camel(), span)
    }
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

fn impl_reader_traits<W>(writer: &mut W, name: &m4::Ident) -> io::Result<()>
where
    W: io::Write,
{
    let code = quote!(
        impl<'m> molecule::prelude::Reader<'m> for #name<'m> {
            fn as_slice(&self) -> &[u8] {
                &self.0[..]
            }
            fn from_slice<'a: 'm>(slice: &'a [u8]) -> molecule::error::VerificationResult<Self> {
                use molecule::prelude::Verifiable;
                Self::verify(slice).map(|_| #name(slice))
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
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    if slice.len() == Self::TOTAL_SIZE {
                        Ok(())
                    } else {
                        let err = VerificationError::TotalSizeNotMatch(
                            stringify!(#name).to_owned(), Self::TOTAL_SIZE, slice.len());
                        Err(err)
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_reader_traits(writer, &name)?;
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
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    if slice.len() == Self::TOTAL_SIZE {
                        Ok(())
                    } else {
                        let err = VerificationError::TotalSizeNotMatch(
                            stringify!(#name).to_owned(), Self::TOTAL_SIZE, slice.len());
                        Err(err)
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_reader_traits(writer, &name)?;
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
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    let len = slice.len();
                    if len < 4 {
                        let err = VerificationError::HeaderIsBroken(
                            stringify!(#name).to_owned(), 4, len);
                        Err(err)
                    } else {
                        let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                        let item_count = u32::from_le(ptr[0]) as usize;
                        let expected = 4 + Self::ITEM_SIZE * item_count;
                        if len == expected {
                            let err = VerificationError::TotalSizeNotMatch(
                                stringify!(#name).to_owned(), expected, len);
                            Err(err)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_reader_traits(writer, &name)?;
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
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    let len = slice.len();
                    if len < 4 {
                        let err = VerificationError::HeaderIsBroken(
                            stringify!(#name).to_owned(), 4, len);
                        Err(err)?;
                    }
                    let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                    let total_size = u32::from_le(ptr[0]) as usize;
                    if total_size != len {
                        let err = VerificationError::TotalSizeNotMatch(
                            stringify!(#name).to_owned(), total_size, len);
                        Err(err)?;
                    }
                    if total_size == 4 {
                        return Ok(());
                    }
                    if total_size < 4 + 4 {
                        let err = VerificationError::DataIsShort(
                            stringify!(#name).to_owned(), 8, total_size);
                        Err(err)?;
                    }
                    let offset_first = u32::from_le(ptr[1]) as usize;
                    if offset_first % 4 != 0 {
                        let err = VerificationError::FirstOffsetIsBroken(
                            stringify!(#name).to_owned(), offset_first);
                        Err(err)?;
                    }
                    if offset_first < 4 + 4 {
                        let err = VerificationError::FirstOffsetIsShort(
                            stringify!(#name).to_owned(), 8, offset_first);
                        Err(err)?;
                    }
                    let item_count = offset_first / 4 - 1;
                    let expected = 4 + 4 * item_count;
                    if total_size < expected {
                        let err = VerificationError::DataIsShort(
                            stringify!(#name).to_owned(), expected, total_size);
                        Err(err)?;
                    }
                    let mut offsets: Vec<usize> = ptr[1..(item_count+1)]
                        .iter()
                        .map(|x| u32::from_le(*x) as usize)
                        .collect();
                    offsets.push(total_size);
                    if offsets.windows(2).any(|i| i[0] + 4 > i[1]) {
                        let err = VerificationError::OffsetsNotMatch(stringify!(#name).to_owned());
                        Err(err)?;
                    }
                    for i in 0..=(offsets.len()-2) {
                        #inner::verify(&slice[offsets[i]..offsets[i+1]])?;
                    }
                    Ok(())
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_reader_traits(writer, &name)?;
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
        let field_count = usize_lit(info.inner.len());
        let code = quote!(pub const FIELD_COUNT: usize = #field_count;);
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
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let mut verify_fields: Vec<m4::TokenStream> = Vec::new();
        for (i, field) in info.inner.iter().enumerate() {
            let field_type = reader_name(&field.typ.name);
            let start = usize_lit(i);
            let end = usize_lit(i + 1);
            let code = if field.typ.is_atom() {
                quote!(
                    if offsets[#start] + 1 != offsets[#end] {
                        let err = VerificationError::FieldIsBroken(
                            stringify!(#name).to_owned(), #start);
                        Err(err)?;
                    }
                )
            } else {
                quote!(
                    #field_type::verify(&slice[offsets[#start]..offsets[#end]])?;
                )
            };
            verify_fields.push(code);
        }
        let code = quote!(
            impl<'m> molecule::prelude::Verifiable for #name<'m> {
                fn verify(slice: &[u8]) -> molecule::error::VerificationResult<()> {
                    use molecule::error::VerificationError;
                    let len = slice.len();
                    if len < 4 {
                        let err = VerificationError::HeaderIsBroken(
                            stringify!(#name).to_owned(), 4, len);
                        Err(err)?;
                    }
                    let ptr: &[u32] = unsafe { std::mem::transmute(slice) };
                    let total_size = u32::from_le(ptr[0]) as usize;
                    if total_size != len {
                        let err = VerificationError::TotalSizeNotMatch(
                            stringify!(#name).to_owned(), total_size, len);
                        Err(err)?;
                    }
                    let expected = 4 + 4 * Self::FIELD_COUNT;
                    if total_size < expected {
                        let err = VerificationError::HeaderIsBroken(
                            stringify!(#name).to_owned(), expected, total_size);
                        Err(err)?;
                    }
                    let mut offsets: Vec<usize> = ptr[1..(Self::FIELD_COUNT+1)]
                        .iter()
                        .map(|x| u32::from_le(*x) as usize)
                        .collect();
                    if offsets[0] != expected {
                        let err = VerificationError::FirstOffsetIsShort(
                            stringify!(#name).to_owned(), expected, offsets[0]);
                        Err(err)?;
                    }
                    offsets.push(total_size);
                    if offsets.windows(2).any(|i| i[0] + 4 > i[1]) {
                        let err = VerificationError::OffsetsNotMatch(stringify!(#name).to_owned());
                        Err(err)?;
                    }
                    #( #verify_fields )*
                    Ok(())
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    impl_reader_traits(writer, &name)?;
    writeln!(writer)
}

/*
 * Builder
 */

fn builder_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    if name == ast::ATOM_NAME {
        m4::Ident::new(ATOM_NAME, span)
    } else {
        m4::Ident::new(&format!("{}Builder", name).to_camel(), span)
    }
}

fn build_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    let name = builder_name(origin_name);
    let inner = builder_name(&info.typ.name);
    let item_count = usize_lit(info.item_count);
    {
        let code = if info.typ.is_atom() {
            quote!(
                #[derive(Debug)]
                pub struct #name<'m> ([#inner; #item_count], ::std::marker::PhantomData<&'m #inner>);
            )
        } else {
            quote!(
                #[derive(Debug)]
                pub struct #name<'m> ([#inner<'m>; #item_count]);
            )
        };
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn empty() -> Self {
                    #name([0; #item_count], ::std::marker::PhantomData)
                }
            )
        } else {
            let inner_array = (0..info.item_count)
                .map(|_| inner.clone())
                .collect::<Vec<_>>();
            quote!(
                pub fn empty() -> Self {
                    #name([#(#inner_array::empty(), )*])
                }
            )
        };
        defuns.push(code);
    }
    for idx in 0..info.item_count {
        let index = usize_lit(idx);
        let func = func_name(&format!("nth{}", idx));
        let code = if info.typ.is_atom() {
            quote!(
                pub fn #func(mut self, v: #inner) -> Self {
                    self.0[#index] = v;
                    self
                }
            )
        } else {
            quote!(
                pub fn #func<'n: 'm>(mut self, v: #inner<'n>) -> Self {
                    self.0[#index] = v;
                    self
                }
            )
        };
        defuns.push(code);
    }
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let owned = owned_name(origin_name);
        let total_size = usize_lit(info.item_size * info.item_count);
        let code = if info.typ.is_atom() {
            quote!(
                impl<'m> molecule::prelude::Builder for #name<'m> {
                    type Output = #owned;
                    fn calc_len(&self) -> usize {
                        #total_size
                    }
                    fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                        writer.write_all(&self.0)?;
                        Ok(())
                    }
                    fn build(&self) -> ::std::io::Result<Self::Output> {
                        use std::io::Write;
                        let mut inner: [u8; #total_size] = unsafe { ::std::mem::uninitialized() };
                        (&mut inner[..]).write_all(&self.0)?;
                        Ok(#owned(inner))
                    }
                }
            )
        } else {
            let idx = (0..info.item_count).map(usize_lit).collect::<Vec<_>>();
            quote!(
                impl<'m> molecule::prelude::Builder for #name<'m> {
                    type Output = #owned;
                    fn calc_len(&self) -> usize {
                        #total_size
                    }
                    fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                        #( self.0[#idx].write(writer)?; )*
                        Ok(())
                    }
                    fn build(&self) -> ::std::io::Result<Self::Output> {
                        let mut inner: [u8; #total_size] = unsafe { ::std::mem::uninitialized() };
                        let mut cursor = std::io::Cursor::new(&mut inner[..]);
                        self.write(&mut cursor)?;
                        Ok(#owned(inner))
                    }
                }
            )
        };
        write!(writer, "{}", code)?;
    }
    writeln!(writer)
}

fn build_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    let name = builder_name(origin_name);
    {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let field_type = builder_name(&field.typ.name);
            let code = if field.typ.is_atom() {
                quote!(#field_name: #field_type,)
            } else {
                quote!(#field_name: #field_type<'m>,)
            };
            fields.push(code);
        }
        let code = quote!(
            #[derive(Debug)]
            pub struct #name<'m> {
                #( #fields )*
            }
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let field_type = builder_name(&field.typ.name);
            let code = if field.typ.is_atom() {
                quote!(#field_name: 0,)
            } else {
                quote!(#field_name: #field_type::empty(),)
            };
            fields.push(code);
        }
        let code = quote!(
            pub fn empty() -> Self {
                #name {
                    #( #fields )*
                }
            }
        );
        defuns.push(code);
    }
    {
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let field_type = builder_name(&field.typ.name);
            let code = if field.typ.is_atom() {
                quote!(
                    pub fn #field_name(mut self, v: #field_type) -> Self {
                        self.#field_name = v;
                        self
                    }
                )
            } else {
                quote!(
                    pub fn #field_name<'n:'m>(mut self, v: #field_type<'n>) -> Self {
                        self.#field_name = v;
                        self
                    }
                )
            };
            defuns.push(code);
        }
    }
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let code = if field.typ.is_atom() {
                quote!(writer.write_all(&[self.#field_name])?;)
            } else {
                quote!(self.#field_name.write(writer)?;)
            };
            fields.push(code);
        }
        let owned = owned_name(origin_name);
        let total_size = usize_lit(info.field_size.iter().sum());
        let code = quote!(
            impl<'m> molecule::prelude::Builder for #name<'m> {
                type Output = #owned;
                fn calc_len(&self) -> usize {
                    #total_size
                }
                fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                    #( #fields )*
                    Ok(())
                }
                fn build(&self) -> ::std::io::Result<Self::Output> {
                    let mut inner: [u8; #total_size] = unsafe { ::std::mem::uninitialized() };
                    let mut cursor = std::io::Cursor::new(&mut inner[..]);
                    self.write(&mut cursor)?;
                    Ok(#owned(inner))
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    writeln!(writer)
}

fn build_fix_vec<W>(writer: &mut W, origin_name: &str, info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = builder_name(origin_name);
    let inner = builder_name(&info.typ.name);
    {
        let code = if info.typ.is_atom() {
            quote!(
                #[derive(Debug)]
                pub struct #name<'m> (Vec<#inner>, ::std::marker::PhantomData<&'m #inner>);
            )
        } else {
            quote!(
                #[derive(Debug)]
                pub struct #name<'m> (Vec<#inner<'m>>);
            )
        };
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn empty() -> Self {
                    #name(Vec::new(), ::std::marker::PhantomData)
                }
            )
        } else {
            quote!(
                pub fn empty() -> Self {
                    #name(Vec::new())
                }
            )
        };
        defuns.push(code);
    }
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn push(mut self, v: #inner) -> Self {
                    self.0.push(v);
                    self
                }
            )
        } else {
            quote!(
                pub fn push<'n: 'm>(mut self, v: #inner<'n>) -> Self {
                    self.0.push(v);
                    self
                }
            )
        };
        defuns.push(code);
    }
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let owned = owned_name(origin_name);
        let item_size = usize_lit(info.item_size);
        let code = if info.typ.is_atom() {
            quote!(
                impl<'m> molecule::prelude::Builder for #name<'m> {
                    type Output = #owned;
                    fn calc_len(&self) -> usize {
                        4 + self.0.len()
                    }
                    fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                        let len = (self.0.len() as u32).to_le_bytes();
                        writer.write_all(&len)?;
                        writer.write_all(&self.0)?;
                        Ok(())
                    }
                    fn build(&self) -> ::std::io::Result<Self::Output> {
                        use std::io::Write;
                        let mut inner: Vec<u8> = Vec::with_capacity(4 + #item_size * self.0.len());
                        (&mut inner).write_all(&self.0)?;
                        Ok(#owned(inner))
                    }
                }
            )
        } else {
            quote!(
                impl<'m> molecule::prelude::Builder for #name<'m> {
                    type Output = #owned;
                    fn calc_len(&self) -> usize {
                        4 + #item_size * self.0.len()
                    }
                    fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                        let len = (self.0.len() as u32).to_le_bytes();
                        writer.write_all(&len)?;
                        for inner in &self.0[..] {
                            inner.write(writer)?;
                        }
                        Ok(())
                    }
                    fn build(&self) -> ::std::io::Result<Self::Output> {
                        let mut inner: Vec<u8> = Vec::with_capacity(4 + #item_size * self.0.len());
                        self.write(&mut inner)?;
                        Ok(#owned(inner))
                    }
                }
            )
        };
        write!(writer, "{}", code)?;
    }
    writeln!(writer)
}

fn build_dyn_vec<W>(writer: &mut W, origin_name: &str, info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = builder_name(origin_name);
    let inner = builder_name(&info.typ.name);
    {
        let code = quote!(
            #[derive(Debug)]
            pub struct #name<'m> (Vec<#inner<'m>>);
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let code = quote!(
            pub fn empty() -> Self {
                #name(Vec::new())
            }
        );
        defuns.push(code);
    }
    {
        let code = quote!(
            pub fn push<'n: 'm>(mut self, v: #inner<'n>) -> Self {
                self.0.push(v);
                self
            }
        );
        defuns.push(code);
    }
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let owned = owned_name(origin_name);
        let code = quote!(
            impl<'m> molecule::prelude::Builder for #name<'m> {
                type Output = #owned;
                fn calc_len(&self) -> usize {
                    4 + 4 * self.0.len() + self.0.iter().map(|inner| inner.calc_len()).sum::<usize>()
                }
                fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                    {
                        let len = (self.calc_len() as u32).to_le_bytes();
                        writer.write_all(&len[..])?;
                    }
                    let mut offset = 4 + 4 * self.0.len();
                    for inner in &self.0[..] {
                        let tmp = (offset as u32).to_le_bytes();
                        writer.write_all(&tmp[..])?;
                        offset += inner.calc_len();
                    }
                    for inner in &self.0[..] {
                        inner.write(writer)?;
                    }
                    Ok(())
                }
                fn build(&self) -> ::std::io::Result<Self::Output> {
                    let mut inner: Vec<u8> = Vec::with_capacity(self.calc_len());
                    self.write(&mut inner)?;
                    Ok(#owned(inner))
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    writeln!(writer)
}

fn build_table<W>(writer: &mut W, origin_name: &str, info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    let name = builder_name(origin_name);
    {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let field_type = builder_name(&field.typ.name);
            let code = if field.typ.is_atom() {
                quote!(#field_name: #field_type,)
            } else {
                quote!(#field_name: #field_type<'m>,)
            };
            fields.push(code);
        }
        let code = quote!(
            #[derive(Debug)]
            pub struct #name<'m> {
                #( #fields )*
            }
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let field_type = builder_name(&field.typ.name);
            let code = if field.typ.is_atom() {
                quote!(#field_name: 0,)
            } else {
                quote!(#field_name: #field_type::empty(),)
            };
            fields.push(code);
        }
        let code = quote!(
            pub fn empty() -> Self {
                #name {
                    #( #fields )*
                }
            }
        );
        defuns.push(code);
    }
    {
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let field_type = builder_name(&field.typ.name);
            let code = if field.typ.is_atom() {
                quote!(
                    pub fn #field_name(mut self, v: #field_type) -> Self {
                        self.#field_name = v;
                        self
                    }
                )
            } else {
                quote!(
                    pub fn #field_name<'n:'m>(mut self, v: #field_type<'n>) -> Self {
                        self.#field_name = v;
                        self
                    }
                )
            };
            defuns.push(code);
        }
    }
    def_funcs_with_lifetime(writer, &name, defuns)?;
    {
        let mut fields: Vec<m4::TokenStream> = Vec::new();
        let mut lengths: Vec<m4::TokenStream> = Vec::new();
        for field in info.inner.iter() {
            let field_name = func_name(&field.name);
            let code = if field.typ.is_atom() {
                quote!(writer.write_all(&[self.#field_name])?;)
            } else {
                quote!(self.#field_name.write(writer)?;)
            };
            fields.push(code);
            let code = if field.typ.is_atom() {
                quote!(1)
            } else {
                quote!(self.#field_name.calc_len())
            };
            lengths.push(code);
        }
        let owned = owned_name(origin_name);
        let field_count = usize_lit(info.inner.len());
        let lengths1 = &lengths;
        let lengths2 = &lengths;
        let code = quote!(
            impl<'m> molecule::prelude::Builder for #name<'m> {
                type Output = #owned;
                fn calc_len(&self) -> usize {
                    let len_header = 4 + #field_count * 4;
                    len_header #(+ #lengths1)*
                }
                fn write<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
                    {
                        let len = (self.calc_len() as u32).to_le_bytes();
                        writer.write_all(&len[..])?;
                    }
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
                fn build(&self) -> ::std::io::Result<Self::Output> {
                    let mut inner: Vec<u8> = Vec::with_capacity(self.calc_len());
                    self.write(&mut inner)?;
                    Ok(#owned(inner))
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    writeln!(writer)
}

/*
 * Owned
 */

fn owned_name(name: &str) -> m4::Ident {
    let span = m4::Span::call_site();
    if name == ast::ATOM_NAME {
        m4::Ident::new(ATOM_NAME, span)
    } else {
        m4::Ident::new(&name.to_camel(), span)
    }
}

fn owned_array<W>(writer: &mut W, origin_name: &str, info: &ast::Array) -> io::Result<()>
where
    W: io::Write,
{
    let name = owned_name(origin_name);
    {
        let total_size = usize_lit(info.item_size * info.item_count);
        let code = quote!(
            #[derive(Debug)]
            pub struct #name ([u8; #total_size]);
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let reader = reader_name(origin_name);
        let code = quote!(
            pub fn as_reader(&self) -> #reader {
                #reader(&self.0[..])
            }
        );
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    writeln!(writer)
}

fn owned_struct<W>(writer: &mut W, origin_name: &str, info: &ast::Struct) -> io::Result<()>
where
    W: io::Write,
{
    let name = owned_name(origin_name);
    {
        let total_size = usize_lit(info.field_size.iter().sum());
        let code = quote!(
            #[derive(Debug)]
            pub struct #name ([u8; #total_size]);
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let reader = reader_name(origin_name);
        let code = quote!(
            pub fn as_reader(&self) -> #reader {
                #reader(&self.0[..])
            }
        );
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    writeln!(writer)
}

fn owned_fix_vec<W>(writer: &mut W, origin_name: &str, _info: &ast::FixedVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = owned_name(origin_name);
    {
        let code = quote!(
            #[derive(Debug)]
            pub struct #name (Vec<u8>);
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let reader = reader_name(origin_name);
        let code = quote!(
            pub fn as_reader(&self) -> #reader {
                #reader(&self.0[..])
            }
        );
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    writeln!(writer)
}

fn owned_dyn_vec<W>(writer: &mut W, origin_name: &str, _info: &ast::DynamicVector) -> io::Result<()>
where
    W: io::Write,
{
    let name = owned_name(origin_name);
    {
        let code = quote!(
            #[derive(Debug)]
            pub struct #name (Vec<u8>);
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let reader = reader_name(origin_name);
        let code = quote!(
            pub fn as_reader(&self) -> #reader {
                #reader(&self.0[..])
            }
        );
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    writeln!(writer)
}

fn owned_table<W>(writer: &mut W, origin_name: &str, _info: &ast::Table) -> io::Result<()>
where
    W: io::Write,
{
    let name = owned_name(origin_name);
    {
        let code = quote!(
            #[derive(Debug)]
            pub struct #name (Vec<u8>);
        );
        write!(writer, "{}", code)?;
    }
    let mut defuns: Vec<m4::TokenStream> = Vec::new();
    {
        let reader = reader_name(origin_name);
        let code = quote!(
            pub fn as_reader(&self) -> #reader {
                #reader(&self.0[..])
            }
        );
        defuns.push(code);
    }
    def_funcs(writer, &name, defuns)?;
    writeln!(writer)
}
