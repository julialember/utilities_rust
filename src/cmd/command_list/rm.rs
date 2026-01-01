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
                    "-rf" | "--remove-force" => dir = true,
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
        if path.is_dir() {
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
        let dangerous = [
            "/", "/*", "/etc", "/bin", "/usr", "/lib", "*", ".", "..",
            "/var", "/sys", "/proc", "/dev", "/boot",
        ];
        let mut exit_code = true;
        for arg in self.names {
            for i in dangerous {
                if arg.contains(i) {
                    writeln!(output.stdout, 
                        "sorry, my creator forbid me to\n\
                        (delete those dirs/delete it like that)\n\
                        pls use the system 'rm' for it\n\
                        or you can edit my code in command_list/rm.rs file! shu!")?;
                    continue;
                }
            }
            if let Err(e) = Self::remove(self.path, arg, self.dir) {
                match e {
                    RmError::IsDir(_) => {
                        write!(output.stderr, "{}", e)?;  
                        exit_code = false;
                    },
                    _=> return Err(CommandError::Other("rm", e)),
                }
            }
        }
        Ok(exit_code)
    }
 

    fn help() {
        println!("Remove (unlink) the FILE(s).");
        println!();
        println!("USAGE:");
        println!("  rm [OPTIONS] [FILE]...");
        println!();
        println!("By default, rm does not remove directories.");
        println!();
        println!("OPTIONS:");
        println!("  -rf                   shortcut for recursive and force removal without confirmation");
        println!("  -he, --help           display this help and exit");
        println!();
        println!("EXAMPLES:");
        println!("  rm file.txt           Remove a single file");
        println!("  rm -rf /tmp/logs      Forcefully and recursively remove the logs directory");
        println!("  rm -f config.old      Remove a file without asking, even if it's write-protected");
        println!("  rm file1.txt file2.txt Remove multiple files at once");
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
