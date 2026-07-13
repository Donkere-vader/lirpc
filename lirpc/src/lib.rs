mod codegen;
pub mod connection_details;
pub mod error;
pub mod extractors;
mod handler;
pub mod lirpc_message;
mod server;
mod service;

pub mod api_spec;
pub mod lirpc_type;
pub mod translatable;
pub mod type_definition;

pub use server::NamedHandler;
pub use server::ServerBuilder;

// TODO place in correct place
#[macro_export]
macro_rules! compile_json_api_spec {
    ($server:ident) => {
        $server
            .compile_json_api_spec(
                env!("CARGO_PKG_NAME").to_string(),
                env!("CARGO_PKG_VERSION").to_string(),
            )
            .await
    };
}

#[macro_export]
macro_rules! handlers {
    ($($h:ident),*) => {
        vec![$(
            $crate::NamedHandler::new(stringify!($h).to_string(), $h)
        ),*]
    };
}

#[macro_export]
macro_rules! types {
    ($($t:ident),*) => {
        vec![$((
            stringify!($t).to_string(),
            <$t as lirpc::lirpc_type::LiRpcType>::translate()
        )),*]
    };
}
