use std::{
    fmt, 
    io::{self, BufRead, BufReader, PipeReader, Read, Write}, 
    path::Path
};


use super::command::{
    Command, CommandError, CommandBuild, CommandBackPack,
    BuildError, InputFile
};

pub struct Cat<'a> {
    input_files: Vec<InputFile<'a>>,
    show_end: bool,
    squize_blank: bool,
    count_non_empty: bool,
    line_number: bool,
}

impl<'a> Cat<'a> {
    fn print_out(
        &self, 
        output: &mut CommandBackPack, 
        mut last_blank: bool, 
        file: Box<dyn Read + 'a>, 
        index: &mut usize) -> io::Result<bool> {
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
                *index+=1;
            } 
            writeln!(output.stdout, "{}{}", line, if self.show_end {"$"} else {""})?;
        } 
        Ok(last_blank)
    } 
}

impl<'a> CommandBuild<'a, CatError> for Cat<'a> {
fn new_obj(args: Vec<&'a str>, path: &'a Path, pipe: Option<&'a PipeReader>)
    -> Result<Box<dyn Command<'a, CatError> + 'a>, CommandError<'a, CatError>> {
        let mut i = 0;
        let mut input_files: Vec<InputFile> = Vec::new();
        if let Some(pipe) = pipe {
            input_files.push(InputFile::Pipe(pipe));
        }
        let mut show_end = false;
        let mut line_number = false;
        let mut count_non_empty = false;
        let mut squize_blank = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_files.push(InputFile::Stdin),
                    "-in" | "--input-file" | "-f" | "--from" => {
                        match CommandBackPack::get_next(&args, i) {
                            Ok(res) => {
                                input_files.push(InputFile::File(path, res));
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
                    "-s" | "--squeze" => squize_blank = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            }
            else {input_files.push(InputFile::File(path, args[i]))};
            i += 1;
        }
        if line_number && count_non_empty {
            line_number = false;
        }
        Ok(Box::new(Self {
                line_number, 
                count_non_empty,
                show_end,
                squize_blank,
                input_files,
            }
        ))
    }
}

impl<'a> Command<'a, CatError> for Cat<'a> {
    fn run(mut self: Box<Self>, output: &mut CommandBackPack) 
            -> Result<bool, CommandError<'a, CatError>> {
        let mut exit_code = true;
        let mut last_blank = false;
        if self.input_files.is_empty() {
            self.input_files.push(InputFile::Stdin);
        } ;
        for file in self.input_files.iter() {
            let mut index: usize = 0;
            let file = match Self::input_type(file) {
                Ok(file) => file,
                Err(e) => {
                    exit_code = false;
                    write!(output.stderr, "{}", e)?;
                    continue;
                }
            };
            last_blank = Self::print_out(&self, output, last_blank, file, &mut index)? 
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
    
#[derive(Debug)]
pub enum CatError{
}

impl fmt::Display for CatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "unknown Error")
    }
}
