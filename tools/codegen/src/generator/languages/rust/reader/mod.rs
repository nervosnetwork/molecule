use proc_macro2 as m4;
use quote::quote;

use super::utilities::reader_name;
use crate::ast::verified::HasName;

mod implementation;

pub(super) trait GenReader {
    fn gen_reader(&self) -> m4::TokenStream;
}

impl<T> GenReader for T
where
    T: HasName
        + super::display::ImplDisplay
        + super::constants::DefConstants
        + super::properties::DefProperties
        + super::getters::ImplGetters
        + implementation::ImplReader,
{
    fn gen_reader(&self) -> m4::TokenStream {
        let reader = reader_name(self.name());
        let display_stmts = self.impl_display();
        let constants = self.def_constants();
        let properties = self.def_properties();
        let getters = self.impl_getters_for_reader();
        let implementation = self.impl_reader();
        quote!(
            #[derive(Clone, Copy)]
            pub struct #reader<'r>(&'r [u8]);

            impl<'r> ::core::fmt::LowerHex for #reader<'r> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    use molecule::faster_hex::hex_string;
                    if f.alternate() {
                        write!(f, "0x")?;
                    }
                    write!(f, "{}", hex_string(self.as_slice()).unwrap())
                }
            }

            impl<'r> ::core::fmt::Debug for #reader<'r> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    write!(f, "{}({:#x})", Self::NAME, self)
                }
            }

            impl<'r> ::core::fmt::Display for #reader<'r> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #display_stmts
                }
            }

            impl<'r> #reader<'r> {
                #constants
                #properties
                #getters
            }

            #implementation
        )
    }
}
