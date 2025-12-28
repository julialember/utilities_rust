use std::{
    fmt, 
    fs::{File, OpenOptions}, 
    io::{stdin, BufRead, BufReader, Write}
};


use super::command::{
    Command, CommandError, CommandBuild
};

pub struct Grep{
    pattern: String,
    inputfile: Option<File>,
    outfile: Box<dyn Write>,
    count: bool,
    ignore_case: bool,
    line_number: bool,
}

impl<'a> CommandBuild<'a, GrepError> for Grep {
fn new(args: Vec<&'a str>) -> Result<Box<dyn Command<'a, GrepError> + 'a>, CommandError<'a, GrepError>> {
        let mut i = 1;
        let mut add_mode: bool = false;
        let mut pattern: Option<&str> = None;
        let mut outfile_name: Option<&str> = None;
        let mut input_name: Option<&str> = None;
        let mut ignore_case = false;
        let mut line_number = false;
        let mut count = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_name = None,
                    ">" | "-o" | "--output" | "--outfile" | "--to" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]))
                        } else {
                            i += 1;
                            outfile_name = Some(args[i]);
                        }
                    }
                    ">>" => if i+1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]))
                        } else {
                            i+=1;
                            add_mode = true;
                            outfile_name = Some(args[i]);
                        }
                    "-in" | "--input-file" | "-f" | "--from" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]));
                        } else {
                            i += 1;
                            input_name = Some(args[i])
                        }
                    }
                    "-p" | "--pattern" | "--pat"  => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::NoArgument(args[i]));
                        } else {
                            i += 1;
                            pattern = Some(&args[i])
                        }
                    }
                    "-c" | "--count" | "--count-lines" => count = true,
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-a" | "--add-mode" | "--add" => add_mode = true,
                    "-n" | "-ln"       | "--line-number" => line_number = true,
                    "-i" | "--ignore-case" | "--ignore" => ignore_case = true,
                    _ => return Err(CommandError::UnexpectedArg(args[i])),
                }
            }
            else if pattern.is_none() {
                pattern = Some(&args[i]);
            } 
            else if input_name.is_none() {
                input_name = Some(&args[i])
            }             else {
                outfile_name = Some(&args[i])
            } 
            i += 1;
        }
        match pattern {
            None => Err(CommandError::Other("grep", GrepError::NoPattern)),
            Some(pattern) => Ok(Box::new(Self {
                count,
                pattern: pattern.to_owned(),
                line_number,
                ignore_case,
                outfile: Self::read_out_file(outfile_name, add_mode)?,
                inputfile: Self::read_in_file(input_name)?,
            }
        ))}
    }
}

impl<'a> Command<'a, GrepError> for Grep {
    fn run(mut self: Box<Self>) -> Result<(), CommandError<'a, GrepError>> {
        if self.ignore_case {
            self.pattern = self.pattern.to_lowercase()
        }
        match self.inputfile {
            Some(input) => {
                let buffer = BufReader::new(input);
                if self.count {
                    match writeln!(self.outfile, "{}",  
                        buffer.lines().flatten().filter(|line| 
                                Self::match_pattern(line, &self.pattern, self.ignore_case)).count()) {
                            Err(e) => return Err(CommandError::WriteError(e)),
                            Ok(_) => return Ok(()),
                        } 
                } 
                for (numero, line) in buffer.lines().flatten().enumerate() {
                    if Self::match_pattern(&line, &self.pattern, self.ignore_case){
                        let line = if self.line_number {format!("{}. {}\n", numero, line)} 
                            else {format!("{}\n", line)};
                        if let Err(e) = self.outfile.write_all(line.as_bytes()) {
                            return Err(CommandError::WriteError(e));
                        }
                    }
                }
            }
            None => {
                let mut buffer = String::new();
                let mut line_number = 1;
                let mut line_count = 0;
                while let Ok(num) = stdin().read_line(&mut buffer) {
                    if num == 0 {break;}  
                    if Self::match_pattern(&buffer, &self.pattern, self.ignore_case){
                        if self.count {line_count+=1} 
                        else if let Err(e) = write!(self.outfile, "{}", 
                            if self.line_number {format!("{}{}", line_number, buffer)} 
                            else { format!("{}", buffer)}) {
                            return Err(CommandError::WriteError(e))
                        }
                    } 
                    line_number+=1;
                    buffer.clear();
                }
                if self.count && 
                    let Err(e) = writeln!(self.outfile, "{}", line_count) {
                        return Err(CommandError::WriteError(e)) 
                }
            }
        }
        Ok(())
    }
 
fn help() {
    println!("Search for PATTERN in each FILE or standard input.");
    println!();
    println!("USAGE:");
    println!("  grep [OPTIONS] PATTERN [FILE]...");
    println!();
    println!("If FILE is '-' or omitted, read from standard input.");
    println!();
    println!("OPTIONS:");
    println!("  -i, --ignore-case           ignore case distinctions");
    println!("  -n, --line-number           print line number with output lines");
    println!("  -c, --count                 print only a count of matching lines");
    println!("  -f, --from FILE             search PATTERN in FILE");
    println!("   >, -o,--output FILE        write to FILE instead of stdout");
    println!("   >>                         append to FILE instead of stdout");
    println!("  -he, --help                 display this help and exit");
    println!("  -a, --add-mode,             did not truncate the output file ");
    println!();
    println!("EXAMPLES:");
    println!("  grep error log.txt          Search 'error' in log.txt");
    println!("  grep -i warning *.log       Case-insensitive search in all .log files");
    println!("  grep -n pattern file        Show matching lines with numbers");
    println!("  grep -c error file          Count lines containing 'error'");
}

    
}
    
#[derive(Debug)]
pub enum GrepError {
    NoPattern,
}

impl std::fmt::Display for GrepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPattern => writeln!(f, "no pattern"),
        }
    }
}



impl Grep {
    fn match_pattern(line: &str, pattern: &str, ignore_case: bool) -> bool {
        if ignore_case {
            line.to_lowercase().contains(pattern) 
        } else {
            line.contains(pattern)
        }
    }
    
    fn read_out_file(
        filename: Option<&str>,
        add_mode: bool,
    ) -> Result<Box<dyn Write>, CommandError<'_, GrepError>> {
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

    fn read_in_file(filename: Option<&str>) -> Result<Option<File>, CommandError<'_, GrepError>> {
        match filename {
            Some(name) => match File::open(name) {
                Ok(file) => Ok(Some(file)),
                Err(e) => Err(CommandError::UnopenedFile(name,e)),
            },
            None => Ok(None),
        }
    }
}

