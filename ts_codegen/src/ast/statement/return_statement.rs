use crate::{ast::statement::expression_statement::ExpressionStatement, to_ts::ToTS};

#[derive(Debug)]
pub struct ReturnStatement {
    pub value: ExpressionStatement,
}

impl ToTS for ReturnStatement {
    fn to_typescript(&self) -> String {
        format!("return {}", self.value.to_typescript())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::statement::{
            expression_statement::ExpressionStatement, return_statement::ReturnStatement,
        },
        to_ts::ToTS,
    };

    #[test]
    fn test_return_statement_to_typescript() {
        let statement = ReturnStatement {
            value: ExpressionStatement::ConstantNumber(1),
        };

        let actual = statement.to_typescript();

        assert_eq!(actual, "return 1")
    }
}
