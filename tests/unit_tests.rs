#[cfg(test)]
mod tests{
    use super::*;
    use crate::shell::parser::Parser;
    #[test]
    fn test_simple_command_parsing(){
        let parser=Parser::new("ls -la").unwrap();
        let result=parser.parse().unwrap();
    }
    
    #[test]
    fn test_pipeline_parsing(){
        let parser=Parser::new("ls | grep test").unwrap();
        let result=parser.parse().unwrap();
    }
    
    #[test]
    fn test_builtin_cd(){
        let mut shell=Shell::new();
        let result=builtin_cd(&mut shell,&["/tmp".to_string()]);
        assert_eq!(result, Some(0));
    }
}