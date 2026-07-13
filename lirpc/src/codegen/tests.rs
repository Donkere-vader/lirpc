use std::collections::HashMap;

use crate::{
    api_spec::ApiSpec,
    codegen::{
        ClientCodeGen, rust_client_codegen::RustClientCodeGen,
        typescript_client_codegen::TypeScriptClientCodeGen,
    },
};

#[test]
fn gen_rust_client_code_from_spec() {
    let spec = build_api_spec();

    let generated_code = RustClientCodeGen::gen_client_code(&spec);

    assert_eq!(generated_code, "TODO");
}

#[test]
fn gen_typescript_client_code_from_spec() {
    let spec = build_api_spec();

    let generated_code = TypeScriptClientCodeGen::gen_client_code(&spec);

    assert_eq!(generated_code, "TODO");
}

fn build_api_spec() -> ApiSpec {
    ApiSpec::new(
        "MyApi".to_string(),
        "0.1.0".to_string(),
        HashMap::from([]),
        HashMap::from([]),
    )
    .expect("Failed to build api spec")
}
