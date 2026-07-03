use crate::{
    ast::{assign_to::AssignTo, code_block::CodeBlock, r#type::Type},
    to_ts::ToTS,
};

#[derive(Debug)]
pub enum InfixOperator {
    Add,
    Subtract,
    Times,
    Divide,
    Exponentiation,
    Modulus,
    BooleanAnd,
    BooleanOr,
    Equal,
    StrictEqual,
    NotEqual,
    StrictNotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl ToTS for InfixOperator {
    fn to_typescript(&self) -> String {
        match self {
            Self::Add => " + ",
            Self::Subtract => " - ",
            Self::Times => " * ",
            Self::Divide => "/",
            Self::Exponentiation => "**",
            Self::Modulus => " % ",
            Self::BooleanAnd => " && ",
            Self::BooleanOr => "||",
            Self::Equal => " == ",
            Self::StrictEqual => " === ",
            Self::NotEqual => " != ",
            Self::StrictNotEqual => " !== ",
            Self::GreaterThan => " > ",
            Self::LessThan => " < ",
            Self::GreaterThanOrEqual => " >= ",
            Self::LessThanOrEqual => " <= ",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub enum NewObjectItem {
    Spread(ExpressionStatement),
    Field(String, ExpressionStatement),
}

#[derive(Debug)]
pub enum InlineFunctionBody {
    CodeBlock(CodeBlock),
    ExpressionStatement(Box<ExpressionStatement>),
}

impl ToTS for InlineFunctionBody {
    fn to_typescript(&self) -> String {
        match self {
            InlineFunctionBody::CodeBlock(code_block) => {
                format!("{{ {} }}", code_block.to_typescript())
            }
            InlineFunctionBody::ExpressionStatement(expression_statement) => {
                expression_statement.to_typescript()
            }
        }
    }
}

#[derive(Debug)]
pub enum ExpressionStatement {
    FunctionInvocation {
        name: String,
        parameters: Vec<ExpressionStatement>,
    },
    MemberInvocation {
        name: String,
        on: Box<ExpressionStatement>,
        parameters: Vec<ExpressionStatement>,
    },
    Property {
        name: String,
        on: Box<ExpressionStatement>,
    },
    Indexing {
        on: Box<ExpressionStatement>,
        idx: Box<ExpressionStatement>,
    },
    InfixOperation {
        left: Box<ExpressionStatement>,
        op: InfixOperator,
        right: Box<ExpressionStatement>,
    },
    Object {
        name: String,
    },
    NewClass {
        name: String,
        parameters: Vec<ExpressionStatement>,
    },
    NewArray {
        items: Vec<ExpressionStatement>,
    },
    NewObject {
        items: Vec<NewObjectItem>,
    },
    InlineFunctionDefinition {
        args: Vec<(AssignTo, Type)>,
        return_type: Option<Type>,
        code: InlineFunctionBody,
    },
    SpreadOperator {
        on: Box<ExpressionStatement>,
    },
    As {
        inner: Box<ExpressionStatement>,
        r#type: Type,
    },
    ConstantString(String),
    ConstantNumber(i128),
    ConstantBoolean(bool),
    Undefined,
    RawStatement(String),
}

impl ToTS for ExpressionStatement {
    fn to_typescript(&self) -> String {
        match self {
            Self::FunctionInvocation { name, parameters } => {
                format!("{name}({})", parameters_to_ts(parameters))
            }
            Self::MemberInvocation {
                name,
                on,
                parameters,
            } => format!("{}.{name}({})", on_to_ts(on), parameters_to_ts(parameters)),
            Self::Property { name, on } => format!("{}.{name}", on_to_ts(on)),
            Self::Indexing { on, idx } => {
                format!("{}[{}]", on_to_ts(on), idx.to_typescript())
            }
            Self::InfixOperation { left, op, right } => format!(
                "{}{}{}",
                left.to_typescript(),
                op.to_typescript(),
                right.to_typescript()
            ),
            Self::Object { name } => name.to_string(),
            Self::NewClass { name, parameters } => {
                format!("new {name}({})", parameters_to_ts(parameters))
            }
            Self::NewArray { items } => format!(
                "[{}]",
                items
                    .iter()
                    .map(ToTS::to_typescript)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::NewObject { items } => format!(
                "{{ {} }}",
                items
                    .iter()
                    .map(|item| match item {
                        NewObjectItem::Field(key, value) =>
                            format!("{key}: {}", value.to_typescript()),
                        NewObjectItem::Spread(expr) => format!("...{}", on_to_ts(expr)),
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::InlineFunctionDefinition {
                args,
                return_type,
                code,
            } => format!(
                "({}){} => {}",
                args.iter()
                    .map(|(assign_to, t)| format!(
                        "{}: {}",
                        assign_to.to_typescript(),
                        t.to_typescript()
                    ))
                    .collect::<Vec<String>>()
                    .join(", "),
                match return_type {
                    Some(rt) => format!(": {}", rt.to_typescript()),
                    None => String::new(),
                },
                code.to_typescript()
            ),
            Self::SpreadOperator { on } => format!("...{}", on_to_ts(on)),
            Self::As { inner, r#type } => {
                format!("{} as {}", on_to_ts(inner), r#type.to_typescript())
            }
            Self::ConstantString(s) => format!("\"{s}\""),
            Self::ConstantNumber(n) => n.to_string(),
            Self::ConstantBoolean(b) => b.to_string(),
            Self::Undefined => "undefined".to_string(),
            Self::RawStatement(s) => s.to_string(),
        }
    }
}

fn parameters_to_ts(parameters: &[ExpressionStatement]) -> String {
    parameters
        .iter()
        .map(ToTS::to_typescript)
        .collect::<Vec<String>>()
        .join(", ")
}

fn on_to_ts(on: &ExpressionStatement) -> String {
    match on {
        ExpressionStatement::InfixOperation {
            left: _,
            op: _,
            right: _,
        }
        | ExpressionStatement::ConstantNumber(_)
        | ExpressionStatement::As {
            inner: _,
            r#type: _,
        }
        | ExpressionStatement::RawStatement(_) => format!("({})", on.to_typescript()),
        _ => on.to_typescript(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            assign_to::AssignTo,
            code_block::CodeBlock,
            statement::{
                Statement,
                expression_statement::{
                    ExpressionStatement, InfixOperator, InlineFunctionBody, NewObjectItem,
                },
                return_statement::ReturnStatement,
            },
            r#type::Type,
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_function_invocation_to_typescript() {
        let statement = ExpressionStatement::FunctionInvocation {
            name: "my_func".to_string(),
            parameters: vec![
                ExpressionStatement::FunctionInvocation {
                    name: "inner_func".to_string(),
                    parameters: Vec::new(),
                },
                ExpressionStatement::ConstantNumber(12),
                ExpressionStatement::ConstantString("Hello, world!".to_string()),
            ],
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "my_func(inner_func(), 12, \"Hello, world!\")");
    }

    #[test]
    fn test_member_invocation_to_typescript() {
        let statement = ExpressionStatement::MemberInvocation {
            name: "log".to_string(),
            on: Box::new(ExpressionStatement::Object {
                name: "console".to_string(),
            }),
            parameters: vec![ExpressionStatement::ConstantString(
                "Hello, world!".to_string(),
            )],
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "console.log(\"Hello, world!\")");
    }

    #[test]
    fn test_property_to_typescript() {
        let statement = ExpressionStatement::Property {
            name: "toString".to_string(),
            on: Box::new(ExpressionStatement::InfixOperation {
                left: Box::new(ExpressionStatement::ConstantNumber(11)),
                op: InfixOperator::Add,
                right: Box::new(ExpressionStatement::ConstantNumber(14)),
            }),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "(11 + 14).toString");
    }

    #[test]
    fn test_indexing_to_typescript() {
        let statement = ExpressionStatement::Indexing {
            on: Box::new(ExpressionStatement::Object {
                name: "my_array".to_string(),
            }),
            idx: Box::new(ExpressionStatement::InfixOperation {
                left: Box::new(ExpressionStatement::ConstantNumber(12)),
                op: InfixOperator::Exponentiation,
                right: Box::new(ExpressionStatement::ConstantNumber(2)),
            }),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "my_array[12**2]");
    }

    #[test]
    fn test_infix_operation_to_typescript() {
        let statement = ExpressionStatement::InfixOperation {
            left: Box::new(ExpressionStatement::ConstantNumber(20)),
            op: InfixOperator::BooleanAnd,
            right: Box::new(ExpressionStatement::ConstantBoolean(false)),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "20 && false");
    }

    #[test]
    fn test_object_to_typescript() {
        let statement = ExpressionStatement::Object {
            name: "console".to_string(),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "console");
    }

    #[test]
    fn test_new_class_to_typescript() {
        let statement = ExpressionStatement::NewClass {
            name: "MyClass".to_string(),
            parameters: vec![ExpressionStatement::ConstantNumber(12)],
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "new MyClass(12)");
    }

    #[test]
    fn test_new_array_to_typescript() {
        let statement = ExpressionStatement::NewArray {
            items: vec![
                ExpressionStatement::Object {
                    name: "a".to_string(),
                },
                ExpressionStatement::ConstantNumber(12),
            ],
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "[a, 12]");
    }

    #[test]
    fn test_new_object_to_typescript() {
        let statement = ExpressionStatement::NewObject {
            items: vec![
                NewObjectItem::Spread(ExpressionStatement::Object {
                    name: "otherObject".to_string(),
                }),
                NewObjectItem::Field(
                    "message".to_string(),
                    ExpressionStatement::ConstantString("hello, world!".to_string()),
                ),
                NewObjectItem::Field("code".to_string(), ExpressionStatement::ConstantNumber(32)),
            ],
        };

        let actual = statement.to_typescript();

        assert_eq!(
            actual,
            "{ ...otherObject, message: \"hello, world!\", code: 32 }"
        );
    }

    #[test]
    fn test_empty_inline_function_definition_to_typescript() {
        let expression = ExpressionStatement::InlineFunctionDefinition {
            args: Vec::new(),
            return_type: None,
            code: InlineFunctionBody::CodeBlock(CodeBlock(Vec::new())),
        };

        let actual = expression.to_typescript();

        assert_eq!(actual, "() => {  }")
    }

    #[test]
    fn test_simple_return_inline_function_definition_to_typescript() {
        let expression = ExpressionStatement::InlineFunctionDefinition {
            args: Vec::new(),
            return_type: None,
            code: InlineFunctionBody::ExpressionStatement(Box::new(ExpressionStatement::Undefined)),
        };

        let actual = expression.to_typescript();

        assert_eq!(actual, "() => undefined")
    }

    #[test]
    fn test_elaborate_inline_function_definition_to_typescript() {
        let expression = ExpressionStatement::InlineFunctionDefinition {
            args: vec![(AssignTo::Ident("a".to_string()), Type::String)],
            return_type: Some(Type::Boolean),
            code: InlineFunctionBody::CodeBlock(CodeBlock(vec![Statement::ReturnStatement(
                ReturnStatement {
                    value: ExpressionStatement::InfixOperation {
                        left: Box::new(ExpressionStatement::Object {
                            name: "a".to_string(),
                        }),
                        op: InfixOperator::StrictEqual,
                        right: Box::new(ExpressionStatement::ConstantString(
                            "hello there".to_string(),
                        )),
                    },
                },
            )])),
        };

        let actual = expression.to_typescript();

        assert_eq!(
            actual,
            "(a: string): boolean => { return a === \"hello there\"; }"
        )
    }

    #[test]
    fn test_spread_operator_to_typescript_01() {
        let statement = ExpressionStatement::SpreadOperator {
            on: Box::new(ExpressionStatement::NewArray {
                items: vec![
                    ExpressionStatement::ConstantNumber(12),
                    ExpressionStatement::ConstantBoolean(false),
                ],
            }),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "...[12, false]")
    }

    #[test]
    fn test_spread_operator_to_typescript_02() {
        let statement = ExpressionStatement::SpreadOperator {
            on: Box::new(ExpressionStatement::As {
                inner: Box::new(ExpressionStatement::ConstantBoolean(false)),
                r#type: Type::String,
            }),
        };

        let actual = statement.to_typescript();

        // `false as string` is not valid TS, but good enough for the
        // test case.
        assert_eq!(actual, "...(false as string)")
    }

    #[test]
    fn test_constant_string_to_typescript() {
        let statement = ExpressionStatement::ConstantString("Hello, world!".to_string());

        let actual = statement.to_typescript();

        assert_eq!(actual, "\"Hello, world!\"");
    }

    #[test]
    fn test_constant_number_to_typescript() {
        let statement = ExpressionStatement::ConstantNumber(12);

        let actual = statement.to_typescript();

        assert_eq!(actual, "12");
    }

    #[test]
    fn test_constant_boolean_to_typescript() {
        let statement = ExpressionStatement::ConstantBoolean(false);

        let actual = statement.to_typescript();

        assert_eq!(actual, "false");
    }

    #[test]
    fn test_undefined_to_typescript() {
        let statement = ExpressionStatement::Undefined;

        let actual = statement.to_typescript();

        assert_eq!(actual, "undefined");
    }

    #[test]
    fn test_raw_statement_to_typescript() {
        let statement = ExpressionStatement::RawStatement("MyObj.my_func()[0]".to_string());

        let actual = statement.to_typescript();

        assert_eq!(actual, "MyObj.my_func()[0]");
    }
}
