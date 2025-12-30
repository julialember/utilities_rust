use std::{
    collections::VecDeque,
    fmt,
    io::{self, BufRead, BufReader, Read, Write}, path::PathBuf,
};


#[derive(Debug)]
pub enum HeadTailError<'a> {
    ParseError(&'a str),
}

use super::command::{
    Command, CommandBuild, CommandError, CommandBackPack,
    BuildError,
};

impl fmt::Display for HeadTailError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(s) => writeln!(f, "can't parse argument: {}", s)
        }
    }
}

pub struct HeadTail<'a> {
    mode: bool,
    skip_empty: bool,
    count: usize,
    inputfile: Box<dyn Read + 'a>,
}

impl<'a> Command<'a, HeadTailError<'a>> for HeadTail<'a> {
    fn run(self: Box<Self>, output: &mut CommandBackPack) 
            -> Result<bool, CommandError<'a, HeadTailError<'a>>> {
        let reader = BufReader::new(self.inputfile);
        if self.mode {
            for line in reader
                .lines()
                .map_while(Result::ok) 
                .filter(|l| !(self.skip_empty && l.is_empty()))
                .take(self.count)
            {
                output.stdout.write_all(format!("{}\n", line).as_bytes())?;
            }
        } else {
            let mut buffer = VecDeque::with_capacity(self.count);
            for line in reader.lines().map_while(Result::ok) {
                if self.skip_empty && line.is_empty() {
                    continue;
                }
                if self.count == buffer.len() {
                    buffer.pop_front();
                };
                buffer.push_back(line)
            }
            for line in buffer.iter() {
                output.stdout.write_all(format!("{}\n", line).as_bytes())?;
            }
        }
        Ok(true)
    }
    
    fn help() {
        println!("Display first or last lines of FILE(s) to standard output.");
        println!();
        println!("USAGE:");
        println!("  head-tail [OPTIONS] [FILE]");
        println!();
        println!("If FILE is '-' or omitted, read from standard input.");
        println!();
        println!("OPTIONS:");
        println!("  -h, --head-mode           display first lines (default mode)");
        println!("  -t, --tail-mode           display last lines");
        println!("  -c, --count, --count-lines N");
        println!("                            display N lines (default: 10)");
        println!("  -s, --skip-empty, --skip  skip empty lines");
        println!("  -f, --from, -i, --input-file FILE");
        println!("                            read from FILE instead of stdin");
        println!("  -o, --output, --to, --outfile FILE");
        println!("                            write to FILE instead of stdout");
        println!("  -a, --add, --add-mode     append to FILE instead of overwriting (with -o)");
        println!("  -he, --help, --help-mode  display this help and exit");
        println!();
        println!("EXAMPLES:");
        println!("  head-tail -h -c 5 file.txt       Display first 5 lines of file.txt");
        println!("  head-tail -t -c 20 < input.txt   Display last 20 lines from stdin");
        println!("  head-tail -s -c 15 file.txt      Display first 15 non-empty lines");
        println!("  head-tail -t -o output.txt       Display last 10 lines to output.txt");
    }
}

impl<'a> CommandBuild<'a, HeadTailError<'a>> for HeadTail<'a>  {
fn new_obj(args: Vec<&'a str>, path: PathBuf) -> Result<Box<dyn Command<'a, HeadTailError<'a>> + 'a>, CommandError<'a, HeadTailError<'a>>> {
        let mut i = 0;
        let mut mode: bool = true;
        let mut input_name: Option<&str> = None;
        let mut skip = false;
        let mut count = 10;
        while i < args.len() {
            if args[i].starts_with('-') || args[i].starts_with('>') {
                match args[i].trim() {
                    "-" => input_name = None,
                    "-i" | "--input-file" | "-f" | "--from" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::BuildError(BuildError::NoArgument(args[i])));
                        } else {
                            i += 1;
                            input_name = Some(args[i])
                        }
                    }
                    "-c" | "--count" | "--count-lines" => {
                        if i + 1 >= args.len() {
                            return Err(CommandError::BuildError(BuildError::NoArgument(args[i])));
                        } else {
                            i += 1;
                            count = Self::parse_arg(args[i])?;      
                        }
                    }
                    "-t" | "--tail-mode" => mode = false,
                    "-h" | "--head-mode" => mode = true,
                    "-s" | "--skip-empty" | "--skip" => skip = true,
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(CommandError::Help);
                    }
                    _ => return Err(CommandError::BuildError(BuildError::UnexpectedArg(args[i]))),
                }
            } else if input_name.is_none() {
                input_name = Some(args[i])
            } else {
                count = Self::parse_arg(args[i])?;
            }
            i += 1;
        }
        Ok(Box::new(Self {
            mode,
            count,
            skip_empty: skip,
            inputfile: match input_name {
                Some(input_name) => match CommandBackPack::read_in_file(path.join(input_name)) {
                    Ok(file) => Box::new(file),
                    Err(e) => return Err(CommandError::BuildError(e)),
                }
                None => Box::new(io::stdin()),
            }
        }))
 
    }
}

impl<'a> HeadTail<'a> {
    fn parse_arg(arg: &'a str) -> Result<usize, CommandError<'a, HeadTailError<'a>>>{
        match arg.parse::<usize>() {
            Ok(num) => Ok(num),
            Err(_) => Err(CommandError
              ::Other("head-tail", HeadTailError::ParseError(arg))),
        }
    }
}
