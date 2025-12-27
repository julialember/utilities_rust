mod command;
mod grep;
mod cat;
use std::fmt;

use grep::{Grep, GrepError};
use cat::{Cat, CatError};
    
use command::CommandBuild;

fn run<'a, E, B>(vec: Vec<&'a str>) -> bool 
    where 
        B: CommandBuild<'a, E>,
        E: fmt::Display,
{
    match B::new(vec) {
        Ok(command) => 
            if let Err(e) = command.run() {
                eprintln!("{}", e);
                false
            } else {true}
        Err(e) => {
            eprintln!("{}", e);
            false
        }
    }
}

pub fn todo(command: &str) -> bool {
    let vec: Vec<&str> = command.trim().split_whitespace().collect();
    match vec[0] {
        "grep" => return run::<'_, GrepError, Grep>(vec),
        "cat" => return run::<'_, CatError, Cat>(vec),
        _=> {
            eprintln!("shu: unknown command: {}", vec[0]);
            return false;
        }
        
    }
}
