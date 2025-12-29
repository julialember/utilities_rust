use std::{
    fmt, fs::{self, DirEntry}, io::{self, Write}, os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt}, path::PathBuf
};

use super::command::{
    Command, CommandError, CommandBuild
};

pub struct Ls<'a>{
    dire: PathBuf,
    outfile: Box<dyn Write + 'a>,
    show_hide: bool, 
    classify: bool,
    full_info: bool,
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
        let mut classify = false;
        let mut full_info = false;
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
                    "-F" | "--classify" => classify = true, 
                    "-l" | "--long-format" => full_info = true,
                    "-a" | "-all" => show_hide = true,
                    "-A" | "--almost-all" => show_hide_and = true,
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
                full_info,
                classify,
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
            if self.full_info {
                Self::print_info(".".into(), &mut self.outfile)?;
                Self::print_info("..".into(), &mut self.outfile)?;
            } else {
                writeln!(self.outfile, "{}", if self.classify {"./\n../"} else {".\n.."})?;
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
                                        Self::print_info(ent.path(), &mut self.outfile)?;
                                    }
                                    else {
                                        write!(self.outfile, "{}", name)?;
                                        writeln!(self.outfile, "{}", 
                                            if self.classify {Self::classify(&ent)} else {' '})?;
                                    }
                                }
                            } else {
                                if self.full_info {
                                    Self::print_info(ent.path(), &mut self.outfile)?; 
                                } else {
                                    write!(self.outfile, "{}", name)?;
                                    writeln!(self.outfile, "{}", 
                                        if self.classify {Self::classify(&ent)} else {' '})?;
                                }
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
        println!("  -F, --classify             show the type of element");
        println!("  -l, --long-format          show the full info of the file");
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

impl<'a> Ls<'a> {
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
            "{} {:>2} {} {} {:>5} {}", perms, nlink, uid, gid, size, "")?;
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
        } else if metadata.permissions().readonly() == false && Self::is_executable(&path.path()) {
            '*'
        } else {
            ' '
        }
    }
    fn is_executable(path: &PathBuf) -> bool {
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
                write!(f, "error with reading dire ({}): {}", d.display(), e),
        }
    }
}
