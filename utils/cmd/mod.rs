use std::{fmt, path::PathBuf};

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
    CommandBuild, CommandBackPack};

fn run<'a, E, B>(vec: Vec<&'a str>, path: PathBuf) -> bool 
    where 
        B: CommandBuild<'a, E>,
        E: fmt::Display,
{
    let (mut str, args) = 
            match CommandBackPack::new(vec, &path) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return false;
        }
    };
    match B::new_obj(args, path) {
        Ok(command) => 
            match command.run(&mut str) {
                Err(e) => {
                    if let Err(e) = writeln!(str.stdout, "{}", e) {
                        println!("error with write in stderr, so here the error: {}", e);
                    }
                    false
                }
                Ok(code) => code
            }
        Err(e) => {
            if let Err(e) = writeln!(str.stderr, "{}", e) {
                println!("error with write into stderr, so error: {}", e);
            }
            false
        }
    }
}

pub fn todo(command: &str, path: PathBuf) -> bool {
    let vec: Vec<&str> = command.split_whitespace().collect();
    match vec[0] {
        "grep" => run::<'_, GrepError, Grep>(vec, path),
        "cat" =>  run::<'_, CatError, Cat>(vec, path),
        "head-tail" => run::<'_, HeadTailError, HeadTail>(vec, path),
        "ls" => run::<'_, LsError, Ls>(vec, path),
        _=> {
            eprintln!("shu: unknown command: {}", vec[0]);
            false
        }
        
    }
}

