use std::{
    fmt, 
    fs::{File, OpenOptions}, 
    io::{self, Write}, 
    path::PathBuf
};

pub trait Command<'a, E> {
    fn help() -> () where Self: Sized;
    fn run(self: Box<Self>) -> Result<bool, CommandError<'a, E>>;
}

pub trait CommandBuild<'a, E> {
    fn new(vec: Vec<&'a str>, path: PathBuf) -> Result<Box<dyn Command<'a, E> + 'a>, CommandError<'a, E>>
        where Self: Sized;

    fn read_out_file(
        path: PathBuf,
        add_mode: bool,
    ) -> Result<Box<dyn Write>, CommandError<'a, E>> {
            match OpenOptions::new()
                .append(add_mode)
                .write(true)
                .create(true)
                .truncate(!add_mode)
                .open(&path)
            {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(CommandError::UnopenedFile(path, e)),
            }
    }

    fn read_in_file(filename: PathBuf) -> Result<File, CommandError<'a, E>> {
        match File::open(&filename) {
            Ok(file) => Ok(file),
            Err(e) => Err(CommandError::UnopenedFile(filename ,e)),
        }
    }
}

#[derive(Debug)]
pub enum CommandError<'a, E> {
    UnexpectedArg(&'a str),
    NoArgument(&'a str),
    UnopenedFile(PathBuf, io::Error),
    WriteError(io::Error),
    Help,
    Other(&'a str, E),
}

impl<'a, E> From<io::Error> for CommandError<'a, E> {
    fn from(value: io::Error) -> Self {
        Self::WriteError(value) 
    }
}

impl<'a, E: fmt::Display> fmt::Display for CommandError<'a, E>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArg(s) => write!(f, "shu: unexpected arg: {}", s),
            Self::UnopenedFile(n, s) => write!(f, "shu: can't open the file ({}): {}", n.display(), s),
            Self::NoArgument(s) => write!(f, "shu: no argument after: {}", s),
            Self::WriteError(s) => write!(f, "shu: error with write into file: {}", s),
            Self::Help => write!(f, "shu: Just helping"),
            Self::Other(name, e) => write!(f, "shu: {}: {}", name, e),
        }
    }
}    
