use crate::{api_spec::ApiSpec, codegen::ClientCodeGen};

pub struct TypeScriptClientCodeGen;

impl ClientCodeGen for TypeScriptClientCodeGen {
    fn gen_client_code(_spec: &ApiSpec) -> String {
        "TODO".to_string()
    }
}
