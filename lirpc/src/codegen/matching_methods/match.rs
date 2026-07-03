use ts_codegen::ast::{
    Generic, Visibility,
    assign_to::AssignTo,
    class::class_method::ClassMethod,
    code_block::CodeBlock,
    statement::{
        Statement, expression_statement::ExpressionStatement, return_statement::ReturnStatement,
        switch_case_statement::SwitchCaseStatement,
    },
    r#type::Type,
};

use crate::{
    codegen::{
        matching_methods::util::{variant_invocation, variant_to_handler_type},
        util::free_generic,
    },
    type_definition::{EnumDefinition, EnumVariant},
};

pub fn gen_ts_for_match_method(enm: &EnumDefinition) -> ClassMethod {
    let generic = free_generic(
        &enm.generics
            .iter()
            .map(|g| Generic(g.to_string()))
            .collect::<Vec<Generic>>(),
    );

    ClassMethod {
        visibility: Visibility::Public,
        r#static: false,
        ident: "match".to_string(),
        generics: vec![generic.clone()],
        args: vec![(
            AssignTo::UnpackObject(
                enm.variants
                    .iter()
                    .map(|EnumVariant { ident, fields: _ }| {
                        (ident.to_string(), AssignTo::Ident(ident.to_string()))
                    })
                    .collect(),
            ),
            Type::Object(
                enm.variants
                    .iter()
                    .map(|variant| {
                        (
                            variant.ident.to_string(),
                            true,
                            variant_to_handler_type(variant, &generic),
                        )
                    })
                    .collect(),
            ),
        )],
        return_type: Some(Type::Generic(generic)),
        body: CodeBlock(vec![Statement::SwitchCase(SwitchCaseStatement {
            expression: ExpressionStatement::Property {
                name: "variant".to_string(),
                on: Box::new(ExpressionStatement::Object {
                    name: "this".to_string(),
                }),
            },
            cases: enm
                .variants
                .iter()
                .map(|v| {
                    (
                        ExpressionStatement::ConstantString(v.ident.to_string()),
                        CodeBlock(vec![Statement::ReturnStatement(ReturnStatement {
                            value: variant_invocation(v),
                        })]),
                    )
                })
                .collect(),
            default: None,
        })]),
    }
}
