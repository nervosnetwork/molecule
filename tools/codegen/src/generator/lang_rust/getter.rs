use proc_macro2 as m4;
use quote::quote;

use crate::ast::verified as ast;

use super::utilities::*;

pub(super) fn def_getter_for_option(is_entity: bool, info: &ast::Option_) -> Vec<m4::TokenStream> {
    let (inner, getter_ret, getter_stmt) = if is_entity {
        let inner = entity_name(&info.typ.name);
        let getter_ret = quote!(#inner);
        let getter_stmt = quote!(self.0.clone());
        (inner, getter_ret, getter_stmt)
    } else {
        let inner = reader_name(&info.typ.name);
        let getter_ret = quote!(#inner<'r>);
        let getter_stmt = quote!(self.as_slice());
        (inner, getter_ret, getter_stmt)
    };
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    {
        let code = quote!(
            pub fn is_none(&self) -> bool {
                self.0.is_empty()
            }

            pub fn is_some(&self) -> bool {
                !self.0.is_empty()
            }
        );
        funcs.push(code);
    }
    {
        let code = if info.typ.is_atom() {
            quote!(
                pub fn to_opt(&self) -> Option<#inner> {
                    if self.is_none() {
                        None
                    } else {
                        Some(self.0[0])
                    }
                }
            )
        } else {
            quote!(
                pub fn to_opt(&self) -> Option<#getter_ret> {
                    if self.is_none() {
                        None
                    } else {
                        Some(#inner::new_unchecked(#getter_stmt))
                    }
                }
            )
        };
        funcs.push(code);
    }
    funcs
}

pub(super) fn def_getter_for_union(
    is_entity: bool,
    origin_name: &str,
    info: &ast::Union,
) -> Vec<m4::TokenStream> {
    let (union_name, getter_ret, getter_stmt) = if is_entity {
        let union = entity_union_name(origin_name);
        let union_name = quote!(#union);
        let getter_ret = quote!(#union);
        let getter_stmt = quote!(self.0.slice_from(molecule::ITEM_ID_SIZE));
        (union_name, getter_ret, getter_stmt)
    } else {
        let union = reader_union_name(origin_name);
        let union_name = quote!(#union);
        let getter_ret = quote!(#union<'r>);
        let getter_stmt = quote!(&self.as_slice()[molecule::ITEM_ID_SIZE..]);
        (union_name, getter_ret, getter_stmt)
    };
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    {
        let item_count = usize_lit(info.inner.len());
        let code = quote!(
            pub const ITEM_COUNT: usize = #item_count;

            pub fn item_id(&self) -> molecule::ItemId {
                molecule::extract_item_id(self.as_slice())
            }
        );
        funcs.push(code);
    }
    {
        let match_stmts = info.inner.iter().enumerate().map(|(index, inner)| {
            let item_id = usize_lit(index + 1);
            let inner = if is_entity {
                entity_name(&inner.typ.name)
            } else {
                reader_name(&inner.typ.name)
            };
            quote!(#item_id => #inner::new_unchecked(inner).into(),)
        });
        let code = quote!(
            pub fn to_enum(&self) -> #getter_ret {
                let inner = #getter_stmt;
                match self.item_id() {
                    #( #match_stmts )*
                    0 => #union_name::NotSet,
                    _ => unreachable!(),
                }
            }
        );
        funcs.push(code);
    }
    funcs
}

pub(super) fn def_getter_for_array(is_entity: bool, info: &ast::Array) -> Vec<m4::TokenStream> {
    let (inner, getter_ret, getter_ret_atom, getter_stmt_atom) = if is_entity {
        let inner = entity_name(&info.typ.name);
        let getter_ret = quote!(#inner);
        let getter_ret_atom = quote!(molecule::bytes::Bytes);
        let getter_stmt_atom = quote!(self.as_bytes());
        (inner, getter_ret, getter_ret_atom, getter_stmt_atom)
    } else {
        let inner = reader_name(&info.typ.name);
        let getter_ret = quote!(#inner<'r>);
        let getter_ret_atom = quote!(&'r [u8]);
        let getter_stmt_atom = quote!(self.as_slice());
        (inner, getter_ret, getter_ret_atom, getter_stmt_atom)
    };
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    {
        let total_size = usize_lit(info.item_size * info.item_count);
        let item_size = usize_lit(info.item_size);
        let item_count = usize_lit(info.item_count);
        let code = quote!(
            pub const TOTAL_SIZE: usize = #total_size;
            pub const ITEM_SIZE: usize = #item_size;
            pub const ITEM_COUNT: usize = #item_count;
        );
        funcs.push(code);
    }
    if info.typ.is_atom() {
        let code = quote!(
            pub fn raw_data(&self) -> #getter_ret_atom {
                #getter_stmt_atom
            }
        );
        funcs.push(code);
    }
    for idx in 0..info.item_count {
        let start = usize_lit(idx * info.item_size);
        let func = snake_name(&format!("nth{}", idx));
        let code = if info.typ.is_atom() {
            quote!(
                pub fn #func(&self) -> #inner {
                    self.0[#start]
                }
            )
        } else {
            let end = usize_lit((idx + 1) * info.item_size);
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
        };
        funcs.push(code);
    }
    funcs
}

pub(super) fn def_getter_for_struct(is_entity: bool, info: &ast::Struct) -> Vec<m4::TokenStream> {
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    {
        let total_size = usize_lit(info.field_size.iter().sum());
        let field_count = usize_lit(info.inner.len());
        let fields_size = info.field_size.iter().map(|x| usize_lit(*x as usize));
        let code = quote!(
            pub const TOTAL_SIZE: usize = #total_size;
            pub const FIELD_COUNT: usize = #field_count;
            pub const FIELDS_SIZE: [usize; #field_count]= [ #( #fields_size, )* ];
        );
        funcs.push(code);
    }
    {
        let mut offset = 0;
        for (f, s) in info.inner.iter().zip(info.field_size.iter()) {
            let func = snake_name(&f.name);
            let (inner, getter_ret) = if is_entity {
                let inner = entity_name(&f.typ.name);
                let getter_ret = quote!(#inner);
                (inner, getter_ret)
            } else {
                let inner = reader_name(&f.typ.name);
                let getter_ret = quote!(#inner<'r>);
                (inner, getter_ret)
            };
            let start = usize_lit(offset);
            offset += s;
            let code = if f.typ.is_atom() {
                quote!(
                    pub fn #func(&self) -> #inner {
                        self.0[#start]
                    }
                )
            } else {
                let end = usize_lit(offset);
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
            };
            funcs.push(code);
        }
    }
    funcs
}

pub(super) fn def_getter_for_fix_vec(
    is_entity: bool,
    info: &ast::FixedVector,
) -> Vec<m4::TokenStream> {
    let (inner, getter_ret, getter_stmt, getter_ret_atom, getter_stmt_atom) = if is_entity {
        let inner = entity_name(&info.typ.name);
        let getter_ret = quote!(#inner);
        let getter_stmt = quote!(self.0.slice(start, end));
        let getter_ret_atom = quote!(molecule::bytes::Bytes);
        let getter_stmt_atom = quote!(self.0.slice_from(4));
        (
            inner,
            getter_ret,
            getter_stmt,
            getter_ret_atom,
            getter_stmt_atom,
        )
    } else {
        let inner = reader_name(&info.typ.name);
        let getter_ret = quote!(#inner<'r>);
        let getter_stmt = quote!(&self.as_slice()[start..end]);
        let getter_ret_atom = quote!(&'r [u8]);
        let getter_stmt_atom = quote!(&self.as_slice()[4..]);
        (
            inner,
            getter_ret,
            getter_stmt,
            getter_ret_atom,
            getter_stmt_atom,
        )
    };
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    let item_size = usize_lit(info.item_size);
    {
        let code = quote!(
            pub const ITEM_SIZE: usize = #item_size;

            pub fn len(&self) -> usize {
                let le = self.as_slice().as_ptr() as *const u32;
                u32::from_le(unsafe { *le }) as usize
            }
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
        );
        funcs.push(code);
    }
    {
        let item_size = usize_lit(info.item_size);
        let code = if info.typ.is_atom() {
            quote!(
                pub fn raw_data(&self) -> #getter_ret_atom {
                    #getter_stmt_atom
                }
                pub fn get(&self, idx: usize) -> Option<#inner> {
                    if idx >= self.len() {
                        None
                    } else {
                        Some(self.get_unchecked(idx))
                    }
                }
                pub fn get_unchecked(&self, idx: usize) -> #inner {
                    self.0[4 + idx]
                }
            )
        } else {
            quote!(
                pub fn get(&self, idx: usize) -> Option<#getter_ret> {
                    if idx >= self.len() {
                        None
                    } else {
                        Some(self.get_unchecked(idx))
                    }
                }
                pub fn get_unchecked(&self, idx: usize) -> #getter_ret {
                    let start = 4 + idx * #item_size;
                    let end = start + #item_size;
                    #inner::new_unchecked(#getter_stmt)
                }
            )
        };
        funcs.push(code);
    }
    funcs
}

pub(super) fn def_getter_for_dyn_vec(
    is_entity: bool,
    info: &ast::DynamicVector,
) -> Vec<m4::TokenStream> {
    let (inner, getter_ret, getter_stmt_last, getter_stmt) = if is_entity {
        let inner = entity_name(&info.typ.name);
        let getter_ret = quote!(#inner);
        let getter_stmt_last = quote!(self.0.slice_from(start));
        let getter_stmt = quote!(self.0.slice(start, end));
        (inner, getter_ret, getter_stmt_last, getter_stmt)
    } else {
        let inner = reader_name(&info.typ.name);
        let getter_ret = quote!(#inner<'r>);
        let getter_stmt_last = quote!(&self.as_slice()[start..]);
        let getter_stmt = quote!(&self.as_slice()[start..end]);
        (inner, getter_ret, getter_stmt_last, getter_stmt)
    };
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    {
        let code = quote!(
            pub fn offsets(&self) -> (usize, &[u32]) {
                let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                let bytes_len = u32::from_le(ptr[0]) as usize;
                (bytes_len, &ptr[1..])
            }
        );
        funcs.push(code);
    }
    {
        let code = quote!(
            pub fn len(&self) -> usize {
                let (bytes_len, offsets) = self.offsets();
                if bytes_len == 4 {
                    0
                } else {
                    let first = u32::from_le(offsets[0]) as usize;
                    (first - 4) / 4
                }
            }
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
        );
        funcs.push(code);
    }
    {
        let code = quote!(
            pub fn get(&self, idx: usize) -> Option<#getter_ret> {
                let len = self.len();
                if idx >= len {
                    None
                } else {
                    Some(self.get_unchecked(idx))
                }
            }
            pub fn get_unchecked(&self, idx: usize) -> #getter_ret {
                let len = self.len();
                let (_, offsets) = self.offsets();
                let start = u32::from_le(offsets[idx]) as usize;
                if idx == len - 1 {
                    #inner::new_unchecked(#getter_stmt_last)
                } else {
                    let end = u32::from_le(offsets[idx+1]) as usize;
                    #inner::new_unchecked(#getter_stmt)
                }
            }
        );
        funcs.push(code);
    }
    funcs
}

pub(super) fn def_getter_for_table(is_entity: bool, info: &ast::Table) -> Vec<m4::TokenStream> {
    let (getter_stmt_last, getter_stmt) = if is_entity {
        let getter_stmt_last = quote!(self.0.slice_from(start));
        let getter_stmt = quote!(self.0.slice(start, end));
        (getter_stmt_last, getter_stmt)
    } else {
        let getter_stmt_last = quote!(&self.as_slice()[start..]);
        let getter_stmt = quote!(&self.as_slice()[start..end]);
        (getter_stmt_last, getter_stmt)
    };
    let mut funcs: Vec<m4::TokenStream> = Vec::new();
    {
        let field_count = usize_lit(info.inner.len());
        let code = quote!(pub const FIELD_COUNT: usize = #field_count;);
        funcs.push(code);
    }
    {
        let code = quote!(
            pub fn field_offsets(&self) -> (usize, usize, &[u32]) {
                let ptr: &[u32] = unsafe { ::std::mem::transmute(self.as_slice()) };
                let bytes_len = u32::from_le(ptr[0]) as usize;
                let first = u32::from_le(ptr[1]) as usize;
                let count = (first - 4) / 4;
                (bytes_len, count, &ptr[1..])
            }
            pub fn has_extra_fields(&self) -> bool {
                let (_, real_fields_count, _) = Self::field_offsets(self);
                Self::FIELD_COUNT == real_fields_count
            }
        );
        funcs.push(code);
    }
    {
        let field_count = usize_lit(info.inner.len());
        for (i, f) in info.inner.iter().enumerate() {
            let func = snake_name(&f.name);
            let (inner, getter_ret) = if is_entity {
                let inner = entity_name(&f.typ.name);
                let getter_ret = quote!(#inner);
                (inner, getter_ret)
            } else {
                let inner = reader_name(&f.typ.name);
                let getter_ret = quote!(#inner<'r>);
                (inner, getter_ret)
            };
            let start = usize_lit(i);
            let code = if f.typ.is_atom() {
                quote!(
                    pub fn #func(&self) -> #inner {
                        let (_, _, offsets) = Self::field_offsets(self);
                        let offset = u32::from_le(offsets[#start]) as usize;
                        self.0[offset]
                    }
                )
            } else if i == info.inner.len() - 1 {
                quote!(
                    pub fn #func(&self) -> #getter_ret {
                        let (_, count, offsets) = Self::field_offsets(self);
                        let start = u32::from_le(offsets[#start]) as usize;
                        if count == #field_count {
                            #inner::new_unchecked(#getter_stmt_last)
                        } else {
                            let end = u32::from_le(offsets[#start+1]) as usize;
                            #inner::new_unchecked(#getter_stmt)
                        }
                    }
                )
            } else {
                quote!(
                    pub fn #func(&self) -> #getter_ret {
                        let (_, _, offsets) = Self::field_offsets(self);
                        let start = u32::from_le(offsets[#start]) as usize;
                        let end = u32::from_le(offsets[#start+1]) as usize;
                        #inner::new_unchecked(#getter_stmt)
                    }
                )
            };
            funcs.push(code);
        }
    }
    funcs
}
