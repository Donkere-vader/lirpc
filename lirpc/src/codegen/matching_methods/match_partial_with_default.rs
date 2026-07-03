use std::iter::once;

use ts_codegen::ast::{
    Generic, Visibility,
    assign_to::AssignTo,
    class::class_method::ClassMethod,
    code_block::CodeBlock,
    statement::{
        Statement,
        expression_statement::{ExpressionStatement, InfixOperator},
        if_statement::IfStatement,
        return_statement::ReturnStatement,
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

pub fn gen_ts_for_match_partial_with_default(enm: &EnumDefinition) -> ClassMethod {
    let generic = free_generic(
        &enm.generics
            .iter()
            .map(|g| Generic(g.to_string()))
            .collect::<Vec<Generic>>(),
    );

    ClassMethod {
        visibility: Visibility::Public,
        r#static: false,
        ident: "matchPartialWithDefault".to_string(),
        generics: vec![generic.clone()],
        args: vec![(
            AssignTo::UnpackObject(
                enm.variants
                    .iter()
                    .map(|EnumVariant { ident, fields: _ }| {
                        (ident.to_string(), AssignTo::Ident(ident.to_string()))
                    })
                    .chain(once(("_".to_string(), AssignTo::Ident("_".to_string()))))
                    .collect(),
            ),
            Type::Object(
                enm.variants
                    .iter()
                    .map(|variant| {
                        (
                            variant.ident.to_string(),
                            false,
                            variant_to_handler_type(variant, &generic),
                        )
                    })
                    .chain(once((
                        "_".to_string(),
                        true,
                        Type::Function {
                            args: vec![(
                                AssignTo::Ident("enm".to_string()),
                                Type::Class(
                                    enm.ident.to_string(),
                                    enm.generics
                                        .iter()
                                        .map(|g| Type::Generic(Generic(g.to_string())))
                                        .collect(),
                                ),
                            )],
                            return_type: Box::new(Type::Generic(generic.clone())),
                        },
                    )))
                    .collect(),
            ),
        )],
        return_type: Some(Type::Generic(generic)),
        body: CodeBlock(vec![match_body_statement(enm)]),
    }
}

fn match_body_statement(enm: &EnumDefinition) -> Statement {
    let mut branches = enm.variants.iter().map(|variant| {
        (
            ExpressionStatement::InfixOperation {
                left: Box::new(ExpressionStatement::InfixOperation {
                    left: Box::new(ExpressionStatement::Property {
                        name: "variant".to_string(),
                        on: Box::new(ExpressionStatement::Object {
                            name: "this".to_string(),
                        }),
                    }),
                    op: InfixOperator::StrictEqual,
                    right: Box::new(ExpressionStatement::ConstantString(
                        variant.ident.to_string(),
                    )),
                }),
                op: InfixOperator::BooleanAnd,
                right: Box::new(ExpressionStatement::InfixOperation {
                    left: Box::new(ExpressionStatement::Object {
                        name: variant.ident.to_string(),
                    }),
                    op: InfixOperator::StrictNotEqual,
                    right: Box::new(ExpressionStatement::Undefined),
                }),
            },
            CodeBlock(vec![Statement::ReturnStatement(ReturnStatement {
                value: variant_invocation(variant),
            })]),
        )
    });

    let else_statement = Statement::ReturnStatement(ReturnStatement {
        value: ExpressionStatement::FunctionInvocation {
            name: "_".to_string(),
            parameters: vec![ExpressionStatement::Object {
                name: "this".to_string(),
            }],
        },
    });

    match branches.next() {
        Some(if_branch) => Statement::If(IfStatement {
            if_branch,
            if_else_branches: branches.collect(),
            else_branch: Some(CodeBlock(vec![else_statement])),
        }),
        None => else_statement,
    }
}
