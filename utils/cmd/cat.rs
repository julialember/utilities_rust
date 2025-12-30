use std::{
    fmt, 
    io::{self, BufRead, BufReader, Write, stdin}, 
    path::PathBuf
};


use super::command::{
    Command, CommandError, CommandBuild, CommandBackPack,
    BuildError,
};

pub struct Cat {
    inputfiles: Vec<Option<PathBuf>>,
    show_end: bool,
    squize_blank: bool,
    count_non_empty: bool,
    line_number: bool,
}

impl<'a> CommandBuild<'a, CatError> for Cat {
fn new_obj(args: Vec<&'a str>, path: PathBuf) -> Result<Box<dyn Command<'a, CatError> + 'a>, CommandError<'a, CatError>> {
        let mut i = 0;
        let mut input_files: Vec<Option<PathBuf>> = Vec::new();
        let mut show_end = false;
        let mut line_number = false;
        let mut count_non_empty = false;
        let mut squ_bl = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_files.push(None),
                    "-in" | "--input-file" | "-f" | "--from" => {
                        match CommandBackPack::get_next(&args, i) {
                            Ok(res) => {
                                input_files.push(Some(path.join(res)));
                                i+=1 
                            }
                            Err(e) =>  return Err(CommandError::BuildError(e)),
                        } 
                    }
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-n" | "-ln" | "--line-number" => line_number = true,
                    "-E" | "--show-ends" | "--show" => show_end = true,
                    "-b" | "--non-blank" => count_non_empty = true,
                    "-s" | "--squeze" => squ_bl = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            }
            else {input_files.push(Some(path.join(args[i])))};
            i += 1;
        }
        if line_number && count_non_empty {
            line_number = false;
        }
        Ok(Box::new(Self {
                line_number, 
                count_non_empty,
                show_end,
                squize_blank: squ_bl,
                inputfiles: input_files,
            }
        ))
    }
}

impl<'a> Command<'a, CatError> for Cat {
    fn run(mut self: Box<Self>, output: &mut CommandBackPack) -> Result<bool, CommandError<'a, CatError>> {
        let mut exit_code = true;
        let mut last_blank = false;
        if self.inputfiles.is_empty() {
            self.inputfiles.push(None);
        } ;
        for file in self.inputfiles {
            let mut index = 0;
            match file {
                None => {
                    let mut buffer = String::new();
                    loop {
                        match stdin().read_line(&mut buffer) { 
                            Ok(0) => break,
                            Ok(_) => {
                                let trimmed_buffer = buffer.trim();
                                if self.squize_blank && trimmed_buffer.is_empty() {
                                    if last_blank {
                                        continue;
                                    } else {
                                        last_blank = true;
                                    }
                                }     
                                if self.line_number || (self.count_non_empty && !trimmed_buffer.is_empty()) {
                                    write!(output.stdout, "{}. ", index)?;
                                    index+=1;
                                }
                                else {
                                    last_blank = true;
                                }
                                writeln!(output.stdout, "{}{}", buffer.trim_end_matches(['\n', '\r']), 
                                    if self.show_end {"$"} else {""})?;
                                buffer.clear();
                            } 
                            Err(e) => return Err(CommandError::Other("cat", CatError::StdinError(e))),
                        }
                    }
                }
                Some(file) => {
                    match CommandBackPack::read_in_file(file) {
                        Ok(file) => {
                            let buffer = BufReader::new(file);
                            for line in buffer.lines().map_while(Result::ok) {
                                if self.squize_blank && line.trim().is_empty() {
                                    if last_blank {
                                        continue;
                                    } else {
                                        last_blank = true;
                                    }
                                } else {last_blank = false}                                

                                if self.line_number || (self.count_non_empty && !line.is_empty()) {
                                    write!(output.stdout, "{}. ", index)?;
                                    index+=1;
                                } 
                                writeln!(output.stdout, "{}{}", line, if self.show_end {"$"} else {""})?;
                            } 
                        },
                        Err(e) => {
                            write!(output.stderr, "{}", e)?;
                            exit_code = false; 
                        }
                    }   
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
        println!("  -o, --output, --to FILE   write to FILE instead of stdout");
        println!("   >>                       append to FILE instead of stdout");
        println!("  -a, --add-mode,           did not truncate the output file ");
        println!("  -he, --help               display this help and exit");
        println!();
        println!("EXAMPLES:");
        println!("  cat file.txt              Display file.txt contents");
        println!("  cat -n file1 file2        Display files with line numbers");
        println!("  cat -E > output.txt       Read stdin, show $ at line ends, write to file");
        println!("  cat file1 - file2         Display file1, then stdin, then file2");
    }
    
}
    
#[derive(Debug)]
pub enum CatError{
    StdinError(io::Error),
}

impl fmt::Display for CatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StdinError(e) => writeln!(f, "error with reading stdin: {}", e),
        }
    }
}
