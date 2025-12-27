use std::{fmt, io};

pub trait Command<'a, E> {
    fn help() -> () where Self: Sized;
    fn run(self: Box<Self>) -> Result<(), CommandError<'a, E>>;
}

pub trait CommandBuild<'a, E> {
    fn new(vec: Vec<&'a str>) -> Result<Box<dyn Command<'a, E> + 'a>, CommandError<'a, E>>
        where Self: Sized;
}

#[derive(Debug)]
pub enum CommandError<'a, E> {
    UnexpectedArg(&'a str),
    NoArgument(&'a str),
    UnopenedFile(io::Error),
    WriteError(io::Error),
    Help,
    Other(&'a str, E),
}

impl<'a, E: fmt::Display> fmt::Display for CommandError<'a, E>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArg(s) => writeln!(f, "shu: unexpected arg: {}", s),
            Self::UnopenedFile(s) => writeln!(f, "shu: can't open the file: {}", s),
            Self::NoArgument(s) => writeln!(f, "shu: no argument after: {}", s),
            Self::WriteError(s) => writeln!(f, "shu: error with write into file: {}", s),
            Self::Help => writeln!(f, "shu: Just helping"),
            Self::Other(name, e) => writeln!(f, "shu: {}: {}", name, e),
        }
    }
}    
