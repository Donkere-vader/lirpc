use crate::{
    ast::{Generic, r#type::Type},
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct TypeDefinitionStatement {
    pub name: String,
    pub r#type: Type,
    pub generics: Vec<Generic>,
}

impl ToTS for TypeDefinitionStatement {
    fn to_typescript(&self) -> String {
        format!(
            "type {}{} = {}",
            self.name,
            self.generics.to_typescript(),
            self.r#type.to_typescript(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Generic, statement::type_definition::TypeDefinitionStatement, r#type::Type},
        to_ts::ToTS,
    };

    #[test]
    fn basic_type_definition_statement_to_typescript() {
        let statement = TypeDefinitionStatement {
            name: "MyType".to_string(),
            r#type: Type::String,
            generics: Vec::new(),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "type MyType = string")
    }

    #[test]
    fn type_definition_statement_with_generics_to_typescript() {
        let statement = TypeDefinitionStatement {
            name: "MyType".to_string(),
            r#type: Type::Object(vec![
                (
                    "abc".to_string(),
                    true,
                    Type::Generic(Generic("T".to_string())),
                ),
                (
                    "def".to_string(),
                    false,
                    Type::Generic(Generic("T0".to_string())),
                ),
            ]),
            generics: vec![Generic("T".to_string()), Generic("T0".to_string())],
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "type MyType<T, T0> = { abc: T; def?: T0 }")
    }
}
