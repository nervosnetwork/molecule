use proc_macro2 as m4;
use quote::quote;

use super::super::utilities::{entity_name, reader_name, usize_lit};
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
                    Self::Entity::new_unchecked(self.as_slice().to_owned().into())
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
        let inner = reader_name(self.item().typ().name());
        quote!(
            fn verify(slice: &[u8], compatible: bool) -> molecule::error::VerificationResult<()> {
                if !slice.is_empty() {
                    #inner::verify(&slice[..], compatible)?;
                }
                Ok(())
            }
        )
    }
}

impl ImplReader for ast::Union {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        let verify_inners = self.items().iter().enumerate().map(|(index, inner)| {
            let item_id = usize_lit(index);
            let inner = reader_name(inner.typ().name());
            quote!(
                #item_id => #inner::verify(inner_slice, compatible),
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
                Ok(())
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
                Ok(())
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
                Ok(())
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
                if item_count == 0 {
                    if slice_len != molecule::NUMBER_SIZE {
                        return ve!(Self, TotalSizeNotMatch, molecule::NUMBER_SIZE, slice_len);
                    }
                    return Ok(());
                }
                let total_size = molecule::NUMBER_SIZE + Self::ITEM_SIZE * item_count;
                if slice_len != total_size {
                    return ve!(Self, TotalSizeNotMatch, total_size, slice_len);
                }
                Ok(())
            }
        )
    }
}

impl ImplReader for ast::DynVec {
    fn impl_reader_internal(&self) -> m4::TokenStream {
        let inner = reader_name(self.item().typ().name());
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
                    return Ok(());
                }
                if slice_len < molecule::NUMBER_SIZE * 2 {
                    return ve!(Self, TotalSizeNotMatch, molecule::NUMBER_SIZE * 2, slice_len);
                }
                let offset_first = molecule::unpack_number(&slice[molecule::NUMBER_SIZE..]) as usize;
                if offset_first % 4 != 0 || offset_first < molecule::NUMBER_SIZE * 2 {
                    return ve!(Self, OffsetsNotMatch);
                }
                if slice_len < offset_first {
                    return ve!(Self, HeaderIsBroken, offset_first, slice_len);
                }
                let mut offsets: Vec<usize> = slice[molecule::NUMBER_SIZE..offset_first]
                    .chunks(molecule::NUMBER_SIZE)
                    .map(|x| molecule::unpack_number(x) as usize)
                    .collect();
                offsets.push(total_size);
                for pair in offsets.windows(2) {
                    let start = pair[0];
                    let end =  pair[1];
                    if start > end {
                        return ve!(Self, OffsetsNotMatch);
                    } else {
                        #inner::verify(&slice[start..end], compatible)?;
                    }
                }
                Ok(())
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
                    Ok(())
                }
            )
        } else {
            let verify_fields = self.fields().iter().enumerate().map(|(i, f)| {
                let field = reader_name(f.typ().name());
                let start = usize_lit(i);
                let end = usize_lit(i + 1);
                quote!(
                    #field::verify(&slice[offsets[#start]..offsets[#end]], compatible)?;
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
                    if slice_len == molecule::NUMBER_SIZE && Self::FIELD_COUNT == 0 {
                        return Ok(());
                    }
                    if slice_len < molecule::NUMBER_SIZE * 2 {
                        return ve!(Self, HeaderIsBroken, molecule::NUMBER_SIZE * 2, slice_len);
                    }
                    let offset_first = molecule::unpack_number(&slice[molecule::NUMBER_SIZE..]) as usize;
                    if offset_first % 4 != 0 || offset_first < molecule::NUMBER_SIZE * 2 {
                        return ve!(Self, OffsetsNotMatch);
                    }
                    if slice_len < offset_first {
                        return ve!(Self, HeaderIsBroken, offset_first, slice_len);
                    }
                    let field_count = offset_first / 4 - 1;
                    if field_count < Self::FIELD_COUNT {
                        return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
                    } else if !compatible && field_count > Self::FIELD_COUNT {
                        return ve!(Self, FieldCountNotMatch, Self::FIELD_COUNT, field_count);
                    };
                    let mut offsets: Vec<usize> = slice[molecule::NUMBER_SIZE..offset_first]
                        .chunks(molecule::NUMBER_SIZE)
                        .map(|x| molecule::unpack_number(x) as usize)
                        .collect();
                    offsets.push(total_size);
                    if offsets.windows(2).any(|i| i[0] > i[1]) {
                        return ve!(Self, OffsetsNotMatch);
                    }
                    #( #verify_fields )*
                    Ok(())
                }
            )
        }
    }
}
