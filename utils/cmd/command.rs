use std::{
    fmt, 
    fs::{File, OpenOptions}, 
    io::{self, Write}, 
    path::{Path, PathBuf}
};

pub trait Command<'a, E> {
    fn help() -> () where Self: Sized;
    fn run(self: Box<Self>, output: &mut CommandBackPack<'a>) -> Result<bool, CommandError<'a, E>>;
}

pub struct CommandBackPack<'a> {
    pub stdout: Box<dyn Write + 'a>,
    pub stderr: Box<dyn Write + 'a>,
}

impl<'a> CommandBackPack<'a> {
    pub fn read_in_file(filename: PathBuf) -> Result<File, BuildError<'a>> {
        match File::open(&filename) {
            Ok(file) => Ok(file),
            Err(e) => Err(BuildError::UnopenedFile(filename ,e))
        }
    }
    fn read_out_file(
        path: PathBuf,
        add_mode: bool,
    ) -> Result<Box<dyn Write + 'a >, BuildError<'a>> {
            match OpenOptions::new()
                .append(add_mode)
                .write(true)
                .create(true)
                .truncate(!add_mode)
                .open(&path)
            {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(BuildError::UnopenedFile(path, e)),
            }
    }

    pub fn get_next<'b>(args: &'b [&'a str], i: usize) -> Result<&'a str, BuildError<'a>> {
        if i + 1 >= args.len() {
            Err(BuildError::NoArgument(args[i]))
        } else {
            Ok(args[i+1])
        }
    }

    pub fn new(args: Vec<&'a str>, path: &Path) 
            -> Result<(Self, Vec<&'a str>), BuildError<'a>>{
        let mut args_left = Vec::new();
        let mut i: usize = 1;
        let mut stdout_name = None;
        let mut stderr_name = None;
        let mut add_mode = false;
        let mut err_add_mode = false;
        while args.len() > i {
            match args[i] {
                ">" | "--output" | "-out" => {
                    stdout_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                }
                ">>" => {
                    stdout_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                    err_add_mode = true;
                }
                "-err" | "--stderr" | "2>" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                }
                "2>>" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                    add_mode=true;
                }
                "-add" | "--add-mode" => add_mode = true,
                _=> args_left.push(args[i])
            }
            i+=1;
        } 
        Ok((Self {
            stderr: if let Some(name) = stderr_name {
                Box::new(Self::read_out_file(path.join(name), err_add_mode)?)
            } else {Box::new(io::stderr())},
            stdout: if let Some(name) = stdout_name {
                Box::new(Self::read_out_file(path.join(name), add_mode)?) 
            } else {Box::new(io::stdout())}
        }, args_left))
    }
}

pub trait CommandBuild<'a, E> {
    fn new_obj(args: Vec<&'a str>, path: PathBuf) -> Result<Box<dyn Command<'a, E> + 'a>, CommandError<'a, E>>;
}

#[derive(Debug)]
pub enum BuildError<'a> {
    UnexpectedArg(&'a str),
    NoArgument(&'a str),
    UnopenedFile(PathBuf, io::Error),
}

#[derive(Debug)]
pub enum CommandError<'a, E> {
    WriteError(io::Error),
    BuildError(BuildError<'a>),
    Help,
    Other(&'a str, E),
}

impl<'a, E> From<io::Error> for CommandError<'a, E> {
    fn from(value: io::Error) -> Self {
        Self::WriteError(value) 
    }
}
impl<'a> fmt::Display for BuildError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArg(s) => writeln!(f, "shu: unexpected arg: {}", s),
            Self::UnopenedFile(n, s) => writeln!(f, "shu: can't open the file ({}): {}", n.display(), s),
            Self::NoArgument(s) => writeln!(f, "shu: no argument after: {}", s),
        }
    }
}

impl<'a, E: fmt::Display> fmt::Display for CommandError<'a, E>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WriteError(s) => write!(f, "shu: error with write into file: {}", s),
            Self::Help => write!(f, "shu: Just helping"),
            Self::Other(name, e) => write!(f, "shu: {}: {}", name, e),
            Self::BuildError(e) => write!(f, "shu: build Error: {}", e),
        }
    }
}    
