#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr{
    And(Box<Expr>, Box <Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    Cmp { field: String, op: Op, value: String},


}
pub enum Op {
    Eq,
    Ne, 
    Contains,
    StartsWith,

}
pub fn expr_parse(io: &str) -> Result<Expr, String> {
    let mut parser = Parser {;

}
