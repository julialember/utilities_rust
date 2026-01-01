use std::{
    fmt, 
    fs::{self, DirEntry}, 
    io::{self, PipeReader, Write}, 
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt}, 
    path::{Path, PathBuf}
};

use crate::command_build::{
    command::{Command, CommandError},
    parse::CommandBackPack,
    build::{CommandBuild, BuildError}
};

pub struct Ls {
    dire: PathBuf,
    show_hide: bool, 
    classify: bool,
    full_info: bool,
    show_hide_and: bool,
}

impl<'a> CommandBuild<'a, LsError> for Ls {
fn new_obj(args: Vec<&'a str>, path: &'a Path, _p:Option<&'a PipeReader>) 
    -> Result<Box<dyn Command<'a, LsError> + 'a>, CommandError<'a, LsError>> {
        let mut i = 0;
        let mut dir: Option<PathBuf> = None;
        let mut show_hide = false;
        let mut classify = false;
        let mut full_info = false;
        let mut show_hide_and = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-F" | "--classify" => classify = true, 
                    "-l" | "--long-format" => full_info = true,
                    "-a" | "-all" => show_hide = true,
                    "-A" | "--almost-all" => show_hide_and = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
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
                full_info,
                classify,
                dire: if let Some(dir) = dir {dir} else {path.join(".")},
            }
        ))
    }
}


impl<'a> Command<'a, LsError> for Ls {
    fn run(mut self: Box<Self>, output: &mut CommandBackPack) 
            -> Result<bool, CommandError<'a, LsError>> {
        if self.show_hide && !self.show_hide_and{
            if self.full_info {
                Self::print_info(".".into(), &mut output.stdout)?;
                Self::print_info("..".into(), &mut output.stdout)?;
            } else {
                writeln!(output.stdout, "{}", if self.classify {"./\n../"} else {".\n.."})?;
            }
        } else if self.show_hide_and && !self.show_hide{
            self.show_hide = true; 
        }
        if self.dire.is_dir() {
            match fs::read_dir(&self.dire) {
                Ok(dir) => {
                    for ent in dir.filter_map(Result::ok) {
                        if let Some(name) = ent.path().file_name() &&
                        let Some(name) = name.to_str() {
                            if name.starts_with('.') {
                                if self.show_hide {
                                    if self.full_info {
                                        Self::print_info(ent.path(), &mut output.stdout)?;
                                    }
                                    else {
                                        write!(output.stdout, "{}", name)?;
                                        writeln!(output.stdout, "{}", 
                                            if self.classify {Self::classify(&ent)} else {' '})?;
                                    }
                                }
                            } else if self.full_info {
                                Self::print_info(ent.path(), &mut output.stdout)?; 
                            } else {
                                write!(output.stdout, "{}", name)?;
                                writeln!(output.stdout, "{}", 
                                    if self.classify {Self::classify(&ent)} else {' '})?;
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
        println!("  -a, --all                  do not ignore entries starting with .");
        println!("  -A, --almost-all           do not list implied . and ..");
        println!("  -F, --classify             show the type of element");
        println!("  -l, --long-format          show the full info of the file");
        println!("  -h, --help                 display this help and exit");
        println!();
        println!("EXAMPLES:");
        println!("  ls                         List files in the current directory");
        println!("  ls -a                      List all files, including hidden ones");
        println!("  ls -A /home/user           List all files in a directory, except '.' and '..'");
    }

    
}

impl<'a> Ls {
    fn print_info(path: PathBuf, outfile: &mut Box<dyn Write + 'a>) -> io::Result<()> {
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error with perm {:?}: {}", &path.display(), e);
                return Err(e);
            }
        };
        let mode = metadata.permissions().mode();
        let file_type = if metadata.is_dir() { 'd' } else { '-' };
        let perms = format!(
            "{}{}{}{}{}{}{}{}{}{}",
            file_type,
            if mode & 0o400 != 0 { 'r' } else { '-' },
            if mode & 0o200 != 0 { 'w' } else { '-' },
            if mode & 0o100 != 0 { 'x' } else { '-' },
            if mode & 0o040 != 0 { 'r' } else { '-' },
            if mode & 0o020 != 0 { 'w' } else { '-' },
            if mode & 0o010 != 0 { 'x' } else { '-' },
            if mode & 0o004 != 0 { 'r' } else { '-' },
            if mode & 0o002 != 0 { 'w' } else { '-' },
            if mode & 0o001 != 0 { 'x' } else { '-' },
        );

        let nlink = metadata.nlink();

        let uid = metadata.uid();
        let gid = metadata.gid();
        let size = metadata.len();


        write!(outfile,
            "{} {:>2} {} {} {:>5} {}", perms, nlink, uid, gid, size, " ")?;
        if let Some(name) = 
            path.file_name() && 
            let Some(name) = name.to_str() {
            writeln!(outfile, "{}", name)     
        } else {
            writeln!(outfile, "{}", path.display())
        }
    }


    fn classify(path: &DirEntry) -> char {
        let metadata = match path.path().symlink_metadata() {
            Ok(m) => m,
            Err(_) => return ' ', 
        };

        let file_type = metadata.file_type();

        if file_type.is_dir() {
            '/'
        } else if file_type.is_symlink() {
            '@'
        } else if file_type.is_fifo() {
            '|'
        } else if file_type.is_socket() {
            '='
        } else if !metadata.permissions().readonly() && Self::is_executable(&path.path()) {
            '*'
        } else {
            ' '
        }
    }
    fn is_executable(path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = path.metadata() {
                return meta.permissions().mode() & 0o111 != 0;
            }
        }
        false
    }
}
    
pub enum LsError{
    NotDir(PathBuf),
    ReadDirError(PathBuf, io::Error),
}

impl fmt::Display for LsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotDir(d) => write!(f, "not the dir: {}", d.display()),
            Self::ReadDirError(d, e) =>
                write!(f, "error with reading dire ({}): {}", d.display(), e),
        }
    }
}
