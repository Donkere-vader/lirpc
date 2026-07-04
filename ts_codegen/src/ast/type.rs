use std::iter::once;

use crate::{
    ast::{Generic, assign_to::AssignTo},
    to_ts::ToTS,
};

#[derive(Debug)]
pub enum Type {
    SpecificString(String),
    SpecificNumber(i128),
    String,
    Number,
    Boolean,
    Undefined,
    Null,
    Generic(Generic),
    Union(Box<Type>, Vec<Type>),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    /// An object consisting of different `(key, required, type)`
    Object(Vec<(String, bool, Type)>),
    /// A reference to another type defined by the user
    TypeRef(String),
    Function {
        args: Vec<(AssignTo, Type)>,
        return_type: Box<Type>,
    },
    Class(String, Vec<Type>),
}

impl ToTS for Type {
    fn to_typescript(&self) -> String {
        match self {
            Self::SpecificString(str) => format!("\"{str}\""),
            Self::SpecificNumber(num) => format!("{num}"),
            Self::String => "string".to_string(),
            Self::Number => "number".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::Undefined => "undefined".to_string(),
            Self::Null => "null".to_string(),
            Self::Generic(g) => g.to_typescript(),
            Self::Union(t, ts) => once(&**t)
                .chain(ts.iter())
                .map(parenthesised)
                .collect::<Vec<String>>()
                .join(" | "),
            Self::Array(t) => match **t {
                Self::Union(_, _) => format!("({})[]", t.to_typescript()),
                _ => format!("{}[]", t.to_typescript()),
            },
            Self::Tuple(ts) => format!(
                "[{}]",
                ts.iter()
                    .map(ToTS::to_typescript)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Object(fields) => format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(ident, required, t)| format!(
                        "{ident}{}: {}",
                        if !required { "?" } else { "" },
                        t.to_typescript()
                    ))
                    .collect::<Vec<String>>()
                    .join("; ")
            ),
            Self::TypeRef(ident) => ident.to_string(),
            Self::Function { args, return_type } => {
                format!(
                    "({}) => {}",
                    args.iter()
                        .map(|(assign_to, t)| format!(
                            "{}: {}",
                            assign_to.to_typescript(),
                            t.to_typescript()
                        ))
                        .collect::<Vec<String>>()
                        .join(", "),
                    return_type.to_typescript(),
                )
            }
            Self::Class(name, generic_types) => format!(
                "{}{}",
                name,
                if !generic_types.is_empty() {
                    format!(
                        "<{}>",
                        generic_types
                            .iter()
                            .map(ToTS::to_typescript)
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                } else {
                    "".to_string()
                }
            ),
        }
    }
}

fn parenthesised(t: &Type) -> String {
    match t {
        Type::Union(_, _)
        | Type::Function {
            args: _,
            return_type: _,
        } => format!("({})", t.to_typescript()),
        _ => t.to_typescript(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Generic, assign_to::AssignTo, r#type::Type},
        to_ts::ToTS,
    };

    #[test]
    fn test_specific_string_to_typescript() {
        let t = Type::SpecificString("Hello, world!".to_string());

        let actual = t.to_typescript();

        assert_eq!(actual, r#""Hello, world!""#);
    }

    #[test]
    fn test_specific_number_to_typescript() {
        let t = Type::SpecificNumber(124);

        let actual = t.to_typescript();

        assert_eq!(actual, r#"124"#);
    }

    #[test]
    fn test_string_to_typescript() {
        let t = Type::String;

        let actual = t.to_typescript();

        assert_eq!(actual, r#"string"#);
    }

    #[test]
    fn test_number_to_typescript() {
        let t = Type::Number;

        let actual = t.to_typescript();

        assert_eq!(actual, r#"number"#);
    }

    #[test]
    fn test_boolean_to_typescript() {
        let t = Type::Boolean;

        let actual = t.to_typescript();

        assert_eq!(actual, r#"boolean"#);
    }

    #[test]
    fn test_undefined_to_typescript() {
        let t = Type::Undefined;

        let actual = t.to_typescript();

        assert_eq!(actual, r#"undefined"#);
    }

    #[test]
    fn test_null_to_typescript() {
        let t = Type::Null;

        let actual = t.to_typescript();

        assert_eq!(actual, r#"null"#);
    }

    #[test]
    fn test_union_to_typescript() {
        let t = Type::Union(Box::new(Type::String), vec![Type::Number]);

        let actual = t.to_typescript();

        assert_eq!(actual, r#"string | number"#);
    }

    #[test]
    fn test_array_to_typescript() {
        let t = Type::Array(Box::new(Type::String));

        let actual = t.to_typescript();

        assert_eq!(actual, r#"string[]"#);
    }

    #[test]
    fn test_array_of_union_to_typescript() {
        let t = Type::Array(Box::new(Type::Union(
            Box::new(Type::String),
            vec![Type::Number],
        )));

        let actual = t.to_typescript();

        assert_eq!(actual, r#"(string | number)[]"#);
    }

    #[test]
    fn test_tuple_to_typescript() {
        let t = Type::Tuple(vec![Type::String, Type::Number]);

        let actual = t.to_typescript();

        assert_eq!(actual, r#"[string, number]"#);
    }

    #[test]
    fn test_object_to_typescript() {
        let t = Type::Object(vec![
            ("field0".to_string(), true, Type::String),
            ("field1".to_string(), false, Type::Number),
        ]);

        let actual = t.to_typescript();

        assert_eq!(actual, r#"{ field0: string; field1?: number }"#);
    }

    #[test]
    fn test_function_to_typescript() {
        let t = Type::Function {
            args: vec![
                (
                    AssignTo::UnpackObject(vec![
                        ("a".to_string(), AssignTo::Ident("a".to_string())),
                        ("b".to_string(), AssignTo::Ident("b".to_string())),
                    ]),
                    Type::Object(vec![
                        (
                            "a".to_string(),
                            true,
                            Type::Function {
                                args: vec![],
                                return_type: Box::new(Type::String),
                            },
                        ),
                        ("b".to_string(), false, Type::Number),
                    ]),
                ),
                (
                    AssignTo::Ident("other".to_string()),
                    Type::SpecificString("test".to_string()),
                ),
            ],
            return_type: Box::new(Type::String),
        };

        let actual = t.to_typescript();

        assert_eq!(
            actual,
            r#"({ a, b }: { a: () => string; b?: number }, other: "test") => string"#
        );
    }

    #[test]
    fn test_class_to_typescript() {
        let t = Type::Class("MyClass".to_string(), Vec::new());

        let actual = t.to_typescript();

        assert_eq!(actual, r#"MyClass"#);
    }

    #[test]
    fn test_class_with_generics_to_typescript() {
        let t = Type::Class(
            "MyClass".to_string(),
            vec![Type::String, Type::Generic(Generic("T".to_string()))],
        );

        let actual = t.to_typescript();

        assert_eq!(actual, r#"MyClass<string, T>"#);
    }
}
