use crate::{ast::statement::Statement, to_ts::ToTS};

#[derive(Debug)]
pub struct ExportStatement {
    pub default: bool,
    pub inner: Box<Statement>,
}

impl ToTS for ExportStatement {
    fn to_typescript(&self) -> String {
        format!(
            "export {}{}",
            if self.default { "default " } else { "" },
            self.inner.to_typescript()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            class::ClassDefinition,
            statement::{Statement, export_statement::ExportStatement},
        },
        to_ts::ToTS,
    };

    #[test]
    fn export_default_to_typescript() {
        let statement = ExportStatement {
            default: true,
            inner: Box::new(Statement::ClassDefinition(ClassDefinition {
                ident: "MyClass".to_string(),
                generics: vec![],
                class_members: vec![],
                methods: vec![],
            })),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "export default class MyClass {   }");
    }
}
