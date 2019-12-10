use proc_macro2 as m4;
use quote::quote;

use super::utilities::{entity_name, reader_name, usize_lit};
use crate::ast::verified::{DefaultContent, HasName};

mod implementation;

pub(super) trait GenEntity {
    fn gen_entity(&self) -> m4::TokenStream;
}

impl<T> GenEntity for T
where
    T: HasName
        + DefaultContent
        + super::display::ImplDisplay
        + super::constants::DefConstants
        + super::properties::DefProperties
        + super::getters::ImplGetters
        + implementation::ImplEntity,
{
    fn gen_entity(&self) -> m4::TokenStream {
        let entity = entity_name(self.name());
        let reader = reader_name(self.name());
        let default_content = self
            .default_content()
            .into_iter()
            .map(|b| usize_lit(b as usize));
        let display_stmts = self.impl_display();
        let constants = self.def_constants();
        let properties = self.def_properties();
        let getters = self.impl_getters_for_entity();
        let implementation = self.impl_entity();
        quote!(
            #[derive(Clone)]
            pub struct #entity(molecule::bytes::Bytes);

            impl ::core::fmt::LowerHex for #entity {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    use molecule::faster_hex::hex_string;
                    if f.alternate() {
                        write!(f, "0x")?;
                    }
                    write!(f, "{}", hex_string(self.as_slice()).unwrap())
                }
            }

            impl ::core::fmt::Debug for #entity {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    write!(f, "{}({:#x})", Self::NAME, self)
                }
            }

            impl ::core::fmt::Display for #entity {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #display_stmts
                }
            }

            impl ::core::default::Default for #entity {
                fn default() -> Self {
                    let v: Vec<u8> = vec![#( #default_content, )*];
                    #entity::new_unchecked(v.into())
                }
            }

            impl #entity {
                #constants
                #properties
                #getters
                pub fn as_reader<'r>(&'r self) -> #reader<'r> {
                    #reader::new_unchecked(self.as_slice())
                }
            }

            #implementation
        )
    }
}
