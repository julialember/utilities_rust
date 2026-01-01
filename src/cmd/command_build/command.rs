use super::build::BuildError;
use super::parse::{CommandBackPack, InputFile};
use std::{
    fmt,
    io::{self, Read},
};

pub enum CommandError<'a, E> {
    WriteError(io::Error),
    BuildError(BuildError<'a>),
    Help,
    Other(&'a str, E),
}

impl<E> From<io::Error> for CommandError<'_, E> {
    fn from(value: io::Error) -> Self {
        Self::WriteError(value)
    }
}

pub trait Command<'a, E> {
    fn help() -> ()
    where
        Self: Sized;
    fn run(self: Box<Self>, output: &mut CommandBackPack<'a>) -> Result<bool, CommandError<'a, E>>;
    fn input_type(file: &InputFile<'a>) -> Result<Box<dyn Read + 'a>, CommandError<'a, E>>
    where
        Self: Sized,
    {
        match file {
            InputFile::Pipe(pipe_read) => Ok(Box::new(*pipe_read)),
            InputFile::Stdin => Ok(Box::new(io::stdin())),
            InputFile::File(path, filename) => {
                match CommandBackPack::read_in_file(path, filename) {
                    Ok(file) => Ok(Box::new(file)),
                    Err(e) => Err(CommandError::BuildError(e)),
                }
            }
        }
    }
}

impl fmt::Display for BuildError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PipeError(e) => writeln!(f, "shu: error with build pipe command: {}", e),
            Self::UnexpectedArg(s) => writeln!(f, "shu: unexpected arg: {}", s),
            Self::UnopenedFile(n, s) => {
                writeln!(f, "shu: can't open the file ({}): {}", n.display(), s)
            }
            Self::NoArgument(s) => writeln!(f, "shu: no argument after: {}", s),
        }
    }
}

impl<E: fmt::Display> fmt::Display for CommandError<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WriteError(s) => write!(f, "shu: error with write into file: {}", s),
            Self::Help => write!(f, "shu: Just helping"),
            Self::Other(name, e) => write!(f, "shu: {}: {}", name, e),
            Self::BuildError(e) => write!(f, "shu: build Error: {}", e),
        }
    }
}
