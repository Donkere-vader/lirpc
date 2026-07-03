use ts_codegen::ast::{
    Generic, Visibility,
    assign_to::AssignTo,
    class::class_method::ClassMethod,
    code_block::CodeBlock,
    statement::{
        Statement,
        assignment_statement::{AssignmentStatement, AssignmentType},
        expression_statement::{ExpressionStatement, NewObjectItem},
        return_statement::ReturnStatement,
    },
    r#type::Type,
};

use crate::{
    codegen::{
        matching_methods::{
            r#match::gen_ts_for_match_method, match_partial::gen_ts_for_match_partial,
            match_partial_with_default::gen_ts_for_match_partial_with_default,
        },
        util::{gen_inner_type, gen_variant_type, rust_type_to_ts_type},
    },
    type_definition::{EnumDefinition, EnumVariant, EnumVariantFields},
};

pub fn gen_class_methods_for_enum(enm: &EnumDefinition) -> Vec<ClassMethod> {
    let mut methods = vec![gen_constructor_method_for_enum(enm)];
    methods.append(&mut gen_static_constructors_for_enum(enm));
    methods.append(&mut gen_matching_methods_for_enum(enm));

    methods
}

fn gen_constructor_method_for_enum(enm: &EnumDefinition) -> ClassMethod {
    ClassMethod {
        visibility: Visibility::Private,
        r#static: false,
        ident: "constructor".to_string(),
        generics: vec![], // no generics on constructors in ts
        args: vec![
            (
                AssignTo::Ident("variant".to_string()),
                gen_variant_type(enm),
            ),
            (AssignTo::Ident("inner".to_string()), gen_inner_type(enm)),
        ],
        return_type: None, // implicitly returns own class instance
        body: CodeBlock(vec![
            Statement::Assignment(AssignmentStatement {
                kind: AssignmentType::Existing,
                left_hand_side: AssignTo::Ident("this.variant".to_string()),
                right_hand_side: ExpressionStatement::Object {
                    name: "variant".to_string(),
                },
            }),
            Statement::Assignment(AssignmentStatement {
                kind: AssignmentType::Existing,
                left_hand_side: AssignTo::Ident("this.inner".to_string()),
                right_hand_side: ExpressionStatement::Object {
                    name: "inner".to_string(),
                },
            }),
        ]),
    }
}

fn gen_static_constructors_for_enum(enm: &EnumDefinition) -> Vec<ClassMethod> {
    enm.variants
        .iter()
        .map(|v| gen_static_constructor_for_variant(enm, v))
        .collect()
}

fn gen_static_constructor_for_variant(
    enm: &EnumDefinition,
    EnumVariant { ident, fields }: &EnumVariant,
) -> ClassMethod {
    let (args, inner) = match fields {
        EnumVariantFields::Unnamed(fields) => (
            fields
                .iter()
                .enumerate()
                .map(|(idx, rust_type)| {
                    (
                        AssignTo::Ident(format!("a{idx}")),
                        rust_type_to_ts_type(rust_type),
                    )
                })
                .collect(),
            ExpressionStatement::NewArray {
                items: fields
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| ExpressionStatement::Object {
                        name: format!("a{idx}"),
                    })
                    .collect(),
            },
        ),
        EnumVariantFields::Named(fields) => (
            vec![(
                AssignTo::UnpackObject(
                    fields
                        .iter()
                        .map(|(name, _)| (name.to_string(), AssignTo::Ident(name.to_string())))
                        .collect(),
                ),
                Type::Object(
                    fields
                        .iter()
                        .map(|(name, rust_type)| {
                            (name.to_string(), true, rust_type_to_ts_type(rust_type))
                        })
                        .collect(),
                ),
            )],
            ExpressionStatement::NewObject {
                items: fields
                    .iter()
                    .map(|(name, _)| {
                        NewObjectItem::Field(
                            name.to_string(),
                            ExpressionStatement::Object {
                                name: name.to_string(),
                            },
                        )
                    })
                    .collect(),
            },
        ),
    };

    ClassMethod {
        visibility: Visibility::Public,
        r#static: true,
        ident: ident.to_string(),
        generics: enm
            .generics
            .iter()
            .map(|g| Generic(g.to_string()))
            .collect(),
        args,
        return_type: None, // return type will be the class implicitly
        body: CodeBlock(vec![Statement::ReturnStatement(ReturnStatement {
            value: ExpressionStatement::NewClass {
                name: enm.ident.to_string(),
                parameters: vec![
                    ExpressionStatement::ConstantString(ident.to_string()),
                    inner,
                ],
            },
        })]),
    }
}

fn gen_matching_methods_for_enum(enm: &EnumDefinition) -> Vec<ClassMethod> {
    vec![
        gen_ts_for_match_method(enm),
        gen_ts_for_match_partial_with_default(enm),
        gen_ts_for_match_partial(enm),
    ]
}
