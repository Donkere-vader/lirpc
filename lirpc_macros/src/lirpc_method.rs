use lirpc::contracts::{
    lirpc_method_file::{LiRpcMethodFile, LiRpcMethodReturn},
    serializable_type::SerializableType,
};
use proc_macro2::TokenStream;
use syn::{ItemFn, ReturnType};

pub(crate) fn lirpc_method(item: TokenStream) -> LiRpcMethodFile {
    let input = match syn::parse2::<ItemFn>(item) {
        Ok(i) => i,
        Err(_) => panic!("Expected a function"),
    };

    let ItemFn {
        sig: method_signature,
        ..
    } = input.clone();

    let mut output: Option<SerializableType> = None;
    let mut message: Option<SerializableType> = None;

    for arg in &method_signature.inputs {
        if let syn::FnArg::Typed(pat) = arg
            && let syn::Type::Path(syn::TypePath { path, .. }) = &*pat.ty
            && let Some(last) = path.segments.last()
        {
            let ident = last.ident.to_string();
            if matches!(ident.as_str(), "Output" | "OutputStream" | "Message")
                && let syn::PathArguments::AngleBracketed(abga) = &last.arguments
            {
                let inner_ty_opt = abga.args.iter().find_map(|ga| {
                    if let syn::GenericArgument::Type(t) = ga {
                        Some(t.clone())
                    } else {
                        None
                    }
                });
                if let Some(inner_ty) = inner_ty_opt {
                    let st = SerializableType::try_from(inner_ty).unwrap_or_else(|err| {
                        panic!("Unsupported inner type for {}: {}", ident, err)
                    });
                    match ident.as_str() {
                        "Output" | "OutputStream" => {
                            if output.is_some() {
                                panic!(
                                    "Multiple output parameters found (Output/OutputStream are exclusive)"
                                );
                            }
                            output = Some(st);
                        }
                        "Message" => {
                            if message.is_some() {
                                panic!(
                                    "Multiple message parameters found (only one Message<T> allowed)"
                                );
                            }
                            message = Some(st);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let return_type = match method_signature.output {
        ReturnType::Default => LiRpcMethodReturn::None,
        ReturnType::Type(_, t) => {
            let t = SerializableType::try_from(*t).expect("Error parsing method's return type");

            match t {
                SerializableType::Result { ok: _, err } => LiRpcMethodReturn::Result(*err),
                _ => panic!(
                    "A lirpc method should return either the (default) union type or a `Result<(), E>`."
                ),
            }
        }
    };

    LiRpcMethodFile {
        name: method_signature.ident.to_string(),
        output,
        message,
        return_type,
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use serde_json::{Value, json};

    use super::lirpc_method;

    #[test]
    fn lirpc_method_with_no_return_type() {
        let token_stream = quote! {
            fn my_handler() {
                todo!()
            }
        };

        let lirpc_method_file = lirpc_method(token_stream);

        assert_eq!(
            serde_json::to_value(lirpc_method_file).expect("Serialization failed"),
            json!({
                "message": Value::Null,
                "name": "my_handler",
                "output": Value::Null,
                "return_type": {
                    "type": "none"
                }
            })
        );
    }

    #[test]
    fn lirpc_method_with_result_return_type() {
        let token_stream = quote! {
            fn my_handler() -> Result<(), MyError> {
                todo!()
            }
        };

        let lirpc_method_file = lirpc_method(token_stream);

        assert_eq!(
            serde_json::to_value(lirpc_method_file).expect("Serialization failed"),
            json!({
                "message": Value::Null,
                "name": "my_handler",
                "output": Value::Null,
                "return_type": {
                    "type": "result",
                    "err_variant": {
                        "type": "custom",
                        "name": "MyError",
                    },
                },
            }),
        );
    }

    #[test]
    fn lirpc_method_with_message() {
        let token_stream = quote! {
            fn my_handler(Message(message): Message<SomeMessage>) {
                todo!()
            }
        };

        let lirpc_method_file = lirpc_method(token_stream);

        assert_eq!(
            serde_json::to_value(lirpc_method_file).expect("Serialization failed"),
            json!({
                "message": {
                    "type": "custom",
                    "name": "SomeMessage",
                },
                "name": "my_handler",
                "output": Value::Null,
                "return_type": {"type": "none"},
            }),
        );
    }

    #[test]
    fn lirpc_method_with_output() {
        let token_stream = quote! {
            fn my_handler(Output(message): Output<SomeResponse>) {
                todo!()
            }
        };

        let lirpc_method_file = lirpc_method(token_stream);

        assert_eq!(
            serde_json::to_value(lirpc_method_file).expect("Serialization failed"),
            json!({
                "message": Value::Null,
                "name": "my_handler",
                "output": {
                    "type": "custom",
                    "name": "SomeResponse",
                },
                "return_type": {"type": "none"},
            }),
        );
    }
}
