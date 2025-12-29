use std::{
    fmt, 
    fs, 
    io::{self, Write}, path::PathBuf
};

use super::command::{
    Command, CommandError, CommandBuild
};

pub struct Ls<'a>{
    dire: PathBuf,
    outfile: Box<dyn Write + 'a>,
    show_hide: bool, 
    show_hide_and: bool,
}

impl<'a> CommandBuild<'a, LsError> for Ls<'a> {
fn new(args: Vec<&'a str>, path: PathBuf) 
    -> Result<Box<dyn Command<'a, LsError> + 'a>, CommandError<'a, LsError>> {
        let mut i = 1;
        let mut add_mode: bool = false;
        let mut outfile_name: Option<&str> = None;
        let mut dir: Option<PathBuf> = None;
        let mut show_hide = false;
        let mut show_hide_and = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    ">>" => {
                        if i+1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]))
                        } else {
                            i+=1;
                            add_mode=true;
                            outfile_name = Some(args[i]);
                        }
                    }
                    ">" | "-o" | "--output" | "--outfile" | "--to" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]))
                        } else {
                            i += 1;
                            outfile_name = Some(args[i]);
                        }
                    }
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "--add-mode" | "--add" => add_mode = true,
                    "-a" => show_hide = true,
                    "-A" => show_hide_and = true,
                    _ => return Err(CommandError::UnexpectedArg(args[i])),
                }
            }
            else {
                dir = Some(path.join(args[i]));
            }
            i += 1;
        }
        Ok(Box::new(Self {
                show_hide,
                show_hide_and,
                outfile: match outfile_name {
                    Some(name) => Self::read_out_file(path.join(name), add_mode)?,
                    None => Box::new(io::stdout()),
                }, 
                dire: if let Some(dir) = dir {dir} else {path},
            }
        ))
    }
}

impl<'a> Command<'a, LsError> for Ls<'a> {
    fn run(mut self: Box<Self>) -> Result<bool, CommandError<'a, LsError>> {
        if self.show_hide && !self.show_hide_and{
            writeln!(self.outfile, ".\n..")?;
        }
        if self.dire.is_dir() {
            match fs::read_dir(&self.dire) {
                Ok(dir) => {
                    for ent in dir.filter_map(Result::ok) {
                        if let Some(name) = ent.path().file_name() &&
                        let Some(name_str) = name.to_str() {
                            if name_str.starts_with('.') {
                                if self.show_hide && !(self.show_hide_and && (name_str == "." || name_str == "..")) {
                                    writeln!(self.outfile, "{}", name.display())?;
                                }
                            } else {
                                writeln!(self.outfile, "{}", name.display())?;
                            }
                        } 
                    } 
                },
                Err(e) =>
                    return Err(CommandError::Other("ls", LsError::ReadDirError(self.dire, e)))
           }
        } else {
            return Err(CommandError::Other("ls", LsError::NotDir(self.dire)))
        }
        Ok(true)
    }

    fn help() {
        println!("List information about the FILEs (the current directory by default).");
        println!();
        println!("USAGE:");
        println!("  ls [OPTIONS] [FILE]...");
        println!();
        println!("DESCRIPTION:");
        println!("  List information about the FILEs (the current directory by default).");
        println!();
        println!("OPTIONS:");
        println!("   >>                        append to FILE instead of stdout");
        println!("  -a, --all                  do not ignore entries starting with .");
        println!("  -A, --almost-all           do not list implied . and ..");
        println!("   >, -o,--output FILE       write to FILE instead of stdout");
        println!("  -h, --help                 display this help and exit");
        println!("  -a, --add-mode,            did not truncate the output file ");
        println!();
        println!("EXAMPLES:");
        println!("  ls                         List files in the current directory");
        println!("  ls -a                      List all files, including hidden ones");
        println!("  ls -A /home/user           List all files in a directory, except '.' and '..'");
    }

    
}
    
#[derive(Debug)]
pub enum LsError{
    NotDir(PathBuf),
    ReadDirError(PathBuf, io::Error),
}

impl fmt::Display for LsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotDir(d) => write!(f, "not the dir: {}", d.display()),
            Self::ReadDirError(d, e) =>
                write!(f, "error with reading die ({}): {}", d.display(), e),
        }
    }
}
