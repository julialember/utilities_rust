use std::{
    fmt,
    io::{self, BufRead, BufReader, PipeReader, Read, Write},
    path::Path,
};

use crate::command_build::{
    build::{BuildError, CommandBuild},
    command::{Command, CommandError},
    parse::{CommandBackPack, InputFile},
};

pub struct Grep<'a> {
    pattern: String,
    input_files: Vec<InputFile<'a>>,
    count: bool,
    ignore_case: bool,
    line_number: bool,
}

impl<'a> CommandBuild<'a, GrepError> for Grep<'a> {
    fn new_obj(
        args: Vec<&'a str>,
        path: &'a Path,
        pipe: Option<&'a PipeReader>,
    ) -> Result<Box<dyn Command<'a, GrepError> + 'a>, CommandError<'a, GrepError>> {
        let mut i = 0;
        let mut pattern: Option<&str> = None;
        let mut input_files: Vec<InputFile> = Vec::new();
        if let Some(pipe) = pipe {
            input_files.push(InputFile::Pipe(pipe));
        }
        let mut ignore_case = false;
        let mut line_number = false;
        let mut count = false;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_files.push(InputFile::Stdin),
                    "-in" | "--input-file" | "-f" | "--from" => {
                        match CommandBackPack::get_next(&args, i) {
                            Ok(res) => {
                                input_files.push(InputFile::File(path, res));
                                i += 1;
                            }
                            Err(e) => return Err(CommandError::BuildError(e)),
                        }
                    }
                    "-p" | "--pattern" | "--pat" => match CommandBackPack::get_next(&args, i) {
                        Ok(res) => {
                            pattern = Some(res);
                            i += 1;
                        }
                        Err(e) => return Err(CommandError::BuildError(e)),
                    },
                    "-c" | "--count" | "--count-lines" => count = true,
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    "-n" | "-ln" | "--line-number" => line_number = true,
                    "-i" | "--ignore-case" | "--ignore" => ignore_case = true,
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            } else if pattern.is_none() {
                pattern = Some(args[i]);
            } else {
                input_files.push(InputFile::File(path, args[i]))
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
                input_files,
            })),
        }
    }
}

impl<'a> Grep<'a> {
    fn match_pattern(line: &str, pattern: &str, ignore_case: bool) -> bool {
        if ignore_case {
            line.to_lowercase().contains(pattern)
        } else {
            line.contains(pattern)
        }
    }
    fn print_out(&self, output: &mut CommandBackPack, file: Box<dyn Read + 'a>) -> io::Result<()> {
        let buffer = BufReader::new(file);
        if self.count {
            writeln!(
                output.stdout,
                "{}",
                buffer
                    .lines()
                    .map_while(Result::ok)
                    .filter(|line| Self::match_pattern(line, &self.pattern, self.ignore_case))
                    .count()
            )?;
        } else {
            for (numero, line) in buffer.lines().map_while(Result::ok).enumerate() {
                if Self::match_pattern(&line, &self.pattern, self.ignore_case) {
                    let line = if self.line_number {
                        format!("{}. {}\n", numero + 1, line)
                    } else {
                        format!("{}\n", line)
                    };
                    output.stdout.write_all(line.as_bytes())?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> Command<'a, GrepError> for Grep<'a> {
    fn run(
        mut self: Box<Self>,
        output: &mut CommandBackPack,
    ) -> Result<bool, CommandError<'a, GrepError>> {
        let mut exit_code = true;
        if self.ignore_case {
            self.pattern = self.pattern.to_lowercase()
        }
        for file in self.input_files.iter() {
            let file = match Self::input_type(file) {
                Ok(file) => file,
                Err(e) => {
                    exit_code = false;
                    write!(output.stderr, "{}", e)?;
                    continue;
                }
            };
            Self::print_out(&self, output, file)?;
        }
        Ok(exit_code)
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

pub enum GrepError {
    NoPattern,
}

impl fmt::Display for GrepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPattern => write!(f, "no pattern"),
        }
    }
}
