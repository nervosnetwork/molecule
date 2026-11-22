use crate::{
    ast::{self, HasName},
    generator::{field_name, ident_name},
    integer::{ByteOrder, IntegerSpec},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::io;

pub(super) fn native(spec: IntegerSpec) -> TokenStream {
    if spec.bits == 256 {
        quote!(molecule::integer::Uint256)
    } else {
        let name = format_ident!("{}{}", if spec.signed { "i" } else { "u" }, spec.bits);
        quote!(#name)
    }
}

pub(super) fn decode(spec: IntegerSpec, bytes: TokenStream) -> TokenStream {
    let native = native(spec);
    let from_bytes = match spec.byte_order {
        ByteOrder::Little => quote!(from_le_bytes),
        ByteOrder::Big => quote!(from_be_bytes),
    };
    quote!(<#native>::#from_bytes(#bytes))
}

fn to_bytes(spec: IntegerSpec) -> proc_macro2::Ident {
    match spec.byte_order {
        ByteOrder::Little => format_ident!("to_le_bytes"),
        ByteOrder::Big => format_ident!("to_be_bytes"),
    }
}

pub(super) fn rust(ast: &ast::Ast, lazy: bool) -> io::Result<TokenStream> {
    let mut output = TokenStream::new();
    for decl in ast.major_decls() {
        if let Some(&spec) = ast.integer(decl.name()) {
            output.extend(scalar_methods(decl.name(), spec, lazy));
        }
        if lazy {
            output.extend(lazy_field_methods(ast, &decl)?);
        } else {
            output.extend(container_methods(ast, &decl));
        }
    }
    Ok(output)
}

fn container_methods(ast: &ast::Ast, decl: &ast::TopDecl) -> TokenStream {
    let item = match decl {
        ast::TopDecl::Option_(option) => option.item(),
        ast::TopDecl::FixVec(vector) => vector.item(),
        _ => return TokenStream::new(),
    };
    let Some(&spec) = ast.integer(item.typ().name()) else {
        return TokenStream::new();
    };
    let reader = ident_name(decl.name(), "Reader");
    let native = native(spec);
    if matches!(decl, ast::TopDecl::Option_(_)) {
        let to_value = value_getter(spec);
        quote! {
            impl<'r> #reader<'r> {
                pub fn try_value(&self) -> ::core::result::Result<::core::option::Option<#native>, molecule::integer::IntegerError> {
                    self.to_opt().map(|item| item.#to_value()).transpose()
                }
            }
        }
    } else {
        let width = spec.byte_width();
        let entity = ident_name(decl.name(), "");
        let encode = to_bytes(spec);
        let value = decode(
            spec,
            quote!(::core::convert::TryInto::try_into(bytes).expect("exact-width chunk")),
        );
        quote! {
            impl #entity {
                /// Encode native values directly, checking the wire size before allocating.
                pub fn try_from_values(values: &[#native]) -> ::core::result::Result<Self, molecule::integer::IntegerError> {
                    Self::try_from_values_by(values, |value| *value)
                }
                /// Project each item to its native value while writing the final wire buffer.
                pub fn try_from_values_by<T>(items: &[T], mut value: impl ::core::ops::FnMut(&T) -> #native) -> ::core::result::Result<Self, molecule::integer::IntegerError> {
                    molecule::integer::pack_fixvec(items, |item| value(item).#encode()).map(Self::new_unchecked)
                }
            }
            impl<'r> #reader<'r> {
                /// Check the fixed-vector header once and decode borrowed elements on demand.
                pub fn try_values(self) -> molecule::error::VerificationResult<impl ::core::iter::ExactSizeIterator<Item = #native> + 'r> {
                    let wire = self.as_slice();
                    Self::verify(wire, false)?;
                    Ok(wire[molecule::NUMBER_SIZE..].chunks_exact(#width).map(|bytes| #value))
                }
            }
        }
    }
}

fn value_getter(spec: IntegerSpec) -> proc_macro2::Ident {
    format_ident!(
        "try_to_{}{}",
        if spec.signed { "i" } else { "u" },
        spec.bits
    )
}

fn scalar_methods(raw_name: &str, spec: IntegerSpec, lazy: bool) -> TokenStream {
    let name = ident_name(raw_name, "");
    let width = spec.byte_width();
    let native = native(spec);
    let suffix = format!("{}{}", if spec.signed { "i" } else { "u" }, spec.bits);
    let to_value = value_getter(spec);
    if lazy {
        let value = decode(spec, quote!(self.cursor.read_array::<#width>()?));
        quote! {
            impl #name {
                /// Read exactly the declared width using its explicit byte order.
                pub fn #to_value(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    Ok(#value)
                }
            }
        }
    } else {
        let reader = ident_name(raw_name, "Reader");
        let from_value = format_ident!("from_{}", suffix);
        let to_bytes = to_bytes(spec);
        let value = decode(spec, quote!(*self.try_as_wire_bytes()?));
        quote! {
            impl #name {
                pub fn #from_value(value: #native) -> Self {
                    <Self as ::core::convert::From<[u8; #width]>>::from(value.#to_bytes())
                }
                pub fn #to_value(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    self.as_reader().#to_value()
                }
            }
            impl<'r> #reader<'r> {
                pub fn try_as_wire_bytes(&self) -> ::core::result::Result<&'r [u8; #width], molecule::integer::IntegerError> {
                    molecule::integer::wire_bytes(self.as_slice())
                }
                pub fn #to_value(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    Ok(#value)
                }
            }
        }
    }
}

fn lazy_field_methods(ast: &ast::Ast, decl: &ast::TopDecl) -> io::Result<TokenStream> {
    let (fields, total_size) = match decl {
        ast::TopDecl::Struct(record) => (record.fields(), Some(record.total_size())),
        ast::TopDecl::Table(table) => (table.fields(), None),
        _ => return Ok(TokenStream::new()),
    };
    let name = ident_name(decl.name(), "");
    let mut output = TokenStream::new();
    let mut offset = 0;
    for (index, field) in fields.iter().enumerate() {
        let start = offset;
        if let ast::TopDecl::Struct(record) = decl {
            offset += record.field_sizes()[index];
        }
        let Some(&spec) = ast.integer(field.typ().name()) else {
            continue;
        };
        let accessor = field_name(&format!("{}_value", field.name()));
        if fields
            .iter()
            .any(|candidate| field_name(candidate.name()) == accessor)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "E_INT_ACCESSOR: {}.{} conflicts with an existing field getter",
                    decl.name(),
                    accessor
                ),
            ));
        }
        let width = spec.byte_width();
        let native = native(spec);
        let slice = if let Some(total) = total_size {
            quote! {
                if self.cursor.size != #total {
                    return Err(molecule::integer::IntegerError::Size { expected: #total, actual: self.cursor.size });
                }
                let field = self.cursor.slice_exact(#start, #width)?;
            }
        } else {
            let count = fields.len();
            quote! {
                let field = self.cursor.table_field_exact(#count, #index)?;
            }
        };
        let value = decode(spec, quote!(field.read_array::<#width>()?));
        output.extend(quote! {
            impl #name {
                pub fn #accessor(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    #slice
                    Ok(#value)
                }
            }
        });
    }
    Ok(output)
}
