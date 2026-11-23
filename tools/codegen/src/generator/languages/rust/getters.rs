use proc_macro2 as m4;
use quote::quote;

use super::utilities::{
    entity_name, entity_new_unchecked, entity_union_name, func_name, reader_name,
    reader_new_unchecked, reader_union_name, usize_lit,
};
use crate::ast::{self as ast, HasName};

pub(super) trait ImplGetters: HasName {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream;

    fn impl_getters(&self, is_entity: bool) -> m4::TokenStream {
        let getters = self.impl_getters_internal(is_entity);
        quote!(
            #getters
        )
    }

    fn impl_getters_for_entity(&self) -> m4::TokenStream {
        self.impl_getters(true)
    }

    fn impl_getters_for_reader(&self) -> m4::TokenStream {
        self.impl_getters(false)
    }
}

impl ImplGetters for ast::Option_ {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let typ = self.item().typ();
        let (getter_ret, ctor) = if is_entity {
            let inner = entity_name(typ.name());
            (
                quote!(#inner),
                entity_new_unchecked(typ, quote!(self.0.clone())),
            )
        } else {
            let inner = reader_name(typ.name());
            (
                quote!(#inner<'r>),
                reader_new_unchecked(typ, quote!(self.0)),
            )
        };
        quote!(
            pub fn to_opt(&self) -> ::core::option::Option<#getter_ret> {
                if self.is_none() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(#ctor)
                }
            }
        )
    }
}

impl ImplGetters for ast::Union {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let (getter_ret, getter_stmt) = if is_entity {
            let union = entity_union_name(self.name());
            let getter_ret = quote!(#union);
            let getter_stmt = quote!(self.0.slice(molecule::NUMBER_SIZE..));
            (getter_ret, getter_stmt)
        } else {
            let union = reader_union_name(self.name());
            let getter_ret = quote!(#union<'r>);
            let getter_stmt = quote!(&self.0[molecule::NUMBER_SIZE..]);
            (getter_ret, getter_stmt)
        };
        let match_stmts = self.items().iter().map(|item| {
            let item_id = usize_lit(item.id());
            let ctor = if is_entity {
                entity_new_unchecked(item.typ(), quote!(inner))
            } else {
                reader_new_unchecked(item.typ(), quote!(inner))
            };
            quote!(#item_id => #ctor.into(),)
        });
        quote!(
            pub fn to_enum(&self) -> #getter_ret {
                let inner = #getter_stmt;
                match self.item_id() {
                    #( #match_stmts )*
                    _ => panic!("{}: invalid data", Self::NAME),
                }
            }
        )
    }
}

impl ImplGetters for ast::Array {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let typ = self.item().typ();
        let (getter_ret, getter_ret_byte, getter_stmt_byte) = if is_entity {
            let inner = entity_name(typ.name());
            (
                quote!(#inner),
                quote!(molecule::bytes::Bytes),
                quote!(self.0.clone()),
            )
        } else {
            let inner = reader_name(typ.name());
            (quote!(#inner<'r>), quote!(&'r [u8]), quote!(self.0))
        };
        let each_getter = (0..self.item_count())
            .map(|i| {
                let func = func_name(&format!("nth{}", i));
                let start = usize_lit(self.item_size() * i);
                let end = usize_lit(self.item_size() * (i + 1));
                let ctor = if is_entity {
                    entity_new_unchecked(typ, quote!(self.0.slice(#start..#end)))
                } else {
                    reader_new_unchecked(typ, quote!(&self.0[#start..#end]))
                };
                quote!(
                    pub fn #func(&self) -> #getter_ret {
                        #ctor
                    }
                )
            })
            .collect::<Vec<_>>();
        if typ.is_byte() {
            quote!(
                #( #each_getter )*
                pub fn raw_data(&self) -> #getter_ret_byte {
                    #getter_stmt_byte
                }
            )
        } else {
            quote!(
                #( #each_getter )*
            )
        }
    }
}

impl ImplGetters for ast::Struct {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let (_, each_getter) = self.fields().iter().zip(self.field_sizes().iter()).fold(
            (0, Vec::with_capacity(self.fields().len())),
            |(mut offset, mut getters), (f, s)| {
                let func = func_name(f.name());
                let start = usize_lit(offset);
                offset += s;
                let end = usize_lit(offset);
                let typ = f.typ();
                let (getter_ret, ctor) = if is_entity {
                    let inner = entity_name(typ.name());
                    (
                        quote!(#inner),
                        entity_new_unchecked(typ, quote!(self.0.slice(#start..#end))),
                    )
                } else {
                    let inner = reader_name(typ.name());
                    (
                        quote!(#inner<'r>),
                        reader_new_unchecked(typ, quote!(&self.0[#start..#end])),
                    )
                };
                let getter = quote!(
                    pub fn #func(&self) -> #getter_ret {
                        #ctor
                    }
                );
                getters.push(getter);
                (offset, getters)
            },
        );
        quote!(
            #( #each_getter )*
        )
    }
}

impl ImplGetters for ast::FixVec {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let typ = self.item().typ();
        let (getter_ret, ctor, getter_ret_byte, getter_stmt_byte) = if is_entity {
            let inner = entity_name(typ.name());
            (
                quote!(#inner),
                entity_new_unchecked(typ, quote!(self.0.slice(start..end))),
                quote!(molecule::bytes::Bytes),
                quote!(self.0.slice(molecule::NUMBER_SIZE..)),
            )
        } else {
            let inner = reader_name(typ.name());
            (
                quote!(#inner<'r>),
                reader_new_unchecked(typ, quote!(&self.0[start..end])),
                quote!(&'r [u8]),
                quote!(&self.0[molecule::NUMBER_SIZE..]),
            )
        };
        let common_part = quote!(
            pub fn get(&self, idx: usize) -> ::core::option::Option<#getter_ret> {
                if idx >= self.len() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(self.get_unchecked(idx))
                }
            }
            pub fn get_unchecked(&self, idx: usize) -> #getter_ret {
                let start = molecule::NUMBER_SIZE + Self::ITEM_SIZE * idx;
                let end = start + Self::ITEM_SIZE;
                #ctor
            }
        );
        if typ.is_byte() {
            quote!(
                #common_part
                pub fn raw_data(&self) -> #getter_ret_byte {
                    #getter_stmt_byte
                }
            )
        } else {
            common_part
        }
    }
}

impl ImplGetters for ast::DynVec {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let typ = self.item().typ();
        let (getter_ret, ctor_last, ctor) = if is_entity {
            let inner = entity_name(typ.name());
            (
                quote!(#inner),
                entity_new_unchecked(typ, quote!(self.0.slice(start..))),
                entity_new_unchecked(typ, quote!(self.0.slice(start..end))),
            )
        } else {
            let inner = reader_name(typ.name());
            (
                quote!(#inner<'r>),
                reader_new_unchecked(typ, quote!(&self.0[start..])),
                reader_new_unchecked(typ, quote!(&self.0[start..end])),
            )
        };
        quote!(
            pub fn get(&self, idx: usize) -> ::core::option::Option<#getter_ret> {
                if idx >= self.len() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(self.get_unchecked(idx))
                }
            }
            pub fn get_unchecked(&self, idx: usize) -> #getter_ret {
                let slice = &self.0[..];
                let start_idx = molecule::NUMBER_SIZE * (1 + idx);
                let start = molecule::unpack_number(&slice[start_idx..]) as usize;
                if idx == self.len() - 1 {
                    #ctor_last
                } else {
                    let end_idx = start_idx + molecule::NUMBER_SIZE ;
                    let end = molecule::unpack_number(&slice[end_idx..]) as usize;
                    #ctor
                }
            }
        )
    }
}

impl ImplGetters for ast::Table {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let each_getter = self
            .fields()
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let func = func_name(f.name());
                let typ = f.typ();
                let (getter_ret, ctor, ctor_last) = if is_entity {
                    let inner = entity_name(typ.name());
                    (
                        quote!(#inner),
                        entity_new_unchecked(typ, quote!(self.0.slice(start..end))),
                        entity_new_unchecked(typ, quote!(self.0.slice(start..))),
                    )
                } else {
                    let inner = reader_name(typ.name());
                    (
                        quote!(#inner<'r>),
                        reader_new_unchecked(typ, quote!(&self.0[start..end])),
                        reader_new_unchecked(typ, quote!(&self.0[start..])),
                    )
                };
                let start = usize_lit((i + 1) * molecule::NUMBER_SIZE);
                let end = usize_lit((i + 2) * molecule::NUMBER_SIZE);
                if i == self.fields().len() - 1 {
                    quote!(
                        pub fn #func(&self) -> #getter_ret {
                            let slice = &self.0[..];
                            let start = molecule::unpack_number(&slice[#start..]) as usize;
                            if self.has_extra_fields() {
                                let end = molecule::unpack_number(&slice[#end..]) as usize;
                                #ctor
                            } else {
                                #ctor_last
                            }
                        }
                    )
                } else {
                    quote!(
                        pub fn #func(&self) -> #getter_ret {
                            let slice = &self.0[..];
                            let start = molecule::unpack_number(&slice[#start..]) as usize;
                            let end = molecule::unpack_number(&slice[#end..]) as usize;
                            #ctor
                        }
                    )
                }
            })
            .collect::<Vec<_>>();
        quote!(
            #( #each_getter )*
        )
    }
}
