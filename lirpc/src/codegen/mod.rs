use crate::api_spec::ApiSpec;

pub mod rust_client_codegen;
#[cfg(test)]
mod tests;
pub mod typescript_client_codegen;

pub trait ClientCodeGen {
    fn gen_client_code(spec: &ApiSpec) -> String;
}
