use ts_codegen::{
    ast::{
        Generic,
        class::ClassDefinition,
        statement::{
            Statement, export_statement::ExportStatement, type_definition::TypeDefinitionStatement,
        },
        r#type::Type,
    },
    to_ts::ToTS,
};

use crate::{
    codegen::{
        class_members::gen_class_members_for_enum, class_methods::gen_class_methods_for_enum,
        util::rust_type_to_ts_type,
    },
    type_definition::{EnumDefinition, StructDefinition, StructFields, TypeDefinition},
};

mod class_members;
mod class_methods;
mod matching_methods;
mod util;

/// Convert a `TypeDefinition` to the contents
/// of a typescript file describing that type in TypeScript.
pub(crate) fn gen_ts_for_type_definition(
    type_definition: &TypeDefinition,
    default_export: bool,
) -> String {
    match type_definition {
        TypeDefinition::Enum(enm) => gen_ts_for_enum(enm, default_export),
        TypeDefinition::Struct(strct) => gen_ts_for_struct(strct, default_export),
    }
}

/// Convert a `EnumDefinition` to the contents
/// of a typescript file describing that enum in TypeScript.
pub(crate) fn gen_ts_for_enum(enm: &EnumDefinition, default_export: bool) -> String {
    let class_export_statement = Statement::ExportStatement(ExportStatement {
        default: default_export,
        inner: Box::new(Statement::ClassDefinition(ClassDefinition {
            ident: enm.ident.to_string(),
            generics: enm
                .generics
                .iter()
                .map(|g| Generic(g.to_string()))
                .collect::<Vec<Generic>>(),
            class_members: gen_class_members_for_enum(enm),
            methods: gen_class_methods_for_enum(enm),
        })),
    });

    class_export_statement.to_typescript()
}

pub(crate) fn gen_ts_for_struct(strct: &StructDefinition, default_export: bool) -> String {
    let type_definition_statement = Statement::ExportStatement(ExportStatement {
        default: default_export,
        inner: Box::new(Statement::TypeDefinition(TypeDefinitionStatement {
            name: strct.ident.to_string(),
            r#type: match &strct.fields {
                StructFields::Named(items) => Type::Object(
                    items
                        .iter()
                        .map(|(name, ty)| (name.to_string(), true, rust_type_to_ts_type(ty)))
                        .collect(),
                ),
                StructFields::Unnamed(items) => {
                    Type::Tuple(items.iter().map(rust_type_to_ts_type).collect())
                }
            },
            generics: strct
                .generics
                .iter()
                .map(|g| Generic(g.to_string()))
                .collect(),
        })),
    });

    type_definition_statement.to_typescript()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_str_eq;

    use crate::{
        codegen::gen_ts_for_enum,
        translatable::Type,
        type_definition::{EnumDefinition, EnumVariant},
    };

    #[test]
    fn codegen_for_basic_enum() {
        let enm = EnumDefinition::new(
            "BasicEnum".to_string(),
            vec![
                EnumVariant::new_unit("A".to_string()),
                EnumVariant::new_unit("B".to_string()),
                EnumVariant::new_unit("C".to_string()),
            ],
            Vec::new(),
        );

        let actual = gen_ts_for_enum(&enm, true);

        assert_str_eq!(&actual, include_str!("test_results/basic_enum.ts"))
    }

    #[test]
    fn codegen_for_enum_with_tuple_variant() {
        let enm = EnumDefinition::new(
            "EnumWithTupleVariant".to_string(),
            vec![
                EnumVariant::new_tuple("A".to_string(), vec![Type::String, Type::U32]),
                EnumVariant::new_unit("B".to_string()),
            ],
            Vec::new(),
        );

        let actual = gen_ts_for_enum(&enm, true);

        assert_str_eq!(
            &actual,
            include_str!("test_results/enum_with_tuple_variant.ts")
        )
    }

    #[test]
    fn codegen_for_enum_with_named_variant_fields() {
        let enm = EnumDefinition::new(
            "EnumWithNamedVariantFields".to_string(),
            vec![
                EnumVariant::new_named(
                    "A".to_string(),
                    vec![
                        ("a".to_string(), Type::String),
                        ("b".to_string(), Type::I32),
                    ],
                ),
                EnumVariant::new_unit("B".to_string()),
            ],
            Vec::new(),
        );

        let actual = gen_ts_for_enum(&enm, true);

        assert_str_eq!(
            &actual,
            include_str!("test_results/enum_with_named_variant_fields.ts")
        )
    }
}
