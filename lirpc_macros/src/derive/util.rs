use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

// TODO: come up with better name.
pub fn generics_with_where_clauses(generics: &Generics) -> TokenStream {
    let gs = generics
        .params
        .iter()
        .map(|g| quote! {#g: lirpc::translatable::Translatable});

    quote! { <#(#gs),*> }
}

// TODO: come up with better name.
pub fn get_type_of_type(generic_names: &[String], ty: &syn::Type) -> TokenStream {
    match ty {
        syn::Type::Path(path) => {
            if let Some(ident) = path.path.get_ident()
                && generic_names.contains(&ident.to_string())
            {
                let name = ident.to_string();
                quote! { lirpc::translatable::Type::Generic(#name.to_string()) }
            } else {
                quote! {
                    <#ty as lirpc::translatable::Translatable>::get_type()
                }
            }
        }
        ty => quote! {
            <#ty as lirpc::translatable::Translatable>::get_type()
        },
    }
}
