use std::{collections::BTreeMap, env, fs, path::PathBuf};

use lirpc::contracts::{
    contract_file::LiRpcType,
    lirpc_method_file::{LiRpcMethodFile, LiRpcMethodReturn},
    lirpc_type_file::LiRpcTypeFile,
    serializable_type::SerializableType,
};
use proc_macro::TokenStream;
use quote::quote;
use serde::Serialize;
use syn::{ItemEnum, ItemFn, ItemStruct, ReturnType, Variant, parse_macro_input};

#[proc_macro_attribute]
pub fn lirpc_type(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    // let input = parse_macro_input!(item as ItemStruct);

    if let Ok(input) = syn::parse::<ItemStruct>(item.clone()) {
        lirpc_type_from_struct(input)
    } else if let Ok(input) = syn::parse::<ItemEnum>(item) {
        lirpc_type_from_enum(input)
    } else {
        panic!("expected struct or enum");
    }
}

fn lirpc_type_from_enum(input: ItemEnum) -> TokenStream {
    let ItemEnum {
        variants: enum_variants,
        ident: enum_ident,
        ..
    } = input.clone();

    let mut variants = BTreeMap::new();

    for var in enum_variants.into_iter() {
        let Variant {
            ident: variant_ident,
            fields: enum_fields,
            ..
        } = var;

        let fields: BTreeMap<String, SerializableType> = enum_fields.iter()
            .map(|f| (
                    f.ident.as_ref().expect("Enum annoted with lirpc_type cannot have variants without identifiers (names) for its fields").to_string(),
                    SerializableType::try_from(f.ty.clone()).expect("Unsupported type")
                )
            )
            .collect();

        variants.insert(variant_ident.to_string(), fields);
    }

    let enum_type = LiRpcType::Enum { variants };

    let lirpc_type_file = LiRpcTypeFile {
        name: enum_ident.to_string(),
        r#type: enum_type,
    };

    persist(&format!("type-{}.json", enum_ident), &lirpc_type_file)
        .expect("Error storing type of lirpc_type");

    TokenStream::from(quote! { #input })
}

fn lirpc_type_from_struct(input: ItemStruct) -> TokenStream {
    let ItemStruct {
        ident: struct_ident,
        fields: struct_fields,
        ..
    } = input.clone();

    let fields = struct_fields
        .into_iter()
        .map(|f| (
                f.ident
                    .expect("Struct annoted with lirpc_type must have identifiers (names) for its fields")
                    .to_string(),
                SerializableType::try_from(f.ty).expect("Unsupported time cannot be used"),
            )
        )
        .collect::<BTreeMap<String, SerializableType>>();

    let lirpc_type = LiRpcTypeFile {
        name: struct_ident.to_string(),
        r#type: LiRpcType::Struct { fields },
    };

    persist(&format!("type-{}.json", struct_ident), &lirpc_type)
        .expect("Error storing type of lirpc_type");

    TokenStream::from(quote! { #input })
}

#[proc_macro_attribute]
pub fn lirpc_method(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        sig: method_signature,
        ..
    } = input.clone();

    let mut output: Option<SerializableType> = None;
    let mut message: Option<SerializableType> = None;

    for arg in &method_signature.inputs {
        if let syn::FnArg::Typed(pat) = arg
            && let syn::Type::Path(syn::TypePath { path, .. }) = &*pat.ty
            && let Some(last) = path.segments.last()
        {
            let ident = last.ident.to_string();
            if matches!(ident.as_str(), "Output" | "OutputStream" | "Message")
                && let syn::PathArguments::AngleBracketed(abga) = &last.arguments
            {
                let inner_ty_opt = abga.args.iter().find_map(|ga| {
                    if let syn::GenericArgument::Type(t) = ga {
                        Some(t.clone())
                    } else {
                        None
                    }
                });
                if let Some(inner_ty) = inner_ty_opt {
                    let st = SerializableType::try_from(inner_ty).unwrap_or_else(|err| {
                        panic!("Unsupported inner type for {}: {}", ident, err)
                    });
                    match ident.as_str() {
                        "Output" | "OutputStream" => {
                            if output.is_some() {
                                panic!(
                                    "Multiple output parameters found (Output/OutputStream are exclusive)"
                                );
                            }
                            output = Some(st);
                        }
                        "Message" => {
                            if message.is_some() {
                                panic!(
                                    "Multiple message parameters found (only one Message<T> allowed)"
                                );
                            }
                            message = Some(st);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let return_type = match method_signature.output {
        ReturnType::Default => LiRpcMethodReturn::None,
        ReturnType::Type(_, t) => {
            let t = SerializableType::try_from(*t).expect("Error parsing method's return type");

            match t {
                SerializableType::Result { ok: _, err } => LiRpcMethodReturn::Result(*err),
                _ => panic!(
                    "A lirpc method should return either the (default) union type or a `Result<(), E>`."
                ),
            }
        }
    };

    let lirpc_method = LiRpcMethodFile {
        name: method_signature.ident.to_string(),
        output,
        message,
        return_type,
    };

    persist(&format!("method-{}.json", lirpc_method.name), &lirpc_method)
        .expect("Error storing method info of lirpc_method");

    TokenStream::from(quote! { #input })
}

fn persist<C>(file_name: &str, contents: &C) -> Result<(), String>
where
    C: Serialize,
{
    let serialized = serde_json::to_string_pretty(contents).map_err(|e| e.to_string())?;

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    let crate_or_bin = env::var("CARGO_CRATE_NAME")
        .or_else(|_| env::var("CARGO_BIN_NAME"))
        .or_else(|_| env::var("CARGO_PKG_NAME"))
        .map_err(|e| e.to_string())?;
    let lirpc_dir = out_dir.join(format!("lirpc-{}", crate_or_bin));

    fs::create_dir_all(&lirpc_dir).map_err(|e| e.to_string())?;
    fs::write(lirpc_dir.join(file_name), serialized).map_err(|e| e.to_string())?;

    Ok(())
}
