use crate::{
    ast::{code_block::CodeBlock, statement::expression_statement::ExpressionStatement},
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct SwitchCaseStatement {
    pub expression: ExpressionStatement,
    pub cases: Vec<(ExpressionStatement, CodeBlock)>,
    pub default: Option<CodeBlock>,
}

impl ToTS for SwitchCaseStatement {
    fn to_typescript(&self) -> String {
        let mut cases_ts = self
            .cases
            .iter()
            .map(|(e, c)| format!("case {}: {}", e.to_typescript(), c.to_typescript()))
            .collect::<Vec<String>>()
            .join(" ");

        if let Some(default) = &self.default {
            cases_ts += &format!(" default: {}", default.to_typescript());
        }

        format!(
            "switch ({}) {{ {} }}",
            self.expression.to_typescript(),
            cases_ts
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
                switch_case_statement::SwitchCaseStatement,
            },
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_empty_switch_case_to_typescript() {
        let statement = SwitchCaseStatement {
            expression: ExpressionStatement::RawStatement("some_var".to_string()),
            cases: Vec::new(),
            default: None,
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "switch (some_var) {  }")
    }

    #[test]
    fn test_switch_case_with_cases_to_typescript() {
        let statement = SwitchCaseStatement {
            expression: ExpressionStatement::RawStatement("some_var".to_string()),
            cases: vec![
                (
                    ExpressionStatement::RawStatement("0".to_string()),
                    CodeBlock(vec![
                        Statement::Expression(ExpressionStatement::RawStatement(
                            "console.log(\"0\")".to_string(),
                        )),
                        Statement::BreakStatement,
                    ]),
                ),
                (
                    ExpressionStatement::RawStatement("1".to_string()),
                    CodeBlock(vec![
                        Statement::Expression(ExpressionStatement::RawStatement(
                            "console.log(\"1\")".to_string(),
                        )),
                        Statement::BreakStatement,
                    ]),
                ),
            ],
            default: None,
        };

        let actual = statement.to_typescript();

        assert_eq!(
            actual,
            "switch (some_var) { case 0: console.log(\"0\"); break; case 1: console.log(\"1\"); break; }"
        )
    }

    #[test]
    fn test_switch_case_with_cases_and_default_to_typescript() {
        let statement = SwitchCaseStatement {
            expression: ExpressionStatement::Object {
                name: "some_var".to_string(),
            },
            cases: vec![
                (
                    ExpressionStatement::ConstantNumber(0),
                    CodeBlock(vec![
                        Statement::Expression(ExpressionStatement::RawStatement(
                            "console.log(\"0\")".to_string(),
                        )),
                        Statement::BreakStatement,
                    ]),
                ),
                (
                    ExpressionStatement::RawStatement("1".to_string()),
                    CodeBlock(vec![
                        Statement::Expression(ExpressionStatement::RawStatement(
                            "console.log(\"1\")".to_string(),
                        )),
                        Statement::BreakStatement,
                    ]),
                ),
            ],
            default: Some(CodeBlock(vec![
                Statement::Expression(ExpressionStatement::RawStatement(
                    "console.log(\"default\")".to_string(),
                )),
                Statement::BreakStatement,
            ])),
        };

        let actual = statement.to_typescript();

        assert_eq!(
            actual,
            "switch (some_var) { case 0: console.log(\"0\"); break; case 1: console.log(\"1\"); break; default: console.log(\"default\"); break; }"
        )
    }
}
