use std::collections::BTreeMap;

use crate::api_spec::ApiSpec;

pub trait CodeGen {
    fn generate_package(spec: &ApiSpec) -> BTreeMap<String, String>;
}
