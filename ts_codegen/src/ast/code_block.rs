use crate::{ast::statement::Statement, to_ts::ToTS};

#[derive(Debug)]
pub struct CodeBlock(pub Vec<Statement>);

impl ToTS for CodeBlock {
    fn to_typescript(&self) -> String {
        self.0
            .iter()
            .map(ToTS::to_typescript)
            .map(|s| s + ";")
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            code_block::CodeBlock,
            statement::{Statement, expression_statement::ExpressionStatement},
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_empty_codeblock_statement_to_typescript() {
        let code_block = CodeBlock(Vec::new());

        let actual = code_block.to_typescript();

        assert_eq!(actual, "");
    }

    #[test]
    fn test_codeblock_with_one_statement_to_typescript() {
        let code_block = CodeBlock(vec![Statement::Expression(
            ExpressionStatement::RawStatement("console.log(1)".to_string()),
        )]);

        let actual = code_block.to_typescript();

        assert_eq!(actual, "console.log(1);");
    }
}
