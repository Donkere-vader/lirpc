use crate::{
    ast::{code_block::CodeBlock, statement::expression_statement::ExpressionStatement},
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct WhileStatement {
    pub condition: ExpressionStatement,
    pub codeblock: CodeBlock,
}

impl ToTS for WhileStatement {
    fn to_typescript(&self) -> String {
        format!(
            "while ({}) {{ {} }}",
            self.condition.to_typescript(),
            self.codeblock.to_typescript()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            code_block::CodeBlock,
            statement::{
                Statement, expression_statement::ExpressionStatement,
                while_statement::WhileStatement,
            },
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_empty_while_statement_to_typescript() {
        let statement = WhileStatement {
            condition: ExpressionStatement::RawStatement("a || b".to_string()),
            codeblock: CodeBlock(Vec::new()),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "while (a || b) {  }");
    }

    #[test]
    fn test_while_statement_with_code_to_typescript() {
        let statement = WhileStatement {
            condition: ExpressionStatement::RawStatement("a || b".to_string()),
            codeblock: CodeBlock(vec![Statement::Expression(
                ExpressionStatement::RawStatement("console.log(a)".to_string()),
            )]),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "while (a || b) { console.log(a); }");
    }
}
