use proc_macro2 as m4;
use quote::quote;

use super::super::utilities::{
    builder_name, entity_as_slice, entity_name, field_name, reader_name, usize_lit,
};
use crate::ast::{self as ast, HasName};

pub(in super::super) trait ImplBuilder: HasName {
    fn impl_builder_internal(&self) -> m4::TokenStream;

    fn impl_builder(&self) -> m4::TokenStream {
        let builder = builder_name(self.name());
        let builder_string = builder.to_string();
        let entity = entity_name(self.name());
        let internal = self.impl_builder_internal();
        quote!(
            impl molecule::prelude::Builder for #builder {
                type Entity = #entity;
                const NAME: &'static str = #builder_string;
                #internal
                fn build(&self) -> Self::Entity {
                    let mut inner = molecule::prelude::Vec::with_capacity(
                        <Self as molecule::prelude::Builder>::expected_length(self),
                    );
                    <Self as molecule::prelude::Builder>::write(self, &mut inner)
                        .unwrap_or_else(|_| panic!("{} build should be ok", Self::NAME));
                    <#entity as molecule::prelude::Entity>::new_unchecked(inner.into())
                }
            }
        )
    }
}

impl ast::Option_ {
    pub(crate) fn gen_from(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        let item_name = entity_name(self.item().typ().name());
        quote!(
            impl ::core::convert::From<#item_name> for #entity {
                fn from(value: #item_name) -> Self {
                    <Self as molecule::prelude::Entity>::new_builder()
                        .set(::core::option::Option::Some(value))
                        .build()
                }
            }
        )
    }
}

impl ImplBuilder for ast::Option_ {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let slice = entity_as_slice(self.item().typ(), quote!(*inner));
        quote!(
            fn expected_length(&self) -> usize {
                self.0
                    .as_ref()
                    .map(|ref inner| #slice.len())
                    .unwrap_or(0)
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                self.0
                    .as_ref()
                    .map(|ref inner| writer.write_all(#slice))
                    .unwrap_or(::core::result::Result::Ok(()))
            }
        )
    }
}

impl ast::Union {
    pub(crate) fn gen_from(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        self.items()
            .iter()
            .map(|item| {
                let item_name = entity_name(item.typ().name());
                quote!(
                    impl ::core::convert::From<#item_name> for #entity {
                        fn from(value: #item_name) -> Self {
                            <Self as molecule::prelude::Entity>::new_builder().set(value).build()
                        }
                    }
                )
            })
            .collect()
    }
}

impl ImplBuilder for ast::Union {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE + self.0.as_slice().len()
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                writer.write_all(&molecule::pack_number(self.0.item_id()))?;
                writer.write_all(self.0.as_slice())
            }
        )
    }
}

impl ImplBuilder for ast::Array {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let typ = self.item().typ();
        let write_inners = {
            let idx = (0..self.item_count()).map(usize_lit).collect::<Vec<_>>();
            let slices = idx
                .iter()
                .map(|i| entity_as_slice(typ, quote!(&self.0[#i])))
                .collect::<Vec<_>>();
            quote!(
                #(
                    writer.write_all(#slices)?;
                )*
            )
        };
        quote!(
            fn expected_length(&self) -> usize {
                Self::TOTAL_SIZE
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                #write_inners
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ast::Array {
    pub(crate) fn gen_from(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        let reader = reader_name(self.name());
        let item_name = entity_name(self.item().typ().name());
        let n = self.item_count();
        let maybe_byte_arr = if self.item().typ().name() == "byte" {
            quote!(
                impl ::core::convert::From<[u8; #n]> for #entity {
                    fn from(value: [u8; #n]) -> Self {
                        <#reader<'_> as molecule::prelude::Reader<'_>>::to_entity(
                            &<#reader<'_> as molecule::prelude::Reader<'_>>::new_unchecked(&value),
                        )
                    }
                }

                impl ::core::convert::TryFrom<&[u8]> for #entity {
                    type Error = ::core::array::TryFromSliceError;
                    fn try_from(value: &[u8]) -> ::core::result::Result<Self, ::core::array::TryFromSliceError> {
                        ::core::result::Result::Ok(<[u8; #n]>::try_from(value)?.into())
                    }
                }

                impl ::core::convert::From<#entity> for [u8; #n] {
                    #[track_caller]
                    fn from(value: #entity) -> Self {
                        ::core::convert::TryFrom::try_from(
                            molecule::prelude::Entity::as_slice(&value),
                        )
                        .unwrap()
                    }
                }

                impl<'a> ::core::convert::From<#reader<'a>> for &'a [u8; #n] {
                    #[track_caller]
                    fn from(value: #reader<'a>) -> Self {
                        ::core::convert::TryFrom::try_from(
                            molecule::prelude::Reader::as_slice(&value),
                        )
                        .unwrap()
                    }
                }

                impl<'a> ::core::convert::From<&'a #reader<'a>> for &'a [u8; #n] {
                    #[track_caller]
                    fn from(value: &'a #reader<'a>) -> Self {
                        ::core::convert::TryFrom::try_from(
                            molecule::prelude::Reader::as_slice(value),
                        )
                        .unwrap()
                    }
                }
            )
        } else {
            quote!()
        };
        let nth = (0..n).map(|i| quote::format_ident!("nth{}", i));
        quote!(
            impl ::core::convert::From<[#item_name; #n]> for #entity {
                fn from(value: [#item_name; #n]) -> Self {
                    <Self as molecule::prelude::Entity>::new_builder().set(value).build()
                }
            }

            impl ::core::convert::TryFrom<&[#item_name]> for #entity {
                type Error = ::core::array::TryFromSliceError;
                fn try_from(value: &[#item_name]) -> ::core::result::Result<Self, ::core::array::TryFromSliceError> {
                    // Use TryFrom<&[T]> for &[T; n].
                    ::core::result::Result::Ok(
                        <Self as molecule::prelude::Entity>::new_builder()
                            .set(<&[#item_name; #n]>::try_from(value)?.clone())
                            .build(),
                    )
                }
            }

            impl ::core::convert::From<#entity> for [#item_name; #n] {
                #[track_caller]
                fn from(value: #entity) -> Self {
                    [#(value.#nth(),)*]
                }
            }

            #maybe_byte_arr
        )
    }
}

impl ImplBuilder for ast::Struct {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let fields = self.fields().iter().map(|f| {
            let field_name = field_name(f.name());
            let slice = entity_as_slice(f.typ(), quote!(&self.#field_name));
            quote!(
                writer.write_all(#slice)?;
            )
        });
        quote!(
            fn expected_length(&self) -> usize {
                Self::TOTAL_SIZE
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                #( #fields )*
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplBuilder for ast::FixVec {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let slice = entity_as_slice(self.item().typ(), quote!(inner));
        let write_inners = quote!(for inner in &self.0[..] {
            writer.write_all(#slice)?;
        });
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE + Self::ITEM_SIZE * self.0.len()
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                writer.write_all(&molecule::pack_number(self.0.len() as molecule::Number))?;
                #write_inners
                ::core::result::Result::Ok(())
            }
        )
    }
}

impl ImplBuilder for ast::DynVec {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        let slice = entity_as_slice(self.item().typ(), quote!(inner));
        quote!(
            fn expected_length(&self) -> usize {
                molecule::NUMBER_SIZE * (self.0.len() + 1)
                    + self
                        .0
                        .iter()
                        .map(|inner| #slice.len())
                        .sum::<usize>()
            }
            fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                let item_count = self.0.len();
                if item_count == 0 {
                    writer.write_all(&molecule::pack_number(
                        molecule::NUMBER_SIZE as molecule::Number,
                    ))?;
                } else {
                    let (total_size, offsets) = self.0.iter().fold(
                        (
                            molecule::NUMBER_SIZE * (item_count + 1),
                            molecule::prelude::Vec::with_capacity(item_count),
                        ),
                        |(start, mut offsets), inner| {
                            offsets.push(start);
                            (start + #slice.len(), offsets)
                        },
                    );
                    writer.write_all(&molecule::pack_number(total_size as molecule::Number))?;
                    for offset in offsets.into_iter() {
                        writer.write_all(&molecule::pack_number(offset as molecule::Number))?;
                    }
                    for inner in self.0.iter() {
                        writer.write_all(#slice)?;
                    }
                }
                ::core::result::Result::Ok(())
            }
        )
    }
}

fn gen_from_iter(name: &str, item_name: &str) -> m4::TokenStream {
    let entity = entity_name(name);
    let item_name = entity_name(item_name);
    quote!(
        impl<T> ::core::iter::FromIterator<T> for #entity
        where
            T: ::core::convert::Into<#item_name>,
        {
            fn from_iter<I: ::core::iter::IntoIterator<Item = T>>(iter: I) -> Self {
                <Self as molecule::prelude::Entity>::new_builder()
                    .extend(iter.into_iter().map(::core::convert::Into::into))
                    .build()
            }
        }

        impl<T> ::core::convert::From<molecule::prelude::Vec<T>> for #entity
        where
            T: ::core::convert::Into<#item_name>
        {
            fn from(v: molecule::prelude::Vec<T>) -> Self {
                v.into_iter().collect()
            }
        }
    )
}

impl ast::FixVec {
    pub(crate) fn gen_from_iter(&self) -> m4::TokenStream {
        gen_from_iter(self.name(), self.item().typ().name())
    }
}

impl ast::DynVec {
    pub(crate) fn gen_from_iter(&self) -> m4::TokenStream {
        gen_from_iter(self.name(), self.item().typ().name())
    }
}

impl ImplBuilder for ast::Table {
    fn impl_builder_internal(&self) -> m4::TokenStream {
        if self.fields().is_empty() {
            quote!(
                fn expected_length(&self) -> usize {
                    molecule::NUMBER_SIZE
                }
                fn write<W: molecule::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> molecule::io::Result<()> {
                    writer.write_all(&molecule::pack_number(
                        molecule::NUMBER_SIZE as molecule::Number,
                    ))?;
                    ::core::result::Result::Ok(())
                }
            )
        } else {
            let slices = &self
                .fields()
                .iter()
                .map(|f| {
                    let field_name = field_name(f.name());
                    entity_as_slice(f.typ(), quote!(&self.#field_name))
                })
                .collect::<Vec<_>>();
            quote!(
                fn expected_length(&self) -> usize {
                    molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1)
                        #(+ #slices.len())*
                }
                fn write<W: molecule::io::Write>(&self, writer: &mut W) -> molecule::io::Result<()> {
                    let mut total_size = molecule::NUMBER_SIZE * (Self::FIELD_COUNT + 1);
                    let mut offsets = molecule::prelude::Vec::with_capacity(Self::FIELD_COUNT);
                    #(
                        offsets.push(total_size);
                        total_size += #slices.len();
                    )*
                    writer.write_all(&molecule::pack_number(total_size as molecule::Number))?;
                    for offset in offsets.into_iter() {
                        writer.write_all(&molecule::pack_number(offset as molecule::Number))?;
                    }
                    #(
                        writer.write_all(#slices)?;
                    )*
                    ::core::result::Result::Ok(())
                }
            )
        }
    }
}
