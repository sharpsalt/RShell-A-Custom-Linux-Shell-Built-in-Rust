// utils/mod.rs
pub mod helpers;

// Only export what's actually used

pub mod string_utils{
    pub fn smart_split(input:&str)->Vec<String>{
        let mut result=Vec::new();
        let mut current=String::new();
        let mut in_quotes=false;
        let mut quote_char='"';
        let mut chars=input.chars().peekable();
        while let Some(ch)=chars.next(){
            match ch{
                '"' | '\'' if !in_quotes => {
                    in_quotes=true;
                    quote_char=ch;
                }
                ch if in_quotes && ch==quote_char=>{
                    in_quotes=false;
                }
                ' ' | '\t' if !in_quotes => {
                    if !current.is_empty(){
                        result.push(current.clone());
                        current.clear();
                    }
                }
                '\\' if chars.peek().is_some()=>{
                    if let Some(next_ch)=chars.next(){
                        match next_ch{
                            'n'=>current.push('\n'),
                            't'=>current.push('\t'),
                            'r'=>current.push('\r'),
                            '\\'=>current.push('\\'),
                            '"'=>current.push('"'),
                            '\''=>current.push('\''),
                            _=>{
                                current.push('\\');
                                current.push(next_ch);
                            }
                        }
                    }
                }
                _=>current.push(ch),
            }
        }
        if !current.is_empty(){
            result.push(current);
        }
        result
    }
    pub fn expand_tilde(path:&str)->String{
        if path.starts_with('~'){//expanding it to home directry
            if let Some(home)=dirs::home_dir(){
                path.replacen('~',&home.to_string_lossy(),1)
            }else{
                path.to_string()
            }
        }else{
            path.to_string()
        }
    }
    pub fn has_metacharacters(s:&str)->bool{
        s.chars().any(|c| matches!(c, '|' | '&' | ';' | '<' | '>' | '(' | ')' | '$' | '`' | '"' | '\'' | ' ' | '\t' | '\n'))
    }
}

pub mod path_utils{
    use std::path::{Path, PathBuf};
    use std::env;
    pub fn find_in_path(program:&str)->Option<PathBuf>{
        if program.contains('/'){// If program contains a slash, treat it as a path
            let path=Path::new(program);
            if path.is_file() && is_executable(path){
                return Some(path.to_path_buf());
            }
        }else{// Search in PATH
            if let Ok(path_var)=env::var("PATH"){
                for dir in path_var.split(':'){
                    let full_path=Path::new(dir).join(program);
                    if full_path.is_file() && is_executable(&full_path) {
                        return Some(full_path);
                    }
                }
            }
        }
        None
    }
    fn is_executable(path:&Path)->bool{
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata)=path.metadata() {
            let permissions=metadata.permissions();
            permissions.mode()&0o111!=0
        }else{
            false
        }
    }
    pub fn resolve_path(path:&str)->Result<PathBuf,std::io::Error>{
        let path=Path::new(path);
        if path.is_absolute(){
            Ok(path.to_path_buf())
        }else{
            let current_dir=env::current_dir()?;
            Ok(current_dir.join(path))
        }
    }
}
pub mod env_utils{
    use std::collections::HashMap;
    use std::env;
    pub fn get_all_env()->HashMap<String,String>{
        env::vars().collect()
    }
pub fn expand_variables(input:&str,env:&HashMap<String,String>)->String{
    let mut result=input.to_string();
    while let Some(start)=result.find("${"){
        if let Some(end)=result[start..].find('}'){
            let var_name=&result[start+2..start + end];
            let replacement=env.get(var_name).map(|s| s.as_str()).unwrap_or("");
            result.replace_range(start..start+end+1,replacement);
        }else{
            break;
        }
    }
    let mut chars:Vec<char>=result.chars().collect();
    let mut i=0;
    while i<chars.len(){
        if chars[i]=='$' && i+1<chars.len() && (chars[i+1].is_alphabetic() || chars[i+1]=='_') {
            let start=i;
            i+=1;
            while i<chars.len() && (chars[i].is_alphanumeric()||chars[i]=='_'){
                i+=1;
            }
            let var_name: String=chars[start + 1..i].iter().collect();
            let replacement=env.get(&var_name).map(|s| s.as_str()).unwrap_or("");
            let original_var:String = chars[start..i].iter().collect();
            result=result.replace(&format!("${}", var_name), replacement);
            result=result.replace(&original_var, replacement);
            chars=result.chars().collect();
            i=start+replacement.len();
        }else{
            i+=1;
        }
    }
    result
}
}
