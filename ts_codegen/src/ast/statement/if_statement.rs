use crate::{
    ast::{code_block::CodeBlock, statement::expression_statement::ExpressionStatement},
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct IfStatement {
    pub if_branch: (ExpressionStatement, CodeBlock),
    pub if_else_branches: Vec<(ExpressionStatement, CodeBlock)>,
    pub else_branch: Option<CodeBlock>,
}

impl ToTS for IfStatement {
    fn to_typescript(&self) -> String {
        let mut branches = vec![format!(
            "if ({}) {{ {} }}",
            self.if_branch.0.to_typescript(),
            self.if_branch.1.to_typescript()
        )];

        let mut if_else_branches_str = self
            .if_else_branches
            .iter()
            .map(|(expression, codeblock)| {
                format!(
                    "else if ({}) {{ {} }}",
                    expression.to_typescript(),
                    codeblock.to_typescript()
                )
            })
            .collect::<Vec<String>>();

        branches.append(&mut if_else_branches_str);

        if let Some(else_branch) = &self.else_branch {
            branches.push(format!("else {{ {} }}", else_branch.to_typescript()))
        }

        branches.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            code_block::CodeBlock,
            statement::{
                Statement, expression_statement::ExpressionStatement, if_statement::IfStatement,
            },
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_single_if_statement_to_typescript() {
        let statement = IfStatement {
            if_branch: (
                ExpressionStatement::RawStatement("a || b".to_string()),
                CodeBlock(Vec::new()),
            ),
            if_else_branches: vec![],
            else_branch: None,
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "if (a || b) {  }")
    }

    #[test]
    fn test_single_if_else_statement_to_typescript() {
        let statement = IfStatement {
            if_branch: (
                ExpressionStatement::RawStatement("a && b".to_string()),
                CodeBlock(Vec::new()),
            ),
            if_else_branches: vec![(
                ExpressionStatement::RawStatement("a || b".to_string()),
                CodeBlock(vec![Statement::Expression(
                    ExpressionStatement::RawStatement("console.log(a)".to_string()),
                )]),
            )],
            else_branch: None,
        };

        let actual = statement.to_typescript();

        assert_eq!(
            actual,
            "if (a && b) {  } else if (a || b) { console.log(a); }"
        )
    }

    #[test]
    fn test_multiple_if_else_statement_to_typescript() {
        let statement = IfStatement {
            if_branch: (
                ExpressionStatement::RawStatement("a && b".to_string()),
                CodeBlock(Vec::new()),
            ),
            if_else_branches: vec![
                (
                    ExpressionStatement::RawStatement("a || b".to_string()),
                    CodeBlock(vec![Statement::Expression(
                        ExpressionStatement::RawStatement("console.log(a)".to_string()),
                    )]),
                ),
                (
                    ExpressionStatement::RawStatement("c".to_string()),
                    CodeBlock(vec![Statement::Expression(
                        ExpressionStatement::RawStatement("console.log(c)".to_string()),
                    )]),
                ),
            ],
            else_branch: None,
        };

        let actual = statement.to_typescript();

        assert_eq!(
            actual,
            "if (a && b) {  } else if (a || b) { console.log(a); } else if (c) { console.log(c); }"
        )
    }

    #[test]
    fn test_else_branch_to_typescript() {
        let statement = IfStatement {
            if_branch: (
                ExpressionStatement::RawStatement("a && b".to_string()),
                CodeBlock(Vec::new()),
            ),
            if_else_branches: vec![],
            else_branch: Some(CodeBlock(vec![Statement::Expression(
                ExpressionStatement::RawStatement("console.log(a)".to_string()),
            )])),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "if (a && b) {  } else { console.log(a); }")
    }
}
