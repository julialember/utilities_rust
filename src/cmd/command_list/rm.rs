use std::{
    fmt, fs, io::{self, PipeReader, Write}, path::{Path, PathBuf}
};

use crate::command_build::{
    command::{Command, CommandError},
    parse::CommandBackPack,
    build::{CommandBuild, BuildError}
};

pub struct Rm<'a> {
    path: &'a Path, 
    names: Vec<&'a str>,
    dir: bool,
}

impl<'a> CommandBuild<'a, RmError> for Rm<'a> {
fn new_obj(args: Vec<&'a str>, path: &'a Path, _pipe: Option<&'a PipeReader>)
    -> Result<Box<dyn Command<'a, RmError> + 'a>, CommandError<'a, RmError>> {
        let mut i = 0;
        let mut names: Vec<&str> = Vec::new();
        let mut dir = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-rf" | "--rf" => dir = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            }
            else {
                names.push(args[i]);
            }
            i += 1;
        }
        if !names.is_empty() {
            Ok(Box::new(Self {
                names,
                path,
                dir
            }))
        } else {
            Err(CommandError::Help)
        }
    }
}

impl Rm<'_> {
    fn remove(path: &Path, name: &str, dir: bool) -> Result<(), RmError> {
        let path = path.join(name);
        if path.is_dir() && !dir {
            if !dir {
                return Err(RmError::IsDir(path))
            } else if let Err(e) = fs::remove_dir_all(&path) {
                return Err(RmError::RmError(path, e))
            }
        } else if let Err(e) = fs::remove_file(&path) {
            return Err(RmError::RmError(path, e)) 
        }
        Ok(())
    }
}

impl<'a> Command<'a, RmError> for Rm<'a> {
    fn run(self: Box<Self>, output: &mut CommandBackPack) 
            -> Result<bool, CommandError<'a, RmError>> {
        let mut exit_code = true;
        let mut last_arg: Option<PathBuf> = None;
        for arg in self.names {
            if arg.starts_with('{') && let Some(arg) = 
                    arg.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
                    for i in arg.split(',').map(|x|x.trim()) {
                        if let Err(e) = Self::remove(match &last_arg {
                            Some(pth) => pth,
                            None => &self.path
                        }, i, self.dir) {
                            match e {
                                RmError::IsDir(_) => {
                                    write!(output.stderr, "{}", e)?;  
                                    exit_code = false;
                                },
                                _=> return Err(CommandError::Other("rm", e)),
                            }
                        }               
                    }
                }
            else {
                match Self::remove(self.path, arg, self.dir) {
                    Err(e) => return Err(CommandError::Other("rm", e)),
                    Ok(pth) => last_arg = Some(pth),
                }
            }
        }
        Ok(exit_code)
    }
 
    fn help() {
        println!("Concatenate FILE(s) to standard output.");
        println!();
        println!("USAGE:");
        println!("  cat [OPTIONS] [FILE]...");
        println!();
        println!("If FILE is '-' or omitted, read from standard input.");
        println!();
        println!("OPTIONS:");
        println!("  -n, --line-number         number all output lines");
        println!("  -b, --non-blank           number non-empty output lines");
        println!("  -E, --show-ends           display $ at end of each line");
        println!("  -s, --squeeze-blank       suppress repeated empty output lines");
        println!("  -f, --from, --input-file  specify input file (can be used multiple times)");
        println!("  -he, --help               display this help and exit");
        println!();
        println!("EXAMPLES:");
        println!("  cat file.txt              Display file.txt contents");
        println!("  cat -n file1 file2        Display files with line numbers");
        println!("  cat -E > output.txt       Read stdin, show $ at line ends, write to file");
        println!("  cat file1 - file2         Display file1, then stdin, then file2");
    }
    
}
    
pub enum RmError{
    RmError(PathBuf, io::Error),
    IsDir(PathBuf),
}

impl fmt::Display for RmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RmError(pth, e) => write!(f, "error with remove element({}): {}",pth.display(), e),
            Self::IsDir(pth) => writeln!(f, "the ({}) is dir, can't remove it (use -rf for do it)", pth.display()),
        }
    }
}
