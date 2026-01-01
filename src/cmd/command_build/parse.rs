use super::build::BuildError;
use std::{
    fs::{File, OpenOptions}, 
    io::{self, PipeReader, Write}, 
    path::Path
};


pub struct CommandBackPack<'a> {
    pub stdout: Box<dyn Write + 'a>,
    pub stderr: Box<dyn Write + 'a>,
}

pub enum InputFile<'a> {
    Stdin, 
    Pipe(&'a PipeReader),
    File(&'a Path, &'a str),
}


impl<'a> CommandBackPack<'a> {
    pub fn read_in_file(path: &Path, filename: &'a str) 
            -> Result<File, BuildError<'a>> {
        let path = path.join(filename);
        match File::open(&path) {
            Ok(file) => Ok(file),
            Err(e) => Err(BuildError::UnopenedFile(path ,e))
        }
    }

    fn read_out_file(
        path: &Path,
        filename: &'a str,
        add_mode: bool,
    ) -> Result<Box<dyn Write + 'a>, BuildError<'a>> {
        let path = path.join(filename);
            match OpenOptions::new()
                .append(add_mode)
                .write(true)
                .create(true)
                .truncate(!add_mode)
                .open(&path)
            {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(BuildError::UnopenedFile(path, e)),
            }
    }

    pub fn get_next<'b>(args: &'b [&'a str], i: usize) 
            -> Result<&'a str, BuildError<'a>> {
        if i + 1 >= args.len() {
            Err(BuildError::NoArgument(args[i]))
        } else {
            Ok(args[i+1])
        }
    }
    
    pub fn new(args: Vec<&'a str>, path: &Path) 
        -> Result<
            (Self, Vec<&'a str>, (Option<PipeReader>, Option<Vec<&'a str>>)), 
            BuildError<'a>
        >{
        let mut args_left = Vec::new();
        let mut i: usize = 1;
        let mut stdout_name = None;
        let mut stderr_name = None;
        let mut pipe_part = (None, None);
        let mut add_mode = false;
        let mut err_add_mode = false;
        while args.len() > i {
            match args[i] {
                ">" | "--output" | "-out" => {
                    stdout_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                }
                ">>" => {
                    stdout_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                    add_mode = true;
                }
                "|" | "--pipe" | "--pipe-mode" => {
                    if i+1 < args.len() {
                        pipe_part.1 = Some(Vec::from(&args[i+1..]));
                        break;
                    } 
                }
                "--err" | "--stderr" | "2>" | "--error" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                }
                "2>>" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                    err_add_mode=true;
                }
                "-add" | "--add-mode" => add_mode = true,
                _=> args_left.push(args[i])
            }
            i+=1;
        } 
        Ok((Self {
            stderr: if let Some(name) = stderr_name {
                Box::new(Self::read_out_file(path, name, err_add_mode)?)
            } else {Box::new(io::stderr())},
            stdout: if pipe_part.1.is_some() {
                match io::pipe() {
                    Ok((pipe_re, pipe_wr)) => {
                        pipe_part.0 = Some(pipe_re);
                        Box::new(pipe_wr)
                    }
                    Err(e) => return Err(BuildError::PipeError(e))
                } 
            }
            else if let Some(name) = stdout_name {
                Box::new(Self::read_out_file(path, name, add_mode)?) 
            } else {Box::new(io::stdout())}
        }, args_left, pipe_part))
    }
}



pub fn split_args(command: &str) -> Vec<&str> {
    let mut vec = Vec::new();
    let mut start_arg = 0;
    let mut end_arg = 0;
    let mut chars = command.chars().peekable();
    let mut was_blank = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '#' => break,
            '|' => {
                if !was_blank {
                    vec.push(&command[start_arg..end_arg]);
                }
                vec.push("|");
                was_blank = true;
                start_arg = end_arg + 1;
                end_arg = start_arg;
            }
            '>' => {
                if !was_blank {
                    vec.push(&command[start_arg..end_arg]);
                }
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    vec.push(">=");
                    end_arg += 2;
                } else if let Some(&'>') = chars.peek() {
                    chars.next();
                    vec.push(">>");
                    end_arg += 2;
                } else {
                    vec.push(">");
                    end_arg += 1;
                }
                was_blank = true;
                start_arg = end_arg;
            }
            '<' => {
                if !was_blank {
                    vec.push(&command[start_arg..end_arg]);
                }
                if let Some(&'<') = chars.peek() {
                    chars.next();
                    vec.push("<<");
                    end_arg += 2;
                } else if let Some(&'>') = chars.peek() {
                    chars.next();
                    vec.push("<>");
                    end_arg += 2;
                } else {
                    vec.push("<");
                    end_arg += 1;
                }
                was_blank = true;
                start_arg = end_arg;
            }
            '&' => {
                if !was_blank {
                    vec.push(&command[start_arg..end_arg]);
                }
                if let Some(&'&') = chars.peek() {
                    chars.next();
                    vec.push("&&");
                    end_arg += 2;
                } else {
                    vec.push("&");
                    end_arg += 1;
                }
                was_blank = true;
                start_arg = end_arg;
            }
            '\'' | '"' => {
                let quote_char = ch;
                if !was_blank && start_arg != end_arg {
                    vec.push(&command[start_arg..end_arg]);
                }
                start_arg = end_arg + 1; 
                
                while let Some(next_ch) = chars.next() {
                    end_arg += 1;
                    if next_ch == quote_char {
                        break;
                    }
                }
                
                if start_arg <= end_arg && start_arg < command.len() {
                    vec.push(&command[start_arg..end_arg]);
                }
                
                end_arg += 1; 
                start_arg = end_arg;
                was_blank = true;
            }
            ' ' | '\t' => {
                if !was_blank && start_arg != end_arg {
                    vec.push(&command[start_arg..end_arg]);
                }
                was_blank = true;
                end_arg += 1;
                start_arg = end_arg;
            }
            _ => {
                end_arg += 1;
                was_blank = false;
            }
        }
    }
    
    if !was_blank && start_arg < end_arg && start_arg < command.len() {
        vec.push(&command[start_arg..end_arg.min(command.len())]);
    }
    
    vec
}
