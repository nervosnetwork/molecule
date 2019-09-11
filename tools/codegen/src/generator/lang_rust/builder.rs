use std::io;

use quote::quote;

use crate::ast::verified as ast;

use super::utilities::*;

pub(super) fn def_builder_for_option<W>(
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
        pub struct #builder (pub(crate) Option<#inner>);
    );
    write!(writer, "{}", code)
}

pub(super) fn def_builder_for_array<W>(
    writer: &mut W,
    origin_name: &str,
    info: &ast::Array,
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let inner = entity_name(&info.typ.name);
    let item_count = usize_lit(info.item_count);
    let code = quote!(
        pub struct #builder (pub(crate) [#inner; #item_count]);

        impl ::std::fmt::Debug for #builder {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}({:?})", Self::NAME, &self.0[..])
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

pub(super) fn def_builder_for_union<W>(writer: &mut W, origin_name: &str) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let entity_union = entity_union_name(origin_name);
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder (pub(crate) #entity_union);
    );
    write!(writer, "{}", code)
}

pub(super) fn def_builder_for_struct_or_table<W>(
    writer: &mut W,
    origin_name: &str,
    inner: &[ast::FieldDecl],
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let fields = inner.iter().map(|f| {
        let field_name = snake_name(&f.name);
        let field_type = entity_name(&f.typ.name);
        quote!(#field_name: #field_type,)
    });
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder { #( pub(crate) #fields )* }
    );
    write!(writer, "{}", code)
}

pub(super) fn def_builder_for_vector<W>(
    writer: &mut W,
    origin_name: &str,
    inner_name: &str,
) -> io::Result<()>
where
    W: io::Write,
{
    let builder = builder_name(origin_name);
    let inner = entity_name(&inner_name);
    let code = quote!(
        #[derive(Debug, Default)]
        pub struct #builder (pub(crate) Vec<#inner>);
    );
    write!(writer, "{}", code)
}
