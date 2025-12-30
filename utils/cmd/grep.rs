use std::{
    fmt, 
    io::{BufRead, BufReader, Write, stdin}, 
    path::PathBuf
};



use super::command::{
    Command, CommandError, CommandBuild, CommandBackPack,
    BuildError
};

pub struct Grep {
    pattern: String,
    inputfile: Vec<Option<PathBuf>>,
    count: bool,
    ignore_case: bool,
    line_number: bool,
}

impl<'a> CommandBuild<'a, GrepError> for Grep {
fn new_obj(args: Vec<&'a str>, path: PathBuf) -> Result<Box<dyn Command<'a, GrepError> + 'a>, CommandError<'a, GrepError>> {
        let mut i = 0;
        let mut pattern: Option<&str> = None;
        let mut input_names: Vec<Option<PathBuf>> = Vec::new();
        let mut ignore_case = false;
        let mut line_number = false;
        let mut count = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_names.push(None),
                    "-in" | "--input-file" | "-f" | "--from" => {
                        match CommandBackPack::get_next(&args, i) {
                            Ok(res) => {
                                input_names.push(Some(path.join(res)));
                                i+=1;
                            }
                            Err(e) => return Err(CommandError::BuildError(e))
                        }
                    }
                    "-p" | "--pattern" | "--pat"  => {
                        match CommandBackPack::get_next(&args, i) {
                            Ok(res) => {
                                pattern = Some(res);
                                i+=1;
                            }
                            Err(e) => return Err(CommandError::BuildError(e)),
                        }
                    }
                    "-c" | "--count" | "--count-lines" => count = true,
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-n" | "-ln"       | "--line-number" => line_number = true,
                    "-i" | "--ignore-case" | "--ignore" => ignore_case = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            }
            else if pattern.is_none() {
                pattern = Some(args[i]);
            } 
            else {
                input_names.push(Some(path.join(args[i])))
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
                inputfile: input_names,
            }
        ))}
    }
}

impl<'a> Command<'a, GrepError> for Grep {
    fn run(mut self: Box<Self>, output: &mut CommandBackPack) -> Result<bool, CommandError<'a, GrepError>> {
        if self.ignore_case {
            self.pattern = self.pattern.to_lowercase()
        }
        for file in self.inputfile {
            match file {
                Some(input) => {
                    let buffer 
                        = match CommandBackPack::read_in_file(input) {
                            Ok(read) => BufReader::new(read),
                            Err(e) => return Err(CommandError::BuildError(e))
                        };
                    if self.count {
                        writeln!(output.stdout, "{}",  
                            buffer.lines().map_while(Result::ok).filter(|line| 
                                    Self::match_pattern(line, &self.pattern, self.ignore_case)).count())?;
                        return Ok(true)
                    } 
                    for (numero, line) in buffer.lines().map_while(Result::ok).enumerate() {
                        if Self::match_pattern(&line, &self.pattern, self.ignore_case){
                            let line = if self.line_number {format!("{}. {}\n", numero+1, line)} 
                                else {format!("{}\n", line)};
                            output.stdout.write_all(line.as_bytes())?;
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
                            write!(output.stdout, "{}", 
                                if self.line_number {format!("{}{}", line_number, buffer)} 
                                else { buffer.to_string() })?;
                        } 
                        line_number+=1;
                        buffer.clear();
                    }
                    if self.count {
                        writeln!(output.stdout, "{}", line_count)?;
                    }
                }
            }
        }
        Ok(true)
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
    println!("  -he, --help                 display this help and exit");
    println!();
    println!("EXAMPLES:");
    println!("  grep error log.txt          Search 'error' in log.txt");
    println!("  grep -n pattern file        Show matching lines with numbers");
    println!("  grep -c error file          Count lines containing 'error'");
}

    
}
    
#[derive(Debug)]
pub enum GrepError {
    NoPattern,
}

impl fmt::Display for GrepError {
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
}

