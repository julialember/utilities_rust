use std::{fmt, io::PipeReader, path::{Path, PathBuf}};

mod command;
mod ls;
mod grep;
mod cat;
mod head_tail;

use grep::{Grep, GrepError};
use cat::{Cat, CatError};
use head_tail::{HeadTail, HeadTailError};
use ls::{Ls, LsError};
    
use command::{
    CommandBuild, CommandBackPack
};

fn run<'a, E, B>(vec: Vec<&'a str>, path: &'a Path, pipe: Option<&'a PipeReader>) -> bool 
    where 
        B: CommandBuild<'a, E>,
        E: fmt::Display,
{
    let (mut str, args, (pipe_next, pipe_args)) = 
            match CommandBackPack::new(vec, path) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return false;
        }
    };
    match B::new_obj(args, path, pipe) {
        Ok(command) =>
            match command.run(&mut str) {
                Err(e) => {
                    if let Err(e) = writeln!(str.stderr, "{}", e) {
                        println!("error with write in stderr, so here the error: {}", e);
                    }
                    false
                }
                Ok(code) => {
                    if let Some(args) = pipe_args && let Some(pipe) = pipe_next{
                        str.stdout.flush().expect("can't flush stdout");
                        drop(str.stdout);
                        set(args, path, Some(&pipe))
                    } else {code}
                }

            }
        Err(e) => {
            if let Err(e) = writeln!(str.stderr, "{}", e) {
                println!("error with write into stderr, so error: {}", e);
            }
            false
        }
    }
}

pub fn set(vec: Vec<&str>, path: &Path, pipe_mode: Option<&PipeReader>) -> bool {
    match vec[0] {
        "grep" => run::<'_, GrepError, Grep>(vec, path, pipe_mode),
        "cat" =>  run::<'_, CatError, Cat>(vec, path, pipe_mode),
        "head-tail" => run::<'_, HeadTailError, HeadTail>(vec, path, pipe_mode),
        "ls" => run::<'_, LsError, Ls>(vec, path, pipe_mode),
        _=> {
            eprintln!("shu: unknown command: {}", vec[0]);
            false
        }
        
    }
}

pub fn todo(command: &str, path: PathBuf) -> bool {
    let vec: Vec<&str> = split_args(command);
    set(vec, &path, None) 
}

fn split_args(command: &str) -> Vec<&str> {
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
