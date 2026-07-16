#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use lirpc::{
    api_spec::{ApiSpec, LiRpcMethodSpec},
    codegen::CodeGen,
    translatable::Type,
    type_definition::{
        EnumDefinition, EnumVariantFields, StructDefinition, StructFields, TypeDefinition,
    },
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct RustCodeGen;

impl CodeGen for RustCodeGen {
    fn generate_package(spec: &ApiSpec) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("Cargo.toml".to_string(), Self::generate_cargo_toml(spec)),
            ("src/lib.rs".to_string(), Self::generate_rust_code(spec)),
        ])
    }
}

impl RustCodeGen {
    fn generate_cargo_toml(spec: &ApiSpec) -> String {
        format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2024\"\n\n[dependencies]\nlirpc_client = {{}}\nserde = {{ version = \"1.0.228\", features = [\"derive\"] }}\n",
            spec.name, spec.version,
        )
    }

    fn generate_rust_code(spec: &ApiSpec) -> String {
        let imports = Self::pretty_print(quote! {
            use lirpc_client::{Client, transport::Transport};
            use serde::{Deserialize, Serialize};
        });

        let mut type_names: Vec<&String> = spec.types.keys().collect();
        type_names.sort();
        let types = type_names
            .into_iter()
            .map(|name| Self::pretty_print(Self::type_definition_to_tokens(&spec.types[name])));

        let mut method_names: Vec<&String> = spec.methods.keys().collect();
        method_names.sort();
        let methods = method_names
            .into_iter()
            .map(|name| Self::pretty_print(Self::method_to_tokens(name, &spec.methods[name])));

        // Sections are pretty-printed independently and joined by a blank line,
        // since blank lines don't survive the quote!/syn token-stream round trip.
        std::iter::once(imports)
            .chain(types)
            .chain(methods)
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn pretty_print(tokens: TokenStream) -> String {
        let file: syn::File =
            syn::parse2(tokens).expect("codegen should always produce valid rust code");
        prettyplease::unparse(&file)
    }

    fn generics_tokens(generics: &[String]) -> TokenStream {
        if generics.is_empty() {
            return TokenStream::new();
        }

        let idents = generics.iter().map(|generic| format_ident!("{generic}"));
        quote! { <#(#idents),*> }
    }

    fn type_to_tokens(ty: &Type) -> TokenStream {
        match ty {
            Type::TypeRef(name) | Type::Generic(name) => {
                let ident = format_ident!("{name}");
                quote! { #ident }
            }
            Type::Box(inner) => {
                let inner = Self::type_to_tokens(inner);
                quote! { Box<#inner> }
            }
            Type::Vec(inner) => {
                let inner = Self::type_to_tokens(inner);
                quote! { Vec<#inner> }
            }
            Type::Result(ok, err) => {
                let ok = Self::type_to_tokens(ok);
                let err = Self::type_to_tokens(err);
                quote! { Result<#ok, #err> }
            }
            Type::Option(inner) => {
                let inner = Self::type_to_tokens(inner);
                quote! { Option<#inner> }
            }
            Type::HashMap(key, value) => {
                let key = Self::type_to_tokens(key);
                let value = Self::type_to_tokens(value);
                quote! { std::collections::HashMap<#key, #value> }
            }
            Type::Unit => quote! { () },
            Type::String => quote! { String },
            Type::Bool => quote! { bool },
            Type::I8 => quote! { i8 },
            Type::I16 => quote! { i16 },
            Type::I32 => quote! { i32 },
            Type::I64 => quote! { i64 },
            Type::I128 => quote! { i128 },
            Type::U8 => quote! { u8 },
            Type::U16 => quote! { u16 },
            Type::U32 => quote! { u32 },
            Type::U64 => quote! { u64 },
            Type::U128 => quote! { u128 },
        }
    }

    fn type_definition_to_tokens(def: &TypeDefinition) -> TokenStream {
        match def {
            TypeDefinition::Struct(strct) => Self::struct_definition_to_tokens(strct),
            TypeDefinition::Enum(enm) => Self::enum_definition_to_tokens(enm),
        }
    }

    fn struct_definition_to_tokens(def: &StructDefinition) -> TokenStream {
        let ident = format_ident!("{}", def.ident);
        let generics = Self::generics_tokens(&def.generics);

        match &def.fields {
            StructFields::Named(fields) => {
                let fields = fields.iter().map(|(name, ty)| {
                    let field_ident = format_ident!("{name}");
                    let ty = Self::type_to_tokens(ty);
                    quote! { pub #field_ident: #ty }
                });

                quote! {
                    #[derive(Debug, Clone, Serialize, Deserialize)]
                    pub struct #ident #generics {
                        #(#fields),*
                    }
                }
            }
            StructFields::Unnamed(types) => {
                let types = types.iter().map(Self::type_to_tokens);

                quote! {
                    #[derive(Debug, Clone, Serialize, Deserialize)]
                    pub struct #ident #generics(#(pub #types),*);
                }
            }
        }
    }

    fn enum_definition_to_tokens(def: &EnumDefinition) -> TokenStream {
        let ident = format_ident!("{}", def.ident);
        let generics = Self::generics_tokens(&def.generics);

        let variants = def.variants.iter().map(|variant| {
            let variant_ident = format_ident!("{}", variant.ident);

            match &variant.fields {
                EnumVariantFields::Named(fields) if fields.is_empty() => quote! { #variant_ident },
                EnumVariantFields::Named(fields) => {
                    let fields = fields.iter().map(|(name, ty)| {
                        let field_ident = format_ident!("{name}");
                        let ty = Self::type_to_tokens(ty);
                        quote! { #field_ident: #ty }
                    });

                    quote! { #variant_ident { #(#fields),* } }
                }
                EnumVariantFields::Unnamed(types) if types.is_empty() => quote! { #variant_ident },
                EnumVariantFields::Unnamed(types) => {
                    let types = types.iter().map(Self::type_to_tokens);
                    quote! { #variant_ident(#(#types),*) }
                }
            }
        });

        quote! {
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub enum #ident #generics {
                #(#variants),*
            }
        }
    }

    fn method_to_tokens(name: &str, spec: &LiRpcMethodSpec) -> TokenStream {
        let fn_ident = format_ident!("{name}");
        let return_type = Self::type_to_tokens(&spec.returns);

        match spec.messages.as_slice() {
            [] => quote! {
                pub async fn #fn_ident<T, F>(
                    client: &mut Client<T, F>,
                ) -> Result<#return_type, lirpc_client::error::Error>
                where
                    T: Transport<F>,
                {
                    client
                        .call::<(), #return_type>(#name.to_string(), None)
                        .await?
                        .resolve()
                        .await
                }
            },
            [message] => {
                let request_type = Self::type_to_tokens(message);

                quote! {
                    pub async fn #fn_ident<T, F>(
                        client: &mut Client<T, F>,
                        request: #request_type,
                    ) -> Result<#return_type, lirpc_client::error::Error>
                    where
                        T: Transport<F>,
                    {
                        client
                            .call::<#request_type, #return_type>(#name.to_string(), Some(request))
                            .await?
                            .resolve()
                            .await
                    }
                }
            }
            messages => panic!(
                "method '{name}' has {} messages, but codegen only supports methods with 0 or 1 messages",
                messages.len()
            ),
        }
    }
}
