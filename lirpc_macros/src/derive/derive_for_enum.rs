use proc_macro::TokenStream;
use quote::quote;
use syn::ItemEnum;

use crate::derive::util::{generics_with_where_clauses, get_type_of_type};

pub fn derive_translatable_for_enum(item: ItemEnum) -> TokenStream {
    let name = item.ident.clone();
    let name_string = name.to_string();

    let gs = &item.generics;
    let gs_with_clauses = generics_with_where_clauses(&item.generics);

    let generic_names = item
        .generics
        .params
        .iter()
        .filter_map(|g| match g {
            syn::GenericParam::Type(g) => Some(g.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<String>>();

    let generics = generic_names.iter().map(|g| quote! { #g.to_string() });

    let variant_tuples = item.variants.iter().map(|var| {
        let var_ident_string = var.ident.to_string();

        let fields = if var.fields.iter().any(|f| f.ident.is_some()) {
            let fs = var.fields.iter().map(|f| {
                let field_name = f
                    .ident
                    .as_ref()
                    .expect("field name should be guaranteed at this point")
                    .to_string();

                let get_type = get_type_of_type(&generic_names, &f.ty);

                quote! {
                    (#field_name.to_string(), #get_type)
                }
            });

            quote! {
                lirpc::type_definition::EnumVariantFields::Named(
                    std::vec![
                        #(#fs),*
                    ]
                )
            }
        } else {
            let fs = var
                .fields
                .iter()
                .map(|f| get_type_of_type(&generic_names, &f.ty));

            quote! {
                lirpc::type_definition::EnumVariantFields::Unnamed(
                    std::vec![
                        #(#fs),*
                    ]
                )
            }
        };

        quote! {
            lirpc::type_definition::EnumVariant {
                ident: #var_ident_string.to_string(),
                fields: #fields,
            }
        }
    });

    TokenStream::from(quote! {
        impl #gs_with_clauses lirpc::lirpc_type::LiRpcType for #name #gs {
            fn translate() -> lirpc::type_definition::TypeDefinition {
                lirpc::type_definition::TypeDefinition::Enum(std::boxed::Box::new(
                    lirpc::type_definition::EnumDefinition {
                        ident: #name_string.to_string(),
                        variants: std::vec![#(#variant_tuples),*],
                        generics: std::vec![#(#generics),*],
                    }
                ))
            }
        }

        impl #gs_with_clauses lirpc::translatable::Translatable for #name #gs {
            fn get_type() -> lirpc::translatable::Type {
                lirpc::translatable::Type::TypeRef(#name_string.to_string())
            }
        }
    })
}
