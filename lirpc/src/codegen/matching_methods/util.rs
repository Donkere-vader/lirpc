use ts_codegen::ast::{
    Generic, assign_to::AssignTo, statement::expression_statement::ExpressionStatement,
    r#type::Type,
};

use crate::{
    codegen::util::{rust_type_to_ts_type, variant_to_inner_type},
    type_definition::{EnumVariant, EnumVariantFields},
};

pub fn variant_to_handler_type(variant: &EnumVariant, generic: &Generic) -> Type {
    Type::Function {
        args: match &variant.fields {
            EnumVariantFields::Unnamed(rust_types) => rust_types
                .iter()
                .enumerate()
                .map(|(idx, rust_type)| {
                    (
                        AssignTo::Ident(format!("a{idx}")),
                        rust_type_to_ts_type(rust_type),
                    )
                })
                .collect(),
            EnumVariantFields::Named(items) => vec![(
                AssignTo::UnpackObject(
                    items
                        .iter()
                        .map(|(name, _)| (name.to_string(), AssignTo::Ident(name.to_string())))
                        .collect(),
                ),
                Type::Object(
                    items
                        .iter()
                        .map(|(name, rust_type)| {
                            (name.to_string(), true, rust_type_to_ts_type(rust_type))
                        })
                        .collect(),
                ),
            )],
        },
        return_type: Box::new(Type::Generic(generic.clone())),
    }
}

pub fn variant_invocation(variant: &EnumVariant) -> ExpressionStatement {
    ExpressionStatement::FunctionInvocation {
        name: variant.ident.to_string(),
        parameters: match &variant.fields {
            EnumVariantFields::Unnamed(fields) => {
                if fields.is_empty() {
                    Vec::new()
                } else {
                    vec![ExpressionStatement::SpreadOperator {
                        on: Box::new(ExpressionStatement::As {
                            inner: Box::new(ExpressionStatement::Property {
                                name: "inner".to_string(),
                                on: Box::new(ExpressionStatement::Object {
                                    name: "this".to_string(),
                                }),
                            }),
                            r#type: variant_to_inner_type(variant),
                        }),
                    }]
                }
            }
            EnumVariantFields::Named(_) => {
                vec![ExpressionStatement::As {
                    inner: Box::new(ExpressionStatement::Property {
                        name: "inner".to_string(),
                        on: Box::new(ExpressionStatement::Object {
                            name: "this".to_string(),
                        }),
                    }),
                    r#type: variant_to_inner_type(variant),
                }]
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use ts_codegen::{ast::Generic, to_ts::ToTS};

    use crate::{
        codegen::matching_methods::util::variant_to_handler_type, translatable::Type,
        type_definition::EnumVariant,
    };

    #[test]
    fn test_tuple_variant_to_handler_type() {
        let variant = EnumVariant::new_tuple("A".to_string(), vec![Type::String, Type::I32]);

        let actual = variant_to_handler_type(&variant, &Generic("T".to_string())).to_typescript();

        assert_eq!(actual, "(a0: string, a1: number) => T");
    }

    #[test]
    fn test_named_variant_to_handler_type() {
        let variant = EnumVariant::new_named(
            "A".to_string(),
            vec![
                ("a".to_string(), Type::String),
                ("b".to_string(), Type::I32),
            ],
        );

        let actual = variant_to_handler_type(&variant, &Generic("T".to_string())).to_typescript();

        assert_eq!(actual, "({ a, b }: { a: string; b: number }) => T");
    }
}
