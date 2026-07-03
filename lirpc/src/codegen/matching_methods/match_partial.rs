use ts_codegen::ast::{
    Generic, Visibility,
    assign_to::AssignTo,
    class::class_method::ClassMethod,
    code_block::CodeBlock,
    statement::{
        Statement,
        expression_statement::{ExpressionStatement, InlineFunctionBody, NewObjectItem},
        return_statement::ReturnStatement,
    },
    r#type::Type,
};

use crate::{
    codegen::{matching_methods::util::variant_to_handler_type, util::free_generic},
    type_definition::EnumDefinition,
};

pub fn gen_ts_for_match_partial(enm: &EnumDefinition) -> ClassMethod {
    let generic = free_generic(
        &enm.generics
            .iter()
            .map(|g| Generic(g.to_string()))
            .collect::<Vec<Generic>>(),
    );

    ClassMethod {
        visibility: Visibility::Public,
        r#static: false,
        ident: "matchPartial".to_string(),
        generics: vec![generic.clone()],
        args: vec![(
            AssignTo::Ident("handlers".to_string()),
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
                    .collect(),
            ),
        )],
        return_type: Some(Type::Union(
            Box::new(Type::Generic(generic)),
            vec![Type::Undefined],
        )),
        body: CodeBlock(vec![Statement::ReturnStatement(ReturnStatement {
            value: ExpressionStatement::MemberInvocation {
                name: "matchPartialWithDefault".to_string(),
                on: Box::new(ExpressionStatement::Object {
                    name: "this".to_string(),
                }),
                parameters: vec![ExpressionStatement::NewObject {
                    items: vec![
                        NewObjectItem::Spread(ExpressionStatement::Object {
                            name: "handlers".to_string(),
                        }),
                        NewObjectItem::Field(
                            "_".to_string(),
                            ExpressionStatement::InlineFunctionDefinition {
                                args: vec![(
                                    AssignTo::Ident("_".to_string()),
                                    Type::Class(
                                        enm.ident.to_string(),
                                        enm.generics
                                            .iter()
                                            .map(|g| Type::Generic(Generic(g.to_string())))
                                            .collect(),
                                    ),
                                )],
                                return_type: None, // implicit
                                code: InlineFunctionBody::ExpressionStatement(Box::new(
                                    ExpressionStatement::Undefined,
                                )),
                            },
                        ),
                    ],
                }],
            },
        })]),
    }
}
