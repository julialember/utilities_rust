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
    
use command::CommandBuild;

fn run<'a, E, B>(vec: Vec<&'a str>, path: PathBuf) -> bool 
    where 
        B: CommandBuild<'a, E>,
        E: fmt::Display,
{
    match B::new(vec, path) {
        Ok(command) => 
            match command.run() {
                Err(e) => {
                    eprintln!("{}", e); 
                    false
                }
                Ok(code) => code
            }
        Err(e) => {
            eprintln!("{}", e);
            false
        }
    }
}

pub fn todo(command: &str, path: PathBuf) -> bool {
    let vec: Vec<&str> = command.split_whitespace().collect();
    match vec[0] {
        "grep" => return run::<'_, GrepError, Grep>(vec, path),
        "cat" => return run::<'_, CatError, Cat>(vec, path),
        "head-tail" => return run::<'_, HeadTailError, HeadTail>(vec, path),
        "ls" => return run::<'_, LsError, Ls>(vec, path),
        _=> {
            eprintln!("shu: unknown command: {}", vec[0]);
            return false;
        }
        
    }
}
