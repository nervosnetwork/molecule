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

fn decoder(spec: IntegerSpec) -> TokenStream {
    let native = native(spec);
    let from_bytes = match spec.byte_order {
        ByteOrder::Little => quote!(from_le_bytes),
        ByteOrder::Big => quote!(from_be_bytes),
    };
    quote!(<#native>::#from_bytes)
}

pub(super) fn decode(spec: IntegerSpec, bytes: TokenStream) -> TokenStream {
    let decode = decoder(spec);
    quote!(#decode(#bytes))
}

fn to_bytes(spec: IntegerSpec) -> proc_macro2::Ident {
    match spec.byte_order {
        ByteOrder::Little => format_ident!("to_le_bytes"),
        ByteOrder::Big => format_ident!("to_be_bytes"),
    }
}

fn value_constructor(spec: IntegerSpec) -> proc_macro2::Ident {
    format_ident!("from_{}{}", if spec.signed { "i" } else { "u" }, spec.bits)
}

fn value_getter(spec: IntegerSpec) -> proc_macro2::Ident {
    format_ident!(
        "try_to_{}{}",
        if spec.signed { "i" } else { "u" },
        spec.bits
    )
}

pub(super) fn rust(ast: &ast::Ast, lazy: bool) -> io::Result<TokenStream> {
    let mut output = TokenStream::new();
    for decl in ast.major_decls() {
        if let Some(&spec) = ast.integer(decl.name()) {
            output.extend(scalar_methods(decl.name(), spec, lazy));
        }
        if lazy {
            output.extend(lazy_container_methods(ast, &decl));
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
    let entity = ident_name(decl.name(), "");
    let reader = ident_name(decl.name(), "Reader");
    let native = native(spec);
    if matches!(decl, ast::TopDecl::Option_(_)) {
        let encode = to_bytes(spec);
        let to_value = value_getter(spec);
        quote! {
            impl #entity {
                pub fn from_value(value: ::core::option::Option<#native>) -> Self {
                    match value {
                        ::core::option::Option::Some(value) => {
                            <#reader<'_> as molecule::prelude::Reader<'_>>::to_entity(
                                &<#reader<'_> as molecule::prelude::Reader<'_>>::new_unchecked(&value.#encode())
                            )
                        }
                        ::core::option::Option::None => <Self as ::core::default::Default>::default(),
                    }
                }
                pub fn try_value(&self) -> ::core::result::Result<::core::option::Option<#native>, molecule::integer::IntegerError> {
                    Self::as_reader(self).try_value()
                }
            }
            impl<'r> #reader<'r> {
                pub fn try_value(&self) -> ::core::result::Result<::core::option::Option<#native>, molecule::integer::IntegerError> {
                    self.to_opt().map(|item| item.#to_value()).transpose()
                }
            }
        }
    } else {
        let width = spec.byte_width();
        let encode = to_bytes(spec);
        let decode = decoder(spec);
        quote! {
            impl #entity {
                /// Encode native values directly, checking the wire size before allocating.
                pub fn try_from_values(values: &[#native]) -> ::core::result::Result<Self, molecule::integer::IntegerError> {
                    Self::try_from_values_by(values, |value| *value)
                }
                /// Project each item to its native value while writing the final wire buffer.
                pub fn try_from_values_by<T>(items: &[T], mut value: impl ::core::ops::FnMut(&T) -> #native) -> ::core::result::Result<Self, molecule::integer::IntegerError> {
                    molecule::integer::pack_fixvec(items, |item| value(item).#encode()).map(<Self as molecule::prelude::Entity>::new_unchecked)
                }
                /// Check the fixed-vector header once and decode borrowed elements on demand.
                pub fn try_values(&self) -> ::core::result::Result<impl ::core::iter::ExactSizeIterator<Item = #native> + ::core::iter::DoubleEndedIterator + '_, molecule::integer::IntegerError> {
                    Self::as_reader(self).try_values()
                }
            }
            impl<'r> #reader<'r> {
                /// Check the fixed-vector header once and decode borrowed elements on demand.
                pub fn try_values(self) -> ::core::result::Result<impl ::core::iter::ExactSizeIterator<Item = #native> + ::core::iter::DoubleEndedIterator + 'r, molecule::integer::IntegerError> {
                    let wire = <Self as molecule::prelude::Reader<'r>>::as_slice(&self);
                    <Self as molecule::prelude::Reader<'r>>::verify(wire, false)?;
                    let (chunks, _) = wire[molecule::NUMBER_SIZE..].as_chunks::<#width>();
                    ::core::result::Result::Ok(chunks.iter().copied().map(#decode))
                }
            }
        }
    }
}

fn lazy_container_methods(ast: &ast::Ast, decl: &ast::TopDecl) -> TokenStream {
    let item = match decl {
        ast::TopDecl::Option_(option) => option.item(),
        ast::TopDecl::FixVec(vector) => vector.item(),
        ast::TopDecl::Array(array) => array.item(),
        _ => return TokenStream::new(),
    };
    let Some(&spec) = ast.integer(item.typ().name()) else {
        return TokenStream::new();
    };
    let name = ident_name(decl.name(), "");
    let width = spec.byte_width();
    let native = native(spec);
    let decode = decoder(spec);
    match decl {
        ast::TopDecl::Option_(_) => quote! {
            impl #name {
                pub fn try_value(&self) -> ::core::result::Result<::core::option::Option<#native>, molecule::integer::IntegerError> {
                    ::core::result::Result::Ok(molecule::integer::read_optional_wire_bytes::<#width>(&self.cursor)?.map(#decode))
                }
            }
        },
        ast::TopDecl::FixVec(_) => quote! {
            impl #name {
                pub fn get_value(&self, index: usize) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    ::core::result::Result::Ok(#decode(molecule::integer::read_fixvec_item::<#width>(&self.cursor, index)?))
                }
            }
        },
        ast::TopDecl::Array(array) => {
            let count = array.item_count();
            quote! {
                impl #name {
                    pub fn get_value(&self, index: usize) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                        ::core::result::Result::Ok(#decode(molecule::integer::read_array_item::<#width>(&self.cursor, #count, index)?))
                    }
                }
            }
        }
        _ => unreachable!(),
    }
}

fn scalar_methods(raw_name: &str, spec: IntegerSpec, lazy: bool) -> TokenStream {
    let name = ident_name(raw_name, "");
    let width = spec.byte_width();
    let native = native(spec);
    let to_value = value_getter(spec);
    let decode = decoder(spec);
    if lazy {
        quote! {
            impl #name {
                /// Read exactly the declared width using its explicit byte order.
                pub fn #to_value(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    ::core::result::Result::Ok(#decode(molecule::integer::read_wire_bytes::<#width>(&self.cursor)?))
                }
            }
        }
    } else {
        let reader = ident_name(raw_name, "Reader");
        let from_value = value_constructor(spec);
        let to_bytes = to_bytes(spec);
        quote! {
            impl #name {
                pub fn #from_value(value: #native) -> Self {
                    <Self as ::core::convert::From<[u8; #width]>>::from(value.#to_bytes())
                }
                pub fn #to_value(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    Self::as_reader(self).#to_value()
                }
            }
            impl<'r> #reader<'r> {
                pub fn try_as_wire_bytes(&self) -> ::core::result::Result<&'r [u8; #width], molecule::integer::IntegerError> {
                    molecule::integer::wire_bytes(<Self as molecule::prelude::Reader<'r>>::as_slice(self))
                }
                pub fn #to_value(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                    ::core::result::Result::Ok(#decode(*self.try_as_wire_bytes()?))
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
    let mut seen_accessors = std::collections::HashSet::new();
    let mut offset = 0;
    for (index, field) in fields.iter().enumerate() {
        let start = offset;
        if let ast::TopDecl::Struct(record) = decl {
            offset += record.field_sizes()[index];
        }
        let (spec, optional) = match (ast.integer(field.typ().name()), field.typ().as_ref()) {
            (Some(&spec), _) => (spec, false),
            (None, ast::TopDecl::Option_(option)) if total_size.is_none() => {
                let Some(&spec) = ast.integer(option.item().typ().name()) else {
                    continue;
                };
                (spec, true)
            }
            _ => continue,
        };
        let accessor = field_name(&format!("{}_value", field.name()));
        if fields
            .iter()
            .any(|candidate| field_name(candidate.name()) == accessor)
            || !seen_accessors.insert(accessor.to_string())
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
        let decode = decoder(spec);
        let count = fields.len();
        output.extend(match (total_size, optional) {
            (Some(total), _) => quote! {
                impl #name {
                    pub fn #accessor(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                        ::core::result::Result::Ok(#decode(molecule::integer::read_fixed_field::<#width>(&self.cursor, #total, #start)?))
                    }
                }
            },
            (None, false) => quote! {
                impl #name {
                    pub fn #accessor(&self) -> ::core::result::Result<#native, molecule::integer::IntegerError> {
                        let field = self.cursor.table_field_exact(#count, #index)?;
                        ::core::result::Result::Ok(#decode(molecule::integer::read_wire_bytes::<#width>(&field)?))
                    }
                }
            },
            (None, true) => quote! {
                impl #name {
                    pub fn #accessor(&self) -> ::core::result::Result<::core::option::Option<#native>, molecule::integer::IntegerError> {
                        let field = self.cursor.table_field_exact(#count, #index)?;
                        ::core::result::Result::Ok(molecule::integer::read_optional_wire_bytes::<#width>(&field)?.map(#decode))
                    }
                }
            },
        });
    }
    Ok(output)
}
