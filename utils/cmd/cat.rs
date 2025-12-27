use std::{fmt, fs::{File, OpenOptions}, io::{stdin, BufRead, BufReader, Write}};

use super::command::{
    Command, CommandError, CommandBuild
};

pub struct Cat<'a>{
    inputfiles: Vec<&'a str>,
    outfile: Box<dyn Write>,
    show_end: bool,
    squize_blank: bool,
    count_non_empty: bool,
    line_number: bool,
}

impl<'a> CommandBuild<'a, CatError> for Cat<'a> {
fn new(args: Vec<&'a str>) -> Result<Box<dyn Command<'a, CatError> + 'a>, CommandError<'a, CatError>> {
        let mut i = 1;
        let mut add_mode: bool = false;
        let mut outfile_name: Option<&str> = None;
        let mut input_files: Vec<&str> = Vec::new();
        let mut show_end = false;
        let mut line_number = false;
        let mut number_non_empt = false;
        let mut squ_bl = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_files.push("-"),
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
                            outfile_name = Some(&args[i]);
                        }
                    }
                    "-in" | "--input-file" | "-f" | "--from" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]));
                        } else {
                            i += 1;
                            input_files.push(&args[i])
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
            else {input_files.push(args[i])};
            i += 1;
        }
        Ok(Box::new(Self {
                line_number: line_number, 
                count_non_empty: number_non_empt,
                show_end: show_end,
                squize_blank: squ_bl,
                outfile: Self::read_out_file(outfile_name, add_mode)?,
                inputfiles: input_files,
            }
        ))
    }
}

impl<'a> Command<'a, CatError> for Cat<'a> {
    fn run(mut self: Box<Self>) -> Result<(), CommandError<'a, CatError>> {
        if self.inputfiles.len() == 0 {
            self.inputfiles.push("-");
        } ;
        for file in self.inputfiles {
            let mut index = 1;
            match file {
                "-" => {
                    let mut buffer = String::new();
                    while stdin().read_line(&mut buffer).expect("can't read line") != 0 {
                        if self.squize_blank && buffer.is_empty() {continue;}
                        else if self.line_number || (self.count_non_empty && !buffer.trim().is_empty()) {
                            if let Err(e) = write!(self.outfile, "{}. ", index) {
                                return Err(CommandError::WriteError(e));
                            }
                            index+=1;
                        }
                        if let Err(e) = 
                            write!(self.outfile, "{}{}", buffer, if self.show_end {"$"} else {""}) {
                                    return Err(CommandError::WriteError(e))
                                }
                            buffer.clear();
                        }
                }
                _ => {
                    match Self::read_in_file(file) {
                        Ok(file) => {
                            let buffer = BufReader::new(file);
                            for line in buffer.lines().flatten() {
                                if self.squize_blank && line.is_empty() {continue;}
                                else if self.line_number || (self.count_non_empty && !line.is_empty()) {
                                    if let Err(e) = write!(self.outfile, "{}. ", index) {
                                        return Err(CommandError::WriteError(e));
                                    }
                                    index+=1;
                                }
                                if let Err(e) = 
                                    writeln!(self.outfile, "{}{}", line, if self.show_end {"$"} else {""}) {
                                    return Err(CommandError::WriteError(e))
                                }
                            } 
                        },
                        Err(e) => if let Err(e) =
                            writeln!(self.outfile, "shu: error with file ({}): {}", file, e) {
                            return Err(CommandError::WriteError(e));
                        }
                    }   
                }
            }
        }
        Ok(())
    }
 
    fn help() {
        println!("[SEARCH IN] [PATTERN] [WRITE OUT]\nFlags and commands:");
        println!("USAGE: [ --from        | -f  |-in | --input-file  ] (default: STDIN) /PATH/TO/INPUT/FILE \\");
        println!("       [ --output      | -o  |-to |               ] (default: STDOUT) /PATH/TO/OUTPUT/FILE \\");
        println!("       [ --pattern     | -p       | --pat         ]: NECESSARILY PART \\");
        println!("       [ --count-lines | -c       | --count       ] default: (NON COUNT) \\");
        println!("       [ --line-number | -l       | -line         ] default: (NON NUMBER) \\");
        println!("       [ --ignore-case | -i       | -ignore       ] default: (NON IGNORE) \\");
        println!("       [ --help        | -he      | --help-mode   ]: help cmmand \\");
    }

    
}
    
#[derive(Debug)]
pub enum CatError{}

impl std::fmt::Display for CatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "") 
    }
}



impl Cat<'_> {
    fn read_out_file(
        filename: Option<&str>,
        add_mode: bool,
    ) -> Result<Box<dyn Write>, CommandError<'_, CatError>> {
        match filename {
            Some(name) => match OpenOptions::new()
                .append(add_mode)
                .write(true)
                .create(true)
                .truncate(!add_mode)
                .open(name)
            {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(CommandError::UnopenedFile(name,e)),
            },
            None => Ok(Box::new(std::io::stdout())),
        }
    }

    fn read_in_file(filename: &str) -> Result<File, CommandError<'_, CatError>> {
        match File::open(filename) {
            Ok(file) => Ok(file),
            Err(e) => Err(CommandError::UnopenedFile(filename,e)),
        }
    }
}

