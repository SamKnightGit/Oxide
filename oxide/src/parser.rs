use std::collections::HashSet;
use std::fmt;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
struct ParseError {
    message: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse Error: {}", self.message)
    }
}

lazy_static! {
    static ref COMMANDS: HashSet<&'static str> =
        [
            "ls", "list", "cat", "show", "exit", "cd", "cf", "clear", 
            "mkdir", "createf", "rm", "remove", "rmf", "removef", "create", "touch"
        ].iter().cloned().collect();

    
    static ref REDIRECTION_OPS: HashSet<&'static str> = 
        [
            ">", ">>", "<", "|"
        ].iter().cloned().collect();
}

#[derive(Debug)]
pub enum ParseNodeType {
    Expr,
    CommandExpr,
    RedirectionExpr,
    Command(String),
    File(String),
    RedirectionOp(String),
    Pipe,
}

#[derive(Debug)]
pub struct ParseNode {
    entry: ParseNodeType,
    children: Option<Vec<ParseNode>>,
} 



pub fn parse_input(input_tokens: &Vec<&str>)
{
    let mut input_index: usize = 0; 
    match parse_expr(input_tokens, &mut input_index) {
        Ok(ast) => println!("{:?}", ast),
        Err(e) => println!("Parsing Error: {0}", e),
    }
    
}

fn parse_expr(input_tokens: &Vec<&str>, input_index: &mut usize) -> Result<ParseNode>
{
    let mut parse_tree = ParseNode {
        entry: ParseNodeType::Expr,
        children: Some(Vec::new()),
    };
    
    parse_command_expr(input_tokens, input_index, &mut parse_tree)?;
   
    parse_redirection_expr(input_tokens, input_index, &mut parse_tree)?;
    
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
    Ok(())
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

    if input_tokens[*input_index] == "|"
    {
        // Add pipe to AST
        let pipe_node = ParseNode {
            entry: ParseNodeType::Pipe,
            children: None,
        };
        redirection_expr_node.children.as_mut().unwrap().push(pipe_node);
        *input_index += 1;
        parse_command_expr(input_tokens, input_index, &mut redirection_expr_node)?;
            
    }
    else
    {
        parse_redirection_op(input_tokens, input_index, &mut redirection_expr_node)?;
        
        parse_filelist(input_tokens, input_index, &mut redirection_expr_node)?; 
    }

    tree_node.children.as_mut().unwrap().push(redirection_expr_node);
    return Ok(())
}

fn parse_command(input_tokens: &Vec<&str>, input_index: &mut usize, tree_node: &mut ParseNode) -> Result<()>
{
    let command = input_tokens[*input_index];
    if !COMMANDS.contains(command) {
        return Err(ParseError { 
            message: format!("could not understand command '{0}'", command) 
        })
    }

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
    if is_filename(token)
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
    if !REDIRECTION_OPS.contains(token)
    {
        return Err(ParseError {
            message: format!("expected redirection operator, got '{0}'", token)
        })
    }
    // Add token as redirection op to syntax tree
    let redirection_op_node = ParseNode {
        entry: ParseNodeType::RedirectionOp(token.to_string()),
        children: None,
    };

    tree_node.children.as_mut().unwrap().push(redirection_op_node);
    *input_index += 1;
    return Ok(())
}

fn is_filename(word: &str) -> bool
{
    // Everything that is not a command or a redirection operator is a filename
    if COMMANDS.contains(word) || REDIRECTION_OPS.contains(word)
    {
        return false
    }
    return true
}


