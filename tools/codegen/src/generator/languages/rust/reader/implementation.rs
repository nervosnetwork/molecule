use proc_macro2 as m4;
use quote::quote;

use super::super::utilities::{entity_name, reader_name, reader_verify, usize_lit};
use crate::ast::{self as ast, HasName};

pub(in super::super) trait ImplReader: HasName {
    fn impl_reader_internal(&self) -> m4::TokenStream;

    fn impl_reader(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        let reader = reader_name(self.name());
        let reader_string = reader.to_string();
        let internal = self.impl_reader_internal();
        quote!(
            impl<'r> molecule::prelude::Reader<'r> for #reader<'r> {
                type Entity = #entity;
                const NAME: &'static str = #reader_string;
                fn to_entity(&self) -> Self::Entity {
                    <Self::Entity as molecule::prelude::Entity>::new_unchecked(molecule::prelude::Vec::from(self.0).into())
                }
                fn new_unchecked(slice: &'r [u8]) -> Self {
                    #reader(slice)
                }
                fn as_slice(&self) -> &'r [u8] {
                    self.0
                }
                #internal
            }
        )
    }
}

impl ImplReader for ast::Option_ {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        let verify_inner = reader_verify(self.item().typ(), quote!(&slice[..]));
        quote!(
            fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                if !slice.is_empty() {
                    #verify_inner?;
                }
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplReader for ast::Union {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        let verify_inners = self.items().iter().map(|item| {
            let item_id = usize_lit(item.id());
            let verify_inner = reader_verify(item.typ(), quote!(inner_slice));
            quote!(
                #item_id => #verify_inner,
            )
        });
        quote!(
            fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::verification_error as ve;
                let slice_len = slice.len();
                if slice_len < molecule::NUMBER_SIZE {
                    return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
                }
                let item_id = molecule::unpack_number(slice);
                let inner_slice = &slice[molecule::NUMBER_SIZE..];
                match item_id {
                    #( #verify_inners )*
                    _ => ve!(Self, UnknownItem, Self::ITEMS_COUNT, item_id),
                }?;
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplReader for ast::Array {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::verification_error as ve;
                let slice_len = slice.len();
                if slice_len != Self::TOTAL_SIZE {
                    return ve!(Self, TotalSizeNotMatch, Self::TOTAL_SIZE, slice_len);
                }
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplReader for ast::Struct {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::verification_error as ve;
                let slice_len = slice.len();
                if slice_len != Self::TOTAL_SIZE {
                    return ve!(Self, TotalSizeNotMatch, Self::TOTAL_SIZE, slice_len);
                }
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplReader for ast::FixVec {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        quote!(
            fn verify(slice: &[u8], _compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::verification_error as ve;
                let slice_len = slice.len();
                if slice_len < molecule::NUMBER_SIZE {
                    return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
                }
                let item_count = molecule::unpack_number(slice) as usize;
                // Saturation cannot match a real slice length.
                let total_size = Self::ITEM_SIZE
                    .saturating_mul(item_count)
                    .saturating_add(molecule::NUMBER_SIZE);
                if slice_len != total_size {
                    return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
                }
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplReader for ast::DynVec {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        let verify_inner = reader_verify(self.item().typ(), quote!(&slice[start..end]));
        quote!(
            fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                use molecule::verification_error as ve;
                let slice_len = slice.len();
                if slice_len < molecule::NUMBER_SIZE {
                    return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
                }
                let total_size = molecule::unpack_number(slice) as usize;
                if slice_len != total_size {
                    return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
                }
                if slice_len == molecule::NUMBER_SIZE {
                    return ::core::result::Result::Ok(());
                }
                if slice_len < molecule::NUMBER_SIZE * 2 {
                    return ve!(Self, TotalSizeNotMatch, molecule::NUMBER_SIZE * 2, slice_len);
                }
                let offset_first = molecule::unpack_number(&slice[molecule::NUMBER_SIZE..]) as usize;
                if offset_first % molecule::NUMBER_SIZE != 0 || offset_first < molecule::NUMBER_SIZE * 2 {
                    return ve!(Self, OffsetsNotMatch);
                }
                if slice_len < offset_first {
                    return ve!(Self, HeaderIsBroken, offset_first, slice_len);
                }
                let mut offsets: molecule::prelude::Vec<usize> = slice[molecule::NUMBER_SIZE..offset_first]
                    .chunks_exact(molecule::NUMBER_SIZE)
                    .map(|x| molecule::unpack_number(x) as usize)
                    .collect();
                offsets.push(total_size);
                if offsets.windows(2).any(|i| i[0] > i[1]) {
                    return ve!(Self, OffsetsNotMatch);
                }
                for pair in offsets.windows(2) {
                    let start = pair[0];
                    let end =  pair[1];
                    #verify_inner?;
                }
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplReader for ast::Table {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        if self.fields().is_empty() {
            quote!(
                fn verify(
                    slice: &[u8],
                    compatible: bool,
                ) -> molecule::error::VerificationResult<()> {
                    use molecule::verification_error as ve;
                    let slice_len = slice.len();
                    if slice_len < molecule::NUMBER_SIZE {
                        return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
                    }
                    let total_size = molecule::unpack_number(slice) as usize;
                    if slice_len != total_size {
                        return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
                    }
                    if slice_len > molecule::NUMBER_SIZE && !compatible {
                        return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, !0);
                    }
                    ::core::result::Result::Ok(())
                }
            )
        } else {
            let verify_fields = self.fields().iter().enumerate().map(|(i, f)| {
                let start = usize_lit(i);
                let end = usize_lit(i + 1);
                let verify_field =
                    reader_verify(f.typ(), quote!(&slice[offsets[#start]..offsets[#end]]));
                quote!(
                    #verify_field?;
                )
            });
            quote!(
                fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                    use molecule::verification_error as ve;
                    let slice_len = slice.len();
                    if slice_len < molecule::NUMBER_SIZE {
                        return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE, slice_len);
                    }
                    let total_size = molecule::unpack_number(slice) as usize;
                    if slice_len != total_size {
                        return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
                    }
                    if slice_len < molecule::NUMBER_SIZE * 2 {
                        return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE * 2, slice_len);
                    }
                    let offset_first = molecule::unpack_number(&slice[molecule::NUMBER_SIZE..]) as usize;
                    if offset_first % molecule::NUMBER_SIZE != 0 || offset_first < molecule::NUMBER_SIZE * 2 {
                        return ve!(Self, OffsetsNotMatch);
                    }
                    if slice_len < offset_first {
                        return ve!(Self, HeaderIsBroken, offset_first, slice_len);
                    }
                    let field_count = offset_first / molecule::NUMBER_SIZE - 1;
                    if field_count < Self::FIELD_COUNT {
                        return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
                    } else if !compatible && field_count > Self::FIELD_COUNT {
                        return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
                    };
                    let mut offsets: molecule::prelude::Vec<usize> = slice[molecule::NUMBER_SIZE..offset_first]
                        .chunks_exact(molecule::NUMBER_SIZE)
                        .map(|x| molecule::unpack_number(x) as usize)
                        .collect();
                    offsets.push(total_size);
                    if offsets.windows(2).any(|i| i[0] > i[1]) {
                        return ve!(Self, OffsetsNotMatch);
                    }
                    #( #verify_fields )*
                    ::core::result::Result::Ok(())
                }
            )
        }
    }
}
