#[cfg(test)]
mod tests {
    use crate::shell::{Shell, builtins};
    use crate::shell::parser::Parser;
    use crate::command::CommandType;
    
    #[test]
    fn test_simple_command_parsing() {
        let mut parser = Parser::new("ls -la").unwrap();
        let result = parser.parse().unwrap();
        // Add assertions here
    }
    
    #[test]
    fn test_builtin_cd() {
        let mut shell = Shell::new();
        let result = builtins::execute_builtin(&mut shell, "cd", &["/tmp".to_string()]);
        assert_eq!(result, Some(0));
    }
}