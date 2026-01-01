use std::{
    fmt, fs, io::{self, PipeReader},
    path::{Path, PathBuf}
};

use crate::command_build::{
    command::{Command, CommandError},
    parse::CommandBackPack,
    build::{CommandBuild, BuildError}
};

pub struct Mkdir<'a> {
    path: &'a Path,
    command_format: Vec<&'a str>,
    parents: bool, 
    verbose: bool,
}

impl<'a> CommandBuild<'a, MkdirError> for Mkdir<'a> {
fn new_obj(args: Vec<&'a str>, path: &'a Path, _p: Option<&PipeReader>)
    -> Result<Box<dyn Command<'a, MkdirError> + 'a>, CommandError<'a, MkdirError>> {
        let mut i = 0;
        let mut format: Vec<&str> = Vec::new();
        let mut verbose = false;
        let mut parents = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-p" | "--parents" => parents = true,
                    "-v" | "--verbose" => verbose = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            } else {
                format.push(args[i]);
            }
            i += 1;
        }
        if !format.is_empty() {Ok(
                Box::new(Self {
                    path, 
                    verbose,
                    command_format:format, 
                    parents
            }))}
        else {
            Err(CommandError::Help)
        } 
        
    }
}

impl Mkdir<'_> {
    fn makedir(path: &Path, name: &str, parents: bool) 
        -> Result<(), MkdirError> {
        let path = path.join(name);
        if parents{
            if let Err(e) = fs::create_dir_all(&path) {
                Err(MkdirError::CantCreateDir(path, e))
            } else {
                Ok(())
            }
        } else if let Err(e) = fs::create_dir(&path) {
            Err(MkdirError::CantCreateDir(path, e))
        } else {
            Ok(())
        }
    }
}

impl<'a> Command<'a, MkdirError> for Mkdir<'a> {
    fn run(self: Box<Self>, output: &mut CommandBackPack) 
            -> Result<bool, CommandError<'a, MkdirError>> {
        for arg in self.command_format {
            if let Err(e) = Self::makedir(
                self.path, arg, self.parents) {
                return Err(CommandError::Other("mkdir", e))
            } else if self.verbose {
                writeln!(output.stdout, "dir {} was created", arg)?;
            }
        }
        Ok(true)
    }
 
    fn help() {
        println!("Create the DIRECTORY(ies), if they do not already exist.");
        println!();
        println!("USAGE:");
        println!("  mkdir [OPTIONS] DIRECTORY...");
        println!();
        println!("OPTIONS:");
        println!("  -p, --parents     no error if existing, make parent directories as needed");
        println!("  -v, --verbose     print a message for each created directory");
        println!("  -h, --help        display this help and exit");
        println!();
        println!("EXAMPLES:");
        println!("  mkdir dir1               Create directory 'dir1'");
        println!("  mkdir -p dir1/dir2/dir3  Create directory tree (parents if needed)");
        println!("  mkdir -v dir1 dir2       Create directories with verbose output");
        println!("  mkdir -p -v a/b/c        Create directory tree with verbose output");
    }

    
}
    
pub enum MkdirError {
    CantCreateDir(PathBuf, io::Error),
    UnclosedBrecker,
}

impl fmt::Display for MkdirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CantCreateDir(pth, e) => write!(f, "can't create the directory({}): {}", pth.display(), e),
            Self::UnclosedBrecker => writeln!(f, "unclosed brecker"),
        }
    }
}
