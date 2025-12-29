use std::{
    fmt, 
    io::{self, BufRead, BufReader, Write, stdin}, 
    path::PathBuf
};

use super::command::{
    Command, CommandError, CommandBuild
};

pub struct Cat<'a>{
    inputfiles: Vec<Option<PathBuf>>,
    outfile: Box<dyn Write + 'a>,
    show_end: bool,
    squize_blank: bool,
    count_non_empty: bool,
    line_number: bool,
}

impl<'a> CommandBuild<'a, CatError> for Cat<'a> {
fn new(args: Vec<&'a str>, path: PathBuf) 
    -> Result<Box<dyn Command<'a, CatError> + 'a>, CommandError<'a, CatError>> {
        let mut i = 1;
        let mut add_mode: bool = false;
        let mut outfile_name: Option<&str> = None;
        let mut input_files: Vec<Option<PathBuf>> = Vec::new();
        let mut show_end = false;
        let mut line_number = false;
        let mut number_non_empt = false;
        let mut squ_bl = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_files.push(None),
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
                    "-in" | "--input-file" | "-f" | "--from" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]));
                        } else {
                            i += 1;
                            input_files.push(Some(path.join(args[i])))
                        }
                    }
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-a" | "--add-mode" | "--add" => add_mode = true,
                    "-n" | "-ln" | "--line-number" => line_number = true,
                    "-E" | "--show-ends" | "--show" => show_end = true,
                    "-b" | "--non-blank" => number_non_empt = true,
                    "-s" | "--squeze" => squ_bl = true,
                    _ => return Err(CommandError::UnexpectedArg(args[i])),
                }
            }
            else {input_files.push(Some(path.join(args[i])))};
            i += 1;
        }
        if line_number && number_non_empt {
            line_number = false;
        }
        Ok(Box::new(Self {
                line_number: line_number, 
                count_non_empty: number_non_empt,
                show_end: show_end,
                squize_blank: squ_bl,
                outfile: match outfile_name {
                    Some(name) => Self::read_out_file(path.join(name), add_mode)?,
                    None => Box::new(io::stdout()),
                },
                inputfiles: input_files,
            }
        ))
    }
}

impl<'a> Command<'a, CatError> for Cat<'a> {
    fn run(mut self: Box<Self>) -> Result<bool, CommandError<'a, CatError>> {
        let mut exit_code = true;
        let mut last_blank = false;
        if self.inputfiles.len() == 0 {
            self.inputfiles.push(None);
        } ;
        for file in self.inputfiles {
            let mut index = 1;
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
                                    write!(self.outfile, "{}. ", index)?;
                                    index+=1;
                                }
                                else {
                                    last_blank = true;
                                }
                                writeln!(self.outfile, "{}{}", buffer.trim_end_matches(|x| x == '\n' || x == '\r'), 
                                    if self.show_end {"$"} else {""})?;
                                buffer.clear();
                            } 
                            Err(e) => return Err(CommandError::Other("cat", CatError::StdinError(e))),
                        }
                    }
                }
                Some(file) => {
                    match Self::read_in_file(file) {
                        Ok(file) => {
                            let buffer = BufReader::new(file);
                            for line in buffer.lines().flatten() {
                                if self.squize_blank && line.trim().is_empty() {
                                    if last_blank {
                                        continue;
                                    } else {
                                        last_blank = true;
                                    }
                                } else {last_blank = false}                                

                                if self.line_number || (self.count_non_empty && !line.is_empty()) {
                                    write!(self.outfile, "{}. ", index)?;
                                    index+=1;
                                } 
                                writeln!(self.outfile, "{}{}", line, if self.show_end {"$"} else {""})?;
                            } 
                        },
                        Err(e) => {
                            eprintln!("{}", e);
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
