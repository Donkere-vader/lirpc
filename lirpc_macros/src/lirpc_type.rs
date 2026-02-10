use std::collections::BTreeMap;

use lirpc::contracts::{
    contract_file::LiRpcType, lirpc_type_file::LiRpcTypeFile, serializable_type::SerializableType,
};
use proc_macro2::TokenStream;
use syn::{ItemEnum, ItemStruct, Variant};

pub(crate) fn lirpc_type(item: TokenStream) -> LiRpcTypeFile {
    if let Ok(input) = syn::parse2::<ItemStruct>(item.clone()) {
        lirpc_type_from_struct(input)
    } else if let Ok(input) = syn::parse2::<ItemEnum>(item) {
        lirpc_type_from_enum(input)
    } else {
        panic!("expected struct or enum");
    }
}

fn lirpc_type_from_enum(input: ItemEnum) -> LiRpcTypeFile {
    let ItemEnum {
        variants: enum_variants,
        ident: enum_ident,
        ..
    } = input.clone();

    let mut variants = BTreeMap::new();

    for var in enum_variants.into_iter() {
        let Variant {
            ident: variant_ident,
            fields: enum_fields,
            ..
        } = var;

        let fields: BTreeMap<String, SerializableType> = enum_fields.iter()
            .map(|f| (
                    f.ident.as_ref().expect("Enum annoted with lirpc_type cannot have variants without identifiers (names) for its fields").to_string(),
                    SerializableType::try_from(f.ty.clone()).expect("Unsupported type")
                )
            )
            .collect();

        variants.insert(variant_ident.to_string(), fields);
    }

    let enum_type = LiRpcType::Enum { variants };

    LiRpcTypeFile {
        name: enum_ident.to_string(),
        r#type: enum_type,
    }
}

fn lirpc_type_from_struct(input: ItemStruct) -> LiRpcTypeFile {
    let ItemStruct {
        ident: struct_ident,
        fields: struct_fields,
        ..
    } = input.clone();

    let fields = struct_fields
        .into_iter()
        .map(|f| (
                f.ident
                    .expect("Struct annoted with lirpc_type must have identifiers (names) for its fields")
                    .to_string(),
                SerializableType::try_from(f.ty).expect("Unsupported time cannot be used"),
            )
        )
        .collect::<BTreeMap<String, SerializableType>>();

    LiRpcTypeFile {
        name: struct_ident.to_string(),
        r#type: LiRpcType::Struct { fields },
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use serde_json::json;

    use super::lirpc_type;

    #[test]
    fn lirpc_type_macro_from_struct() {
        let token_stream = quote! {
            struct MyType {
                value1: String,
                value2: MyOtherType,
            }
        };

        let lirpc_type_file = lirpc_type(token_stream);

        assert_eq!(
            serde_json::to_value(&lirpc_type_file).expect("Serialization failed"),
            json!({
                "name": "MyType",
                "type": {
                    "type": "struct",
                    "fields": {
                        "value1": {
                            "type": "string",
                        },
                        "value2": {
                            "type": "custom",
                            "name": "MyOtherType",
                        },
                    },
                },
            }),
        );
    }

    #[test]
    fn lirpc_type_macro_from_enum() {
        let token_stream = quote! {
            enum MyType {
                Variant,
                VariantWithNamedFields {
                    value1: String,
                    value2: MyOtherType,
                }
            }
        };

        let lirpc_type_file = lirpc_type(token_stream);

        assert_eq!(
            serde_json::to_value(&lirpc_type_file).expect("Serialization failed"),
            json!({
                "name": "MyType",
                "type": {
                    "type": "enum",
                    "variants": {
                        "Variant": {},
                        "VariantWithNamedFields": {
                            "value1": {
                                "type" : "string",
                            },
                            "value2": {
                                "type": "custom",
                                "name": "MyOtherType",
                            },
                        },
                    },
                },
            }),
        );
    }
}
