use crate::{
    ast::{
        code_block::CodeBlock,
        statement::{
            assignment_statement::AssignmentStatement, expression_statement::ExpressionStatement,
        },
    },
    to_ts::ToTS,
};

#[derive(Debug)]
pub struct ForStatement {
    pub assignment: AssignmentStatement,
    pub conditional: ExpressionStatement,
    pub increment: ExpressionStatement,
    pub codeblock: CodeBlock,
}

impl ToTS for ForStatement {
    fn to_typescript(&self) -> String {
        format!(
            "for ({}; {}; {}) {{ {} }}",
            self.assignment.to_typescript(),
            self.conditional.to_typescript(),
            self.increment.to_typescript(),
            self.codeblock.to_typescript(),
        )
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
                assignment_statement::{AssignmentStatement, AssignmentType},
                expression_statement::ExpressionStatement,
                for_statement::ForStatement,
            },
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_empty_for_statement_to_typescript() {
        let statement = ForStatement {
            assignment: AssignmentStatement {
                kind: AssignmentType::Let,
                left_hand_side: AssignTo::Ident("i".to_string()),
                right_hand_side: ExpressionStatement::RawStatement("0".to_string()),
            },
            conditional: ExpressionStatement::RawStatement("i < 10".to_string()),
            increment: ExpressionStatement::RawStatement("i++".to_string()),
            codeblock: CodeBlock(Vec::new()),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "for (let i = 0; i < 10; i++) {  }");
    }

    #[test]
    fn test_for_statement_with_codeblock_to_typescript() {
        let statement = ForStatement {
            assignment: AssignmentStatement {
                kind: AssignmentType::Let,
                left_hand_side: AssignTo::Ident("i".to_string()),
                right_hand_side: ExpressionStatement::RawStatement("0".to_string()),
            },
            conditional: ExpressionStatement::RawStatement("i < 10".to_string()),
            increment: ExpressionStatement::RawStatement("i++".to_string()),
            codeblock: CodeBlock(vec![Statement::Expression(
                ExpressionStatement::RawStatement("console.log(\"Hello, world!\")".to_string()),
            )]),
        };

        let actual = statement.to_typescript();

        assert_eq!(
            actual,
            "for (let i = 0; i < 10; i++) { console.log(\"Hello, world!\"); }"
        );
    }
}
