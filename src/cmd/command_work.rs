use std::{fmt, io::PipeReader, path::{Path, PathBuf}};

use crate::command_build::{
    build::CommandBuild, parse::{CommandBackPack, split_args, split_args_string}
};

use crate::command_list::{
    Grep, GrepError, 
    Cat, CatError,
    Ls, LsError,
    HeadTail, HeadTailError,
    Mkdir, MkdirError,
    RmError, Rm
};

fn run<'a, E, B>(vec: Vec<&'a str>, path: &'a Path, pipe: Option<&'a PipeReader>) -> bool 
    where 
        B: CommandBuild<'a, E>,
        E: fmt::Display,
{
    let (mut str, 
        args, 
        (pipe_next, pipe_args)) = 
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
                Ok(code) => 
                    if let Some(args_pipe) 
                        = pipe_args && let Some(pipe) = pipe_next{
                        let _= str.stdout.flush();
                        drop(str);
                        set(args_pipe, path, Some(&pipe))
                    } else {code}
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
        "mkdir" => run::<'_, MkdirError, Mkdir>(vec, path, pipe_mode), 
        "rm" => run::<'_, RmError, Rm>(vec, path, pipe_mode),
        _=> {
            eprintln!("shu: unknown command: {}", vec[0]);
            false
        }
        
    }
}

pub fn todo(command: &str, path: PathBuf) -> bool {
    let veec_s: Vec<String> = split_args_string(command); 
    println!("{:?}", veec_s);
    false
}


