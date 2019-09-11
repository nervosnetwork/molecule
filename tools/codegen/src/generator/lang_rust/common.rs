use std::io;

use proc_macro2 as m4;
use quote::quote;

use crate::ast::verified as ast;

use super::utilities::*;

pub(super) fn def_entity_and_reader<W>(writer: &mut W, origin_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let code = quote!(
        #[derive(Clone)]
        pub struct #entity(molecule::bytes::Bytes);
        #[derive(Clone, Copy)]
        pub struct #reader<'r>(&'r [u8]);

        impl ::std::fmt::Debug for #entity {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}(0x{})", Self::NAME, hex_string(self.as_slice()).unwrap())
            }
        }
        impl<'r> ::std::fmt::Debug for #reader<'r> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}(0x{})", Self::NAME, hex_string(self.as_slice()).unwrap())
            }
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_display_for_entity_and_reader<W>(
    writer: &mut W,
    origin_name: &str,
    stmts: m4::TokenStream,
) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let reader = reader_name(origin_name);
    let code = quote!(
        impl ::std::fmt::Display for #entity {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #stmts
            }
        }
        impl<'r> ::std::fmt::Display for #reader<'r> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #stmts
            }
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_default_for_entity<W>(
    writer: &mut W,
    origin_name: &str,
    content: Vec<u8>,
) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let bytes = content.into_iter().map(|b| usize_lit(b as usize));
    let code = quote!(
        impl ::std::default::Default for #entity {
            fn default() -> Self {
                let v: Vec<u8> = vec![#( #bytes, )*];
                #entity::new_unchecked(v.into())
            }
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn def_items_for_union<W>(
    writer: &mut W,
    origin_name: &str,
    info: &ast::Union,
) -> io::Result<()>
where
    W: io::Write,
{
    let entity_union = entity_union_name(origin_name);
    let reader_union = reader_union_name(origin_name);
    let entity_union_string = entity_union.to_string();
    let reader_union_string = reader_union.to_string();
    let (
        ref entity_inners,
        ref reader_inners,
        ref union_items,
        ref union_ids,
        ref entity_union_item_paths,
        ref reader_union_item_paths,
    ) = info.inner.iter().enumerate().fold(
        (
            Vec::with_capacity(info.inner.len()),
            Vec::with_capacity(info.inner.len()),
            Vec::with_capacity(info.inner.len()),
            Vec::with_capacity(info.inner.len()),
            Vec::with_capacity(info.inner.len()),
            Vec::with_capacity(info.inner.len()),
        ),
        |(
            mut entity_inners,
            mut reader_inners,
            mut union_items,
            mut union_ids,
            mut entity_union_item_paths,
            mut reader_union_item_paths,
        ),
         (index, inner)| {
            let entity_name = entity_name(&inner.typ.name);
            let reader_name = reader_name(&inner.typ.name);
            let item_name = union_item_name(&inner.typ.name);
            let item_id = usize_lit(index + 1);
            let entity_union_item_path = quote!(#entity_union::#item_name);
            let reader_union_item_path = quote!(#reader_union::#item_name);
            entity_inners.push(entity_name);
            reader_inners.push(reader_name);
            union_items.push(item_name);
            union_ids.push(item_id);
            entity_union_item_paths.push(entity_union_item_path);
            reader_union_item_paths.push(reader_union_item_path);
            (
                entity_inners,
                reader_inners,
                union_items,
                union_ids,
                entity_union_item_paths,
                reader_union_item_paths,
            )
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
                NotSet,
                #( #union_items(#entity_inners), )*
            }
            #[derive(Debug, Clone, Copy)]
            pub enum #reader_union<'r> {
                NotSet,
                #( #union_items(#reader_inners<'r>), )*
            }

            impl ::std::default::Default for #entity_union {
                fn default() -> Self {
                    #entity_union::#entity_default
                }
            }

            impl ::std::fmt::Display for #entity_union {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        #(
                            #entity_union_item_paths(ref item) => {
                                write!(f, "{}::{}({})", Self::NAME, #union_items::NAME, item)
                            }
                        )*
                        #entity_union::NotSet => { write!(f, "NotSet") }
                    }
                }
            }
            impl<'r> ::std::fmt::Display for #reader_union<'r> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        #(
                            #reader_union_item_paths(ref item) => {
                                write!(f, "{}::{}({})", Self::NAME, #union_items::NAME, item)
                            }
                        )*
                        #reader_union::NotSet => { write!(f, "NotSet") }
                    }
                }
            }

            impl #entity_union {
                pub(crate) fn display_inner(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        #( #entity_union_item_paths(ref item) => write!(f, "{}", item), )*
                        #entity_union::NotSet => { write!(f, "NotSet") }
                    }
                }
            }
            impl<'r> #reader_union<'r> {
                pub(crate) fn display_inner(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        #( #reader_union_item_paths(ref item) => write!(f, "{}", item), )*
                        #reader_union::NotSet => { write!(f, "NotSet") }
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    let union_items_string = &union_items
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
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
                pub const NAME: &'static str = #entity_union_string;
                pub fn as_bytes(&self) -> molecule::bytes::Bytes {
                    match self {
                        #( #entity_union_item_paths(item) => item.as_bytes(), )*
                        #entity_union::NotSet => Default::default(),
                    }
                }
                pub fn as_slice(&self) -> &[u8] {
                    match self {
                        #( #entity_union_item_paths(item) => item.as_slice(), )*
                        #entity_union::NotSet => &[],
                    }
                }
                pub fn item_id(&self) -> molecule::ItemId {
                    match self {
                        #( #entity_union_item_paths(_) => #union_ids, )*
                        #entity_union::NotSet => 0,
                    }
                }
                pub fn item_name(&self) -> &str {
                    match self {
                        #( #entity_union_item_paths(_) => #union_items_string, )*
                        #entity_union::NotSet => "NotSet",
                    }
                }
                pub fn as_reader<'r>(&'r self) -> #reader_union<'r> {
                    match self {
                        #( #entity_union_item_paths(item) => item.as_reader().into(), )*
                        #entity_union::NotSet => #reader_union::NotSet,
                    }
                }
            }
            impl<'r> #reader_union<'r> {
                pub const NAME: &'r str = #reader_union_string;
                pub fn as_slice(&self) -> &'r [u8] {
                    match self {
                        #( #reader_union_item_paths(item) => item.as_slice(), )*
                        #reader_union::NotSet => &[],
                    }
                }
                pub fn item_id(&self) -> molecule::ItemId {
                    match self {
                        #( #reader_union_item_paths(_) => #union_ids, )*
                        #reader_union::NotSet => 0,
                    }
                }
                pub fn item_name(&self) -> &str {
                    match self {
                        #( #reader_union_item_paths(_) => #union_items_string, )*
                        #reader_union::NotSet => "NotSet",
                    }
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    Ok(())
}

pub(super) fn impl_trait_entity<W>(
    writer: &mut W,
    origin_name: &str,
    funcs: Vec<m4::TokenStream>,
) -> io::Result<()>
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
            fn as_bytes(&self) -> molecule::bytes::Bytes {
                self.0.clone()
            }
            fn as_slice(&self) -> &[u8] {
                &self.0[..]
            }
            fn from_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
                #reader::from_slice(slice).map(|reader| reader.to_entity())
            }
            fn from_compatible_slice(slice: &[u8]) -> molecule::error::VerificationResult<Self> {
                #reader::from_compatible_slice(slice).map(|reader| reader.to_entity())
            }
            fn new_builder() -> Self::Builder {
                ::std::default::Default::default()
            }
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_entity<W>(
    writer: &mut W,
    origin_name: &str,
    funcs: Vec<m4::TokenStream>,
) -> io::Result<()>
where
    W: io::Write,
{
    let entity = entity_name(origin_name);
    let entity_string = entity.to_string();
    let reader = reader_name(origin_name);
    let code = quote!(
        impl #entity {
            pub const NAME: &'static str = #entity_string;
            pub fn as_reader<'r>(&'r self) -> #reader<'r> {
                #reader::new_unchecked(self.as_slice())
            }
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_trait_reader<W>(
    writer: &mut W,
    origin_name: &str,
    funcs: m4::TokenStream,
) -> io::Result<()>
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
            fn as_slice(&self) -> &'r [u8] {
                self.0
            }
            #funcs
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_reader<W>(
    writer: &mut W,
    origin_name: &str,
    funcs: Vec<m4::TokenStream>,
) -> io::Result<()>
where
    W: io::Write,
{
    let reader = reader_name(origin_name);
    let reader_string = reader.to_string();
    let code = quote!(
        impl<'r> #reader<'r> {
            pub const NAME: &'r str = #reader_string;
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_trait_builder<W>(
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
            fn build(&self) -> Self::Entity {
                let mut inner = Vec::with_capacity(self.expected_length());
                self.write(&mut inner).expect("write vector should be ok");
                #entity::new_unchecked(inner.into())
            }
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn impl_builder<W>(
    writer: &mut W,
    origin_name: &str,
    funcs: Vec<m4::TokenStream>,
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let builder_string = builder.to_string();
    let code = quote!(
        impl #builder {
            pub const NAME: &'static str = #builder_string;
            #( #funcs )*
        }
    );
    write!(writer, "{}", code)
}

pub(super) fn def_iterator_for_vector<W>(
    writer: &mut W,
    origin_name: &str,
    inner_name: &str,
    is_atom: bool,
) -> io::Result<()>
where
    W: io::Write,
{
    let entity_iterator = entity_iterator_name(origin_name);
    let entity = entity_name(origin_name);
    let entity_inner = entity_name(inner_name);
    let reader_iterator = reader_iterator_name(origin_name);
    let reader = reader_name(origin_name);
    let reader_inner = reader_name(inner_name);
    let code = quote!(
        pub struct #entity_iterator (#entity, usize, usize);
        impl ::std::iter::Iterator for #entity_iterator {
            type Item = #entity_inner;
            fn next(&mut self) -> Option<Self::Item> {
                if self.1 >= self.2 {
                    None
                } else {
                    let ret = self.0.get_unchecked(self.1);
                    self.1 += 1;
                    Some(ret)
                }
            }
        }
        impl ::std::iter::ExactSizeIterator for #entity_iterator {
            fn len(&self) -> usize {
                self.2 - self.1
            }
        }
        impl ::std::iter::IntoIterator for #entity {
            type Item = #entity_inner;
            type IntoIter = #entity_iterator;
            fn into_iter(self) -> Self::IntoIter {
                let len = self.len();
                #entity_iterator(self, 0, len)
            }
        }
    );
    write!(writer, "{}", code)?;
    if !is_atom {
        let code = quote!(
            impl<'r> #reader<'r> {
                pub fn iter<'t>(&'t self) -> #reader_iterator<'t, 'r> {
                    #reader_iterator(&self, 0, self.len())
                }
            }
            pub struct #reader_iterator<'t, 'r> (&'t #reader<'r>, usize, usize);
            impl<'t: 'r, 'r> ::std::iter::Iterator for #reader_iterator<'t, 'r> {
                type Item = #reader_inner<'t>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.1 >= self.2 {
                        None
                    } else {
                        let ret = self.0.get_unchecked(self.1);
                        self.1 += 1;
                        Some(ret)
                    }
                }
            }
            impl<'t: 'r, 'r> ::std::iter::ExactSizeIterator for #reader_iterator<'t, 'r> {
                fn len(&self) -> usize {
                    self.2 - self.1
                }
            }
        );
        write!(writer, "{}", code)?;
    }
    Ok(())
}
