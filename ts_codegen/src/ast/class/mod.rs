pub mod class_member;
pub mod class_method;

use crate::{
    ast::{
        Generic,
        class::{class_member::ClassMember, class_method::ClassMethod},
    },
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct ClassDefinition {
    pub ident: String,
    pub generics: Vec<Generic>,
    pub class_members: Vec<ClassMember>,
    pub methods: Vec<ClassMethod>,
}

impl ToTS for ClassDefinition {
    fn to_typescript(&self) -> String {
        let members = self
            .class_members
            .iter()
            .map(ToTS::to_typescript)
            .map(|c| c + ";")
            .collect::<Vec<String>>()
            .join(" ");

        let methods = &self
            .methods
            .iter()
            .map(ToTS::to_typescript)
            .collect::<Vec<String>>()
            .join(" ");

        format!(
            "class {}{} {{ {members} {methods} }}",
            self.ident,
            self.generics.to_typescript()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            Generic, Visibility,
            assign_to::AssignTo,
            class::{ClassDefinition, class_member::ClassMember, class_method::ClassMethod},
            code_block::CodeBlock,
            statement::{
                Statement,
                assignment_statement::{AssignmentStatement, AssignmentType},
                expression_statement::ExpressionStatement,
                return_statement::ReturnStatement,
            },
            r#type::Type,
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_empty_class_definition_to_typescript() {
        let definition = ClassDefinition {
            ident: "MyClass".to_string(),
            generics: Vec::new(),
            class_members: Vec::new(),
            methods: Vec::new(),
        };

        let actual = definition.to_typescript();

        assert_eq!(actual, "class MyClass {   }");
    }

    #[test]
    fn test_class_with_members_definition_to_typescript() {
        let definition = ClassDefinition {
            ident: "MyClass".to_string(),
            generics: Vec::new(),
            class_members: vec![
                ClassMember {
                    visibility: Visibility::Private,
                    ident: "some_member".to_string(),
                    r#type: Type::String,
                    r#static: false,
                },
                ClassMember {
                    visibility: Visibility::Public,
                    ident: "some_other_member".to_string(),
                    r#type: Type::Number,
                    r#static: false,
                },
            ],
            methods: Vec::new(),
        };

        let actual = definition.to_typescript();

        assert_eq!(
            actual,
            "class MyClass { private some_member: string; public some_other_member: number;  }"
        );
    }

    #[test]
    fn test_class_with_members_and_methods_definition_to_typescript() {
        let definition = ClassDefinition {
            ident: "MyClass".to_string(),
            generics: Vec::new(),
            class_members: vec![ClassMember {
                visibility: Visibility::Private,
                ident: "some_member".to_string(),
                r#type: Type::String,
                r#static: false,
            }],
            methods: vec![ClassMethod {
                visibility: Visibility::Public,
                r#static: false,
                ident: "constructor".to_string(),
                generics: Vec::new(),
                args: vec![(AssignTo::Ident("some_member".to_string()), Type::String)],
                return_type: None,
                body: CodeBlock(vec![Statement::Assignment(AssignmentStatement {
                    kind: AssignmentType::Existing,
                    left_hand_side: AssignTo::Ident("this.some_member".to_string()),
                    right_hand_side: ExpressionStatement::Object {
                        name: "some_member".to_string(),
                    },
                })]),
            }],
        };

        let actual = definition.to_typescript();

        assert_eq!(
            actual,
            "class MyClass { private some_member: string; public constructor(some_member: string) { this.some_member = some_member; } }"
        );
    }

    #[test]
    fn test_class_with_members_methods_and_generics_definition_to_typescript() {
        let definition = ClassDefinition {
            ident: "MyClass".to_string(),
            generics: vec![Generic("T".to_string())],
            class_members: vec![ClassMember {
                visibility: Visibility::Private,
                ident: "some_member".to_string(),
                r#type: Type::Generic(Generic("T".to_string())),
                r#static: false,
            }],
            methods: vec![ClassMethod {
                visibility: Visibility::Public,
                r#static: true,
                ident: "new".to_string(),
                generics: vec![Generic("T".to_string())],
                args: vec![(
                    AssignTo::Ident("some_member".to_string()),
                    Type::Generic(Generic("T".to_string())),
                )],
                return_type: None,
                body: CodeBlock(vec![Statement::ReturnStatement(ReturnStatement {
                    value: ExpressionStatement::NewClass {
                        name: "MyClass".to_string(),
                        parameters: vec![ExpressionStatement::Object {
                            name: "some_member".to_string(),
                        }],
                    },
                })]),
            }],
        };

        let actual = definition.to_typescript();

        assert_eq!(
            actual,
            "class MyClass<T> { private some_member: T; public static new<T>(some_member: T) { return new MyClass(some_member); } }"
        );
    }
}
