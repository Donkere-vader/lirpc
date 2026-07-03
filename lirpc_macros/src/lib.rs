mod derive;

use proc_macro::TokenStream;
use syn::{ItemEnum, ItemStruct};

use crate::{
    derive::derive_for_enum::derive_translatable_for_enum,
    derive::derive_for_struct::derive_translatable_for_struct,
};

#[proc_macro_derive(LiRpcType)]
pub fn derive_translatable(item: TokenStream) -> TokenStream {
    if let Ok(enm) = syn::parse2::<ItemEnum>(item.clone().into()) {
        derive_translatable_for_enum(enm)
    } else if let Ok(enm) = syn::parse2::<ItemStruct>(item.into()) {
        derive_translatable_for_struct(enm)
    } else {
        panic!("Only enums and structs are supported")
    }
}
