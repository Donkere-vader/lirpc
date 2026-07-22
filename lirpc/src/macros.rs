/// For compiling your server's spec into
/// a compiled json spec. automatically
/// inserting the package's name and version.
///
/// # Example
///
/// ```rust
/// # use lirpc::{ServerBuilder, compile_json_api_spec, handlers, types};
/// # use lirpc_macros::LiRpcType;
/// # use serde::{Serialize, Deserialize};
/// #
/// # #[derive(LiRpcType, Serialize, Deserialize)]
/// # struct GreetingRequest;
/// # #[derive(LiRpcType, Serialize, Deserialize)]
/// # struct GreetingResponse;
/// #
/// # pub async fn greet() -> () {
/// #     todo!()
/// # }
/// #
/// let server = ServerBuilder::new()
///     .with_handlers(handlers!(greet))
///     .with_types(types!(GreetingRequest, GreetingResponse))
///     .build();
///
/// let api_spec: String = compile_json_api_spec!(server)
///     .unwrap();
///
/// // Note that the fields "name" and "version" are pulled
/// // from the env variables that cargo sets.
/// assert_eq!(api_spec, "{\"name\":\"lirpc\",\"version\":\"0.1.0\",\"methods\":{\"greet\":{\"messages\":[],\"returns\":\"unit\"}},\"types\":{\"GreetingRequest\":{\"struct\":{\"ident\":\"GreetingRequest\",\"fields\":{\"unnamed\":[]},\"generics\":[]}},\"GreetingResponse\":{\"struct\":{\"ident\":\"GreetingResponse\",\"fields\":{\"unnamed\":[]},\"generics\":[]}}}}");
/// ```
#[macro_export]
macro_rules! compile_json_api_spec {
    ($server:ident) => {
        $server.compile_json_api_spec(
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
    };
}

/// Shorthand for providing a list of handlers and their name
/// to the `ServerBuilder`.
///
/// # Example
///
/// ```rust
/// # use lirpc::{handlers, NamedHandler};
/// #
/// # pub async fn greet() { todo!() }
/// # pub async fn ping() { todo!() }
/// #
/// let h: Vec<NamedHandler<(), ()>> = handlers!(greet, ping);
///
/// // is essentially the same as
///
/// vec![
///     NamedHandler::<(), ()>::new("greet".to_string(), greet),
///     NamedHandler::<(), ()>::new("ping".to_string(), ping),
/// ];
/// ```
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
