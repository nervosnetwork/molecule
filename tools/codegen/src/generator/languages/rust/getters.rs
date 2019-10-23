use proc_macro2 as m4;
use quote::quote;

use super::utilities::{
    entity_name, entity_union_name, func_name, reader_name, reader_union_name, usize_lit,
};
use crate::ast::verified::{self as ast, HasName};

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
        let (inner, getter_ret, getter_stmt) = if is_entity {
            let inner = entity_name(self.typ.name());
            let getter_ret = quote!(#inner);
            let getter_stmt = quote!(self.0.clone());
            (inner, getter_ret, getter_stmt)
        } else {
            let inner = reader_name(self.typ.name());
            let getter_ret = quote!(#inner<'r>);
            let getter_stmt = quote!(self.as_slice());
            (inner, getter_ret, getter_stmt)
        };
        quote!(
            pub fn to_opt(&self) -> Option<#getter_ret> {
                if self.is_none() {
                    None
                } else {
                    Some(#inner::new_unchecked(#getter_stmt))
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
            let getter_stmt = quote!(self.0.slice_from(molecule::NUMBER_SIZE));
            (getter_ret, getter_stmt)
        } else {
            let union = reader_union_name(self.name());
            let getter_ret = quote!(#union<'r>);
            let getter_stmt = quote!(&self.as_slice()[molecule::NUMBER_SIZE..]);
            (getter_ret, getter_stmt)
        };
        let match_stmts = self.inner.iter().enumerate().map(|(index, inner)| {
            let item_id = usize_lit(index);
            let inner = if is_entity {
                entity_name(inner.typ.name())
            } else {
                reader_name(inner.typ.name())
            };
            quote!(#item_id => #inner::new_unchecked(inner).into(),)
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
        let (inner, getter_ret, getter_ret_atom, getter_stmt_atom) = if is_entity {
            let inner = entity_name(self.typ.name());
            let getter_ret = quote!(#inner);
            let getter_ret_atom = quote!(molecule::bytes::Bytes);
            let getter_stmt_atom = quote!(self.as_bytes());
            (inner, getter_ret, getter_ret_atom, getter_stmt_atom)
        } else {
            let inner = reader_name(self.typ.name());
            let getter_ret = quote!(#inner<'r>);
            let getter_ret_atom = quote!(&'r [u8]);
            let getter_stmt_atom = quote!(self.as_slice());
            (inner, getter_ret, getter_ret_atom, getter_stmt_atom)
        };
        let each_getter = (0..self.item_count)
            .map(|i| {
                let func = func_name(&format!("nth{}", i));
                let start = usize_lit(self.item_size * i);
                let end = usize_lit(self.item_size * (i + 1));
                let getter_stmt = if is_entity {
                    quote!(self.0.slice(#start, #end))
                } else {
                    quote!(&self.as_slice()[#start..#end])
                };
                quote!(
                    pub fn #func(&self) -> #getter_ret {
                        #inner::new_unchecked(#getter_stmt)
                    }
                )
            })
            .collect::<Vec<_>>();
        if self.typ.is_atom() {
            quote!(
                #( #each_getter )*
                pub fn raw_data(&self) -> #getter_ret_atom {
                    #getter_stmt_atom
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
        let (_, each_getter) = self.inner.iter().zip(self.field_size.iter()).fold(
            (0, Vec::with_capacity(self.inner.len())),
            |(mut offset, mut getters), (f, s)| {
                let func = func_name(&f.name);
                let (inner, getter_ret) = if is_entity {
                    let inner = entity_name(f.typ.name());
                    let getter_ret = quote!(#inner);
                    (inner, getter_ret)
                } else {
                    let inner = reader_name(f.typ.name());
                    let getter_ret = quote!(#inner<'r>);
                    (inner, getter_ret)
                };
                let start = usize_lit(offset);
                offset += s;
                let end = usize_lit(offset);
                let getter_stmt = if is_entity {
                    quote!(self.0.slice(#start, #end))
                } else {
                    quote!(&self.as_slice()[#start..#end])
                };
                let getter = quote!(
                    pub fn #func(&self) -> #getter_ret {
                        #inner::new_unchecked(#getter_stmt)
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
        let (inner, getter_ret, getter_stmt, getter_ret_atom, getter_stmt_atom) = if is_entity {
            let inner = entity_name(self.typ.name());
            let getter_ret = quote!(#inner);
            let getter_stmt = quote!(self.0.slice(start, end));
            let getter_ret_atom = quote!(molecule::bytes::Bytes);
            let getter_stmt_atom = quote!(self.0.slice_from(molecule::NUMBER_SIZE));
            (
                inner,
                getter_ret,
                getter_stmt,
                getter_ret_atom,
                getter_stmt_atom,
            )
        } else {
            let inner = reader_name(self.typ.name());
            let getter_ret = quote!(#inner<'r>);
            let getter_stmt = quote!(&self.as_slice()[start..end]);
            let getter_ret_atom = quote!(&'r [u8]);
            let getter_stmt_atom = quote!(&self.as_slice()[molecule::NUMBER_SIZE..]);
            (
                inner,
                getter_ret,
                getter_stmt,
                getter_ret_atom,
                getter_stmt_atom,
            )
        };
        let common_part = quote!(
            pub fn get(&self, idx: usize) -> Option<#getter_ret> {
                if idx >= self.len() {
                    None
                } else {
                    Some(self.get_unchecked(idx))
                }
            }
            pub fn get_unchecked(&self, idx: usize) -> #getter_ret {
                let start = molecule::NUMBER_SIZE + Self::ITEM_SIZE * idx;
                let end = start + Self::ITEM_SIZE;
                #inner::new_unchecked(#getter_stmt)
            }
        );
        if self.typ.is_atom() {
            quote!(
                #common_part
                pub fn raw_data(&self) -> #getter_ret_atom {
                    #getter_stmt_atom
                }
            )
        } else {
            common_part
        }
    }
}

impl ImplGetters for ast::DynVec {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let (inner, getter_ret, getter_stmt_last, getter_stmt) = if is_entity {
            let inner = entity_name(self.typ.name());
            let getter_ret = quote!(#inner);
            let getter_stmt_last = quote!(self.0.slice_from(start));
            let getter_stmt = quote!(self.0.slice(start, end));
            (inner, getter_ret, getter_stmt_last, getter_stmt)
        } else {
            let inner = reader_name(self.typ.name());
            let getter_ret = quote!(#inner<'r>);
            let getter_stmt_last = quote!(&self.as_slice()[start..]);
            let getter_stmt = quote!(&self.as_slice()[start..end]);
            (inner, getter_ret, getter_stmt_last, getter_stmt)
        };
        quote!(
            pub fn get(&self, idx: usize) -> Option<#getter_ret> {
                if idx >= self.len() {
                    None
                } else {
                    Some(self.get_unchecked(idx))
                }
            }
            pub fn get_unchecked(&self, idx: usize) -> #getter_ret {
                let offsets = self.item_offsets();
                let start = molecule::unpack_number(&offsets[idx][..]) as usize;
                if idx == self.len() - 1 {
                    #inner::new_unchecked(#getter_stmt_last)
                } else {
                    let end = molecule::unpack_number(&offsets[idx+1][..]) as usize;
                    #inner::new_unchecked(#getter_stmt)
                }
            }
        )
    }
}

impl ImplGetters for ast::Table {
    fn impl_getters_internal(&self, is_entity: bool) -> m4::TokenStream {
        let (getter_stmt_last, getter_stmt) = if is_entity {
            let getter_stmt_last = quote!(self.0.slice_from(start));
            let getter_stmt = quote!(self.0.slice(start, end));
            (getter_stmt_last, getter_stmt)
        } else {
            let getter_stmt_last = quote!(&self.as_slice()[start..]);
            let getter_stmt = quote!(&self.as_slice()[start..end]);
            (getter_stmt_last, getter_stmt)
        };
        let each_getter = self
            .inner
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let func = func_name(&f.name);
                let (inner, getter_ret) = if is_entity {
                    let inner = entity_name(f.typ.name());
                    let getter_ret = quote!(#inner);
                    (inner, getter_ret)
                } else {
                    let inner = reader_name(f.typ.name());
                    let getter_ret = quote!(#inner<'r>);
                    (inner, getter_ret)
                };
                let start = usize_lit(i);
                let end = usize_lit(i + 1);
                if i == self.inner.len() - 1 {
                    quote!(
                        pub fn #func(&self) -> #getter_ret {
                            let offsets = self.field_offsets();
                            let start = molecule::unpack_number(&offsets[#start][..]) as usize;
                            if self.has_extra_fields() {
                                let end = molecule::unpack_number(&offsets[#end][..]) as usize;
                                #inner::new_unchecked(#getter_stmt)
                            } else {
                                #inner::new_unchecked(#getter_stmt_last)
                            }
                        }
                    )
                } else {
                    quote!(
                        pub fn #func(&self) -> #getter_ret {
                            let offsets = self.field_offsets();
                            let start = molecule::unpack_number(&offsets[#start][..]) as usize;
                            let end = molecule::unpack_number(&offsets[#end][..]) as usize;
                            #inner::new_unchecked(#getter_stmt)
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
