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
    UnopenedFile(&'a str, io::Error),
    WriteError(io::Error),
    Help,
    Other(&'a str, E),
}

impl<'a, E: fmt::Display> fmt::Display for CommandError<'a, E>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArg(s) => write!(f, "shu: unexpected arg: {}", s),
            Self::UnopenedFile(n, s) => write!(f, "shu: can't open the file ({}): {}", n, s),
            Self::NoArgument(s) => write!(f, "shu: no argument after: {}", s),
            Self::WriteError(s) => write!(f, "shu: error with write into file: {}", s),
            Self::Help => write!(f, "shu: Just helping"),
            Self::Other(name, e) => write!(f, "shu: {}: {}", name, e),
        }
    }
}    
