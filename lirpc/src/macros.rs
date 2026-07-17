#[macro_export]
macro_rules! compile_json_api_spec {
    ($server:ident) => {
        $server.compile_json_api_spec(
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
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
