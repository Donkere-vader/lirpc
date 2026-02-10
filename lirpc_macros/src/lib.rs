mod lirpc_method;
mod lirpc_type;

use proc_macro::TokenStream;
use serde::Serialize;
use std::{env, fs, path::PathBuf};

#[proc_macro_attribute]
pub fn lirpc_type(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let type_file = crate::lirpc_type::lirpc_type(proc_macro2::TokenStream::from(item.clone()));

    persist(&format!("type-{}.json", type_file.name), &type_file)
        .expect("Error storing type of lirpc_type");

    item
}

#[proc_macro_attribute]
pub fn lirpc_method(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let lirpc_method =
        crate::lirpc_method::lirpc_method(proc_macro2::TokenStream::from(item.clone()));

    persist(&format!("method-{}.json", lirpc_method.name), &lirpc_method)
        .expect("Error storing method info of lirpc_method");

    item
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
