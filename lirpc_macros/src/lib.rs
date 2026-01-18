use std::{collections::BTreeMap, env, fs, path::PathBuf};

use lirpc::contracts::{
    lirpc_method_file::LiRpcMethodFile, lirpc_type_file::LiRpcTypeFile,
    serializable_type::SerializableType,
};
use proc_macro::TokenStream;
use quote::quote;
use serde::Serialize;
use syn::{ItemFn, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn lirpc_type(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let ItemStruct {
        ident: struct_ident,
        fields: struct_fields,
        ..
    } = input.clone();

    let fields = struct_fields
        .into_iter()
        .map(|f| {
            (
                f.ident
                    .expect("Struct annoted with lirpc_type must have identifiers (names) for its fields")
                    .to_string(),
                SerializableType::try_from(f.ty).expect("Unsupported time cannot be used"),
            )
        })
        .collect::<BTreeMap<String, SerializableType>>();

    let lirpc_type = LiRpcTypeFile {
        name: struct_ident.to_string(),
        fields,
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

    let lirpc_method = LiRpcMethodFile {
        name: method_signature.ident.to_string(),
        output,
        message,
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
