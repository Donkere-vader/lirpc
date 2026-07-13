use crate::{api_spec::ApiSpec, codegen::ClientCodeGen};

pub struct RustClientCodeGen;

impl ClientCodeGen for RustClientCodeGen {
    fn gen_client_code(_spec: &ApiSpec) -> String {
        "TODO".to_string()
    }
}
