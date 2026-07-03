use crate::{
    ast::{Generic, Visibility, assign_to::AssignTo, code_block::CodeBlock, r#type::Type},
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct ClassMethod {
    pub visibility: Visibility,
    pub r#static: bool,
    pub ident: String,
    pub generics: Vec<Generic>,
    pub args: Vec<(AssignTo, Type)>,
    pub return_type: Option<Type>,
    pub body: CodeBlock,
}

impl ToTS for ClassMethod {
    fn to_typescript(&self) -> String {
        format!(
            "{}{}{}{}({}){} {{ {} }}",
            self.visibility.to_typescript(),
            if self.r#static { " static " } else { " " },
            self.ident,
            self.generics.to_typescript(),
            self.args
                .iter()
                .map(|(assign_to, t)| format!(
                    "{}: {}",
                    assign_to.to_typescript(),
                    t.to_typescript()
                ))
                .collect::<Vec<String>>()
                .join(", "),
            match &self.return_type {
                Some(rt) => format!(": {}", rt.to_typescript()),
                None => String::new(),
            },
            self.body.to_typescript(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            Generic, Visibility, assign_to::AssignTo, class::class_method::ClassMethod,
            code_block::CodeBlock, r#type::Type,
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_static_constructor_to_typescript() {
        let method = ClassMethod {
            visibility: Visibility::Public,
            r#static: true,
            ident: "A".to_string(),
            generics: Vec::new(),
            args: vec![],
            return_type: None,
            body: CodeBlock(Vec::new()),
        };

        let actual = method.to_typescript();

        assert_eq!(actual, "public static A() {  }");
    }

    #[test]
    fn test_arguments_to_typescript() {
        let method = ClassMethod {
            visibility: Visibility::Public,
            r#static: false,
            ident: "A".to_string(),
            generics: vec![Generic("T".to_string())],
            args: vec![
                (AssignTo::Ident("arg0".to_string()), Type::String),
                (
                    AssignTo::UnpackArray(vec![AssignTo::Ident("a".to_string())]),
                    Type::Tuple(vec![Type::String]),
                ),
                (
                    AssignTo::UnpackObject(vec![(
                        "b".to_string(),
                        AssignTo::Ident("b".to_string()),
                    )]),
                    Type::Object(vec![(
                        "b".to_string(),
                        true,
                        Type::Generic(Generic("T".to_string())),
                    )]),
                ),
            ],
            return_type: None,
            body: CodeBlock(Vec::new()),
        };

        let actual = method.to_typescript();

        assert_eq!(
            actual,
            "public A<T>(arg0: string, [a]: [string], { b }: { b: T }) {  }"
        );
    }
}
