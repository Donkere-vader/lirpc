use std::collections::HashMap;

use lirpc::{
    api_spec::{ApiSpec, LiRpcMethodSpec},
    codegen::CodeGen,
    translatable::Type,
    type_definition::{
        EnumDefinition, EnumVariant, StructDefinition, StructFields, TypeDefinition,
    },
};

use crate::RustCodeGen;

const EMPTY_CARGO_TOML: &str = r#"[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
lirpc_client = {}
serde = { version = "1.0.228", features = ["derive"] }
"#;

const EMPTY_LIB_RS: &str = r#"use lirpc_client::{Client, transport::Transport};
use serde::{Deserialize, Serialize};
"#;

#[test]
fn test_empty_api_spec() {
    let spec = ApiSpec::new(
        "my-app".to_string(),
        "0.1.0".to_string(),
        HashMap::new(),
        HashMap::new(),
    )
    .unwrap();

    let mut package = RustCodeGen::generate_package(&spec);

    let cargo_toml = package.remove("Cargo.toml").unwrap();
    let lib_rs = package.remove("src/lib.rs").unwrap();

    assert!(package.is_empty());
    assert_eq!(cargo_toml, EMPTY_CARGO_TOML);
    assert_eq!(lib_rs, EMPTY_LIB_RS);
}

const GREETER_CARGO_TOML: &str = r#"[package]
name = "greeter"
version = "0.1.0"
edition = "2024"

[dependencies]
lirpc_client = {}
serde = { version = "1.0.228", features = ["derive"] }
"#;

const GREETER_LIB_RS: &str = r#"use lirpc_client::{Client, transport::Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingResponse {
    pub msg: String,
}

pub async fn greet<T, F>(
    client: &mut Client<T, F>,
    request: GreetingRequest,
) -> Result<GreetingResponse, lirpc_client::error::Error>
where
    T: Transport<F>,
{
    client
        .call::<GreetingRequest, GreetingResponse>("greet".to_string(), Some(request))
        .await?
        .resolve()
        .await
}
"#;

#[test]
fn test_greeter_api_spec() {
    let spec = ApiSpec::new(
        "greeter".to_string(),
        "0.1.0".to_string(),
        HashMap::from([(
            "greet".to_string(),
            LiRpcMethodSpec {
                messages: vec![Type::TypeRef("GreetingRequest".to_string())],
                returns: Type::TypeRef("GreetingResponse".to_string()),
            },
        )]),
        HashMap::from([
            (
                "GreetingRequest".to_string(),
                TypeDefinition::Struct(Box::new(StructDefinition {
                    ident: "GreetingRequest".to_string(),
                    fields: StructFields::Named(vec![("name".to_string(), Type::String)]),
                    generics: vec![],
                })),
            ),
            (
                "GreetingResponse".to_string(),
                TypeDefinition::Struct(Box::new(StructDefinition {
                    ident: "GreetingResponse".to_string(),
                    fields: StructFields::Named(vec![("msg".to_string(), Type::String)]),
                    generics: vec![],
                })),
            ),
        ]),
    )
    .unwrap();

    let mut package = RustCodeGen::generate_package(&spec);

    let cargo_toml = package.remove("Cargo.toml").unwrap();
    let lib_rs = package.remove("src/lib.rs").unwrap();

    assert!(package.is_empty());
    assert_eq!(cargo_toml, GREETER_CARGO_TOML);
    assert_eq!(lib_rs, GREETER_LIB_RS);
}

const AUTH_LIB_RS: &str = r#"use lirpc_client::{Client, transport::Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMessage {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyError {
    AuthFailure,
    Unauthenticated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMessage {
    pub secret: String,
}

pub async fn login<T, F>(
    client: &mut Client<T, F>,
    request: AuthMessage,
) -> Result<Result<(), MyError>, lirpc_client::error::Error>
where
    T: Transport<F>,
{
    client
        .call::<AuthMessage, Result<(), MyError>>("login".to_string(), Some(request))
        .await?
        .resolve()
        .await
}

pub async fn protected_function<T, F>(
    client: &mut Client<T, F>,
) -> Result<SecretMessage, lirpc_client::error::Error>
where
    T: Transport<F>,
{
    client
        .call::<(), SecretMessage>("protected_function".to_string(), None)
        .await?
        .resolve()
        .await
}
"#;

#[test]
fn test_auth_like_api_spec() {
    let spec = ApiSpec::new(
        "auth".to_string(),
        "0.1.0".to_string(),
        HashMap::from([
            (
                "login".to_string(),
                LiRpcMethodSpec {
                    messages: vec![Type::TypeRef("AuthMessage".to_string())],
                    returns: Type::Result(
                        Box::new(Type::Unit),
                        Box::new(Type::TypeRef("MyError".to_string())),
                    ),
                },
            ),
            (
                "protected_function".to_string(),
                LiRpcMethodSpec {
                    messages: vec![],
                    returns: Type::TypeRef("SecretMessage".to_string()),
                },
            ),
        ]),
        HashMap::from([
            (
                "AuthMessage".to_string(),
                TypeDefinition::Struct(Box::new(StructDefinition {
                    ident: "AuthMessage".to_string(),
                    fields: StructFields::Named(vec![
                        ("username".to_string(), Type::String),
                        ("password".to_string(), Type::String),
                    ]),
                    generics: vec![],
                })),
            ),
            (
                "SecretMessage".to_string(),
                TypeDefinition::Struct(Box::new(StructDefinition {
                    ident: "SecretMessage".to_string(),
                    fields: StructFields::Named(vec![("secret".to_string(), Type::String)]),
                    generics: vec![],
                })),
            ),
            (
                "MyError".to_string(),
                TypeDefinition::Enum(Box::new(EnumDefinition::new(
                    "MyError".to_string(),
                    vec![
                        EnumVariant::new_unit("AuthFailure".to_string()),
                        EnumVariant::new_unit("Unauthenticated".to_string()),
                    ],
                    vec![],
                ))),
            ),
        ]),
    )
    .unwrap();

    let mut package = RustCodeGen::generate_package(&spec);

    let lib_rs = package.remove("src/lib.rs").unwrap();

    assert_eq!(lib_rs, AUTH_LIB_RS);
}
