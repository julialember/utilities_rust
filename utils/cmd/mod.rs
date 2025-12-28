use std::fmt;

mod command;
mod grep;
mod cat;
mod head_tail;

use grep::{Grep, GrepError};
use cat::{Cat, CatError};
use head_tail::{HeadTail, HeadTailError};
    
use command::CommandBuild;

fn run<'a, E, B>(vec: Vec<&'a str>) -> bool 
    where 
        B: CommandBuild<'a, E>,
        E: fmt::Display,
{
    match B::new(vec) {
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

pub fn todo(command: &str) -> bool {
    let vec: Vec<&str> = command.split_whitespace().collect();
    match vec[0] {
        "grep" => return run::<'_, GrepError, Grep>(vec),
        "cat" => return run::<'_, CatError, Cat>(vec),
        "head-tail" => return run::<'_, HeadTailError, HeadTail>(vec),
        _=> {
            eprintln!("shu: unknown command: {}", vec[0]);
            return false;
        }
        
    }
}
