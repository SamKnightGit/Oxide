use std::collections::HashSet;

lazy_static! {
    static ref COMMANDS: HashSet<&'static str> =
        [
            "ls", "list", "cat", "show", "exit", "cd", "cf", "clear", 
            "mkdir", "createf", "rm", "remove", "rmf", "removef", "create", "touch"
        ].iter().cloned().collect();

    
    static ref REDIRECTION_OPS: HashSet<&'static str> = 
        [
            ">", ">>", "<"
        ].iter().cloned().collect();
}


pub fn parse_input(input_tokens: &Vec<&str>)
{
    let mut input_index: usize = 0;
    parse_expr(input_tokens, &mut input_index);

}

fn parse_expr(input_tokens: &Vec<&str>, input_index: &mut usize) -> bool
{
    // Add Expr to AST node 
    if !parse_command_expr(input_tokens, input_index)
    {
        return false
    }
    if !parse_redirection_expr(input_tokens, input_index)
    {
        return false
    }  
    return true
}

fn parse_command_expr(input_tokens: &Vec<&str>, input_index: &mut usize) -> bool
{
    if !parse_command(input_tokens, input_index)
    {
        return false
    }
    if !parse_filelist(input_tokens, input_index)
    {
        return false
    }
    return true
}

fn parse_redirection_expr(input_tokens: &Vec<&str>, input_index: &mut usize) -> bool
{
    if *input_index == input_tokens.len()
    {
        return true
    }

    if input_tokens[*input_index] == "|"
    {
        // Add pipe to AST
        if !parse_command_expr(input_tokens, input_index)
        {
            return false
        }
    }
    else
    {
        if !parse_redirection_op(input_tokens, input_index)
        {
            return false
        }
        if !parse_filelist(input_tokens, input_index)
        {
            return false
        }
    }
    return true
}

fn parse_command(input_tokens: &Vec<&str>, input_index: &mut usize) -> bool
{
    let command = input_tokens[*input_index];
    if !COMMANDS.contains(command) {
        println!("Failed to parse command: {0}", command);
        return false  
    }

    // Add Expr to AST node 
    *input_index += 1;
    return true
}

fn parse_filelist(input_tokens: &Vec<&str>, input_index: &mut usize) -> bool
{
    // epsilon rule if all tokens have been parsed or not filename
    if *input_index == input_tokens.len()
    {
        return true
    }
    
    let token = input_tokens[*input_index];
    if is_filename(token)
    {
        // Add token as file to syntax tree
        *input_index += 1;
        parse_filelist(input_tokens, input_index);
    }
    return true
}

fn parse_redirection_op(input_tokens: &Vec<&str>, input_index: &mut usize) -> bool
{
    let token = input_tokens[*input_index];
    if !REDIRECTION_OPS.contains(token)
    {
        println!("Expected redirection operator, got: {0}", token);
        return false
    }
    // Add token as redirection op to syntax tree
    *input_index += 1;
    return true
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


