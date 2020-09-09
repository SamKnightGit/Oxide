use std::collections::HashSet;
use std::fmt;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
    token_index: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse Error at token {0}: {1}", self.token_index, self.message)
    }
}

lazy_static! {
    static ref REDIRECTION_OPS: HashSet<&'static str> = 
        [
            ">", ">>", "<", "|"
        ].iter().cloned().collect();
}


#[derive(Clone, Debug, PartialEq)]
pub enum RedirectionOp {
    Output,
    Append,
    Input,
}

#[derive(Debug, PartialEq)]
pub enum ParseNodeType {
    Expr,
    CommandExpr,
    RedirectionPipeExpr,
    PipeExpr,
    RedirectionExpr,
    Command(String),
    File(String),
    RedirectionOp(RedirectionOp),
    Pipe,
}

#[derive(Debug)]
pub struct ParseNode {
    pub entry: ParseNodeType,
    pub children: Option<Vec<ParseNode>>,
} 



pub fn parse_input(input_tokens: &Vec<&str>) -> Result<ParseNode>
{
    let mut input_index: usize = 0; 
    return parse_expr(input_tokens, &mut input_index);
}

fn parse_expr(input_tokens: &Vec<&str>, input_index: &mut usize) -> Result<ParseNode>
{
    let mut parse_tree = ParseNode {
        entry: ParseNodeType::Expr,
        children: Some(Vec::new()),
    };
    
    parse_command_expr(input_tokens, input_index, &mut parse_tree)?;
    
    parse_redirection_pipe_expr(input_tokens, input_index, &mut parse_tree)?;
     
    return Ok(parse_tree)
}

fn parse_command_expr(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{
    let mut command_expr_node = ParseNode {
        entry: ParseNodeType::CommandExpr,
        children: Some(Vec::new()),
    };
     
    parse_command(input_tokens, input_index, &mut command_expr_node)?;
    
    parse_filelist(input_tokens, input_index, &mut command_expr_node)?;
        
    tree_node.children.as_mut().unwrap().push(command_expr_node);
    
    return Ok(())
}

fn parse_redirection_pipe_expr(input_tokens: &Vec<&str>, input_index: &mut usize, mut tree_node: &mut ParseNode) -> Result<()>
{
    if *input_index == input_tokens.len() 
    {
        return Ok(())
    }
    
    if input_tokens[*input_index] != "|"
    {
        parse_redirection_expr(input_tokens, input_index, &mut tree_node)?;
    }

    parse_pipe_expr(input_tokens, input_index, &mut tree_node)?;
    
    return Ok(())
}

fn parse_pipe_expr(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{
    if *input_index == input_tokens.len() 
    {
        return Ok(())
    }

    let token = input_tokens[*input_index];
    if token != "|"
    {
        return Err(ParseError {
            message: format!("Expected '|' to continue piping. Got '{0}' instead.", token),
            token_index: *input_index,
        })
    }

    let mut pipe_expr_node = ParseNode {
        entry: ParseNodeType::PipeExpr,
        children: Some(Vec::new()),
    };

    let pipe_node = ParseNode {
        entry: ParseNodeType::Pipe,
        children: None,
    };

    pipe_expr_node.children.as_mut().unwrap().push(pipe_node);
    *input_index += 1;

    parse_command_expr(input_tokens, input_index, &mut pipe_expr_node)?;

    parse_redirection_pipe_expr(input_tokens, input_index, &mut pipe_expr_node)?;

    tree_node.children.as_mut().unwrap().push(pipe_expr_node);

    return Ok(())
}

fn parse_redirection_expr(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{ 
    if *input_index == input_tokens.len() 
    {
        return Ok(())
    }
   
    let mut redirection_expr_node = ParseNode {
        entry: ParseNodeType::RedirectionExpr,
        children: Some(Vec::new()),
    };

    parse_redirection_op(input_tokens, input_index, &mut redirection_expr_node)?;
        
    parse_filelist(input_tokens, input_index, &mut redirection_expr_node)?; 
    
    tree_node.children.as_mut().unwrap().push(redirection_expr_node);
    
    return Ok(())
}
    
fn parse_command(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{   
    if *input_index == input_tokens.len() 
    {
        return Err(ParseError {
            message: format!(
                         "reached end of input at token '{0}' but expected command.", 
                         input_tokens[*input_index - 1]
                     ),
            token_index: *input_index,
        })
    }

    let command = input_tokens[*input_index];

    // Add Expr to AST node 
    let command_node = ParseNode {
        entry: ParseNodeType::Command(command.to_string()),
        children: None,
    };
    tree_node.children.as_mut().unwrap().push(command_node);
    *input_index += 1;
    return Ok(())
}

fn parse_filelist(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{
    // epsilon rule if all tokens have been parsed or not filename
    if *input_index == input_tokens.len()
    {
        return Ok(())
    }
    
    let token = input_tokens[*input_index];
    // Keep adding files until we hit a redirection operator
    if !REDIRECTION_OPS.contains(token)
    {
        // Add token as file to syntax tree
        let file_node = ParseNode {
            entry: ParseNodeType::File(token.to_string()),
            children: None,
        };
        tree_node.children.as_mut().unwrap().push(file_node);
        *input_index += 1;
        parse_filelist(input_tokens, input_index, tree_node)?;
    }
    return Ok(())
}

fn parse_redirection_op(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{
    let token = input_tokens[*input_index];
    
    let redirection_op_token = match token {
        ">"  => RedirectionOp::Output,
        ">>" => RedirectionOp::Append,
        "<"  => RedirectionOp::Input,
        _    => return Err(ParseError { 
                    message: format!("expected redirection operator, got '{0}'", token),
                    token_index: *input_index,
                })
    };

    let redirection_op_node = ParseNode {
        entry: ParseNodeType::RedirectionOp(redirection_op_token),
        children: None,
    };

    tree_node.children.as_mut().unwrap().push(redirection_op_node);
    *input_index += 1;
    return Ok(())
}
