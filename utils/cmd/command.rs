use std::{
    fmt, 
    fs::{File, OpenOptions}, 
    io::{self, PipeReader, Read, Write}, 
    path::{Path, PathBuf}
};

pub trait Command<'a, E> {
    fn help() -> () where Self: Sized;
    fn run(self: Box<Self>, output: &mut CommandBackPack<'a>) 
        -> Result<bool, CommandError<'a, E>>;
    fn input_type(file: &InputFile<'a>) -> Result<Box<dyn Read + 'a>, CommandError<'a, E>> 
        where Self: Sized  {
        match file {
            InputFile::Pipe(pipe_read) => Ok(Box::new(*pipe_read)),
            InputFile::Stdin => Ok(Box::new(io::stdin())), 
            InputFile::File(path, filename) =>
                match CommandBackPack::read_in_file(path, filename) {
                    Ok(file) => Ok(Box::new(file)),
                    Err(e) => Err(CommandError::BuildError(e)),
            }
        }
    }
}

pub struct CommandBackPack<'a> {
    pub stdout: Box<dyn Write + 'a>,
    pub stderr: Box<dyn Write + 'a>,
}

pub enum InputFile<'a> {
    Stdin, 
    Pipe(&'a PipeReader),
    File(&'a Path, &'a str),
}


impl<'a> CommandBackPack<'a> {
    pub fn read_in_file(path: &Path, filename: &'a str) -> Result<File, BuildError<'a>> {
        let path = path.join(filename);
        match File::open(&path) {
            Ok(file) => Ok(file),
            Err(e) => Err(BuildError::UnopenedFile(path ,e))
        }
    }

    fn read_out_file(
        path: &Path,
        filename: &'a str,
        add_mode: bool,
    ) -> Result<Box<dyn Write + 'a>, BuildError<'a>> {
        let path = path.join(filename);
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
            -> Result<(Self, Vec<&'a str>, (Option<PipeReader>, Option<Vec<&'a str>>)), BuildError<'a>>{
        let mut args_left = Vec::new();
        let mut i: usize = 1;
        let mut stdout_name = None;
        let mut stderr_name = None;
        let mut pipe_part = (None, None);
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
                    add_mode = true;
                }
                "|" | "--pipe" | "--pipe-mode" => {
                    if i+1 < args.len() {
                        pipe_part.1 = Some(Vec::from(&args[i+1..]));
                        break;
                    } 
                }
                "--err" | "--stderr" | "2>" | "--error" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                }
                "2>>" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i+=1;
                    err_add_mode=true;
                }
                "-add" | "--add-mode" => add_mode = true,
                _=> args_left.push(args[i])
            }
            i+=1;
        } 
        Ok((Self {
            stderr: if let Some(name) = stderr_name {
                Box::new(Self::read_out_file(path, name, err_add_mode)?)
            } else {Box::new(io::stderr())},
            stdout: if pipe_part.1.is_some() {
                match io::pipe() {
                    Ok((pipe_re, pipe_wr)) => {
                        pipe_part.0 = Some(pipe_re);
                        Box::new(pipe_wr)
                    }
                    Err(e) => return Err(BuildError::PipeError(e))
                } 
            }
            else if let Some(name) = stdout_name {
                Box::new(Self::read_out_file(path, name, add_mode)?) 
            } else {Box::new(io::stdout())}
        }, args_left, pipe_part))
    }
}

pub trait CommandBuild<'a, E> {
    fn new_obj(args: Vec<&'a str>, path: &'a Path, pipe: Option<&'a PipeReader>) 
        -> Result<Box<dyn Command<'a, E> + 'a>, CommandError<'a, E>>;
}

#[derive(Debug)]
pub enum BuildError<'a> {
    UnexpectedArg(&'a str),
    NoArgument(&'a str),
    UnopenedFile(PathBuf, io::Error),
    PipeError(io::Error),
}

#[derive(Debug)]
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

impl fmt::Display for BuildError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PipeError(e) => writeln!(f, "shu: error with build pipe command: {}", e),
            Self::UnexpectedArg(s) => writeln!(f, "shu: unexpected arg: {}", s),
            Self::UnopenedFile(n, s) => 
                writeln!(f, "shu: can't open the file ({}): {}", n.display(), s),
            Self::NoArgument(s) => writeln!(f, "shu: no argument after: {}", s),
        }
    }
}

impl<E: fmt::Display> fmt::Display for CommandError<'_, E>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WriteError(s) => write!(f, "shu: error with write into file: {}", s),
            Self::Help => write!(f, "shu: Just helping"),
            Self::Other(name, e) => write!(f, "shu: {}: {}", name, e),
            Self::BuildError(e) => write!(f, "shu: build Error: {}", e),
        }
    }
}    
