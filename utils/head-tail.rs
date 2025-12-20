use std::{
    collections::VecDeque,
    env, fmt,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
};

#[derive(Debug)]
enum HeadTailError {
    UnexpectedArg(String),
    NoArgument(String),
    UnopenedFile(io::Error),
    WriteError(io::Error),
    ParseError(String),
    Help,
}

impl std::fmt::Display for HeadTailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArg(s) => writeln!(f, "unexpected arg: {}", s),
            Self::UnopenedFile(s) => writeln!(f, "can't open the file: {}", s),
            Self::NoArgument(s) => writeln!(f, "no argument after: {}", s),
            Self::ParseError(s) => writeln!(f, "can't parse argument: {}", s),
            Self::WriteError(s) => writeln!(f, "error with write into file: {}", s),
            Self::Help => writeln!(f, "message: Just helping"),
        }
    }
}

struct HeadTail {
    mode: bool,
    skip_empty: bool,
    count: usize,
    outfile: Box<dyn Write>,
    inputfile: Box<dyn Read>,
}

impl HeadTail {
    fn run(mut self) -> Result<(), HeadTailError> {
        let reader = BufReader::new(self.inputfile);
        if self.mode {
            for line in reader
                .lines()
                .filter_map(Result::ok) 
                .filter(|l| !(self.skip_empty && l.is_empty()))
                .take(self.count)
            {
                if let Err(e) = self.outfile.write_all(format!("{}\n", line).as_bytes()) {
                    return Err(HeadTailError::WriteError(e));
                };
            }
        } else {
            let mut buffer = VecDeque::with_capacity(self.count);
            for line in reader.lines().flatten() {
                if self.skip_empty && line.is_empty() {
                    continue;
                }
                if self.count == buffer.len() {
                    buffer.pop_front();
                };
                buffer.push_back(line)
            }
            for line in buffer {
                if let Err(e) = self.outfile.write_all(format!("{}\n", line).as_bytes()) {
                    return Err(HeadTailError::WriteError(e));
                }
            }
        }
        Ok(())
    }

    fn new(args: &Vec<String>) -> Result<HeadTail, HeadTailError> {
        let mut i = 1;
        let mut mode: bool = true;
        let mut add_mode: bool = false;
        let mut outfile_name: Option<&str> = None;
        let mut input_name: Option<&str> = None;
        let mut skip = false;
        let mut count = 10;
        while i < args.len() {
            if args[i].starts_with('-') {
                match args[i].trim() {
                    "-" => input_name = None,
                    "-o" | "--output" | "--outfile" | "--to" => {
                        if i + 1 >= args.len() {
                            return Err(HeadTailError::NoArgument(args[i].clone()));
                        } else {
                            i += 1;
                            outfile_name = Some(&args[i]);
                        }
                    }

                    "-i" | "--input-file" | "-f" | "--from" => {
                        if i + 1 >= args.len() {
                            return Err(HeadTailError::NoArgument(args[i].clone()));
                        } else {
                            i += 1;
                            input_name = Some(&args[i])
                        }
                    }
                    "-c" | "--count" | "--count-lines" => {
                        if i + 1 >= args.len() {
                            return Err(HeadTailError::NoArgument(args[i].clone()));
                        } else {
                            i += 1;
                            count = match args[i].parse::<usize>() {
                                Ok(num) => num,
                                Err(_) => return Err(HeadTailError::ParseError(args[i].clone())),
                            }
                        }
                    }
                    "-t" | "--tail-mode" => mode = false,
                    "-h" | "--head-mode" => mode = true,
                    "-s" | "--skip-empty" | "--skip" => skip = true,
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(HeadTailError::Help);
                    }
                    "-a" | "--add-mode" | "--add" => add_mode = true,
                    _ => return Err(HeadTailError::UnexpectedArg(args[i].clone())),
                }
            } else if input_name.is_none() {
                input_name = Some(&args[i])
            } else if outfile_name.is_none() {
                outfile_name = Some(&args[i])
            } else {
                count = match args[i].parse::<usize>() {
                    Ok(num) => num,
                    Err(_) => return Err(HeadTailError::ParseError(args[i].clone())),
                }
            }
            i += 1;
        }
        Ok(Self {
            mode,
            count,
            skip_empty: skip,
            outfile: Self::read_out_file(outfile_name, add_mode)?,
            inputfile: Self::read_in_file(input_name)?,
        })
    }

    fn read_out_file(
        filename: Option<&str>,
        add_mode: bool,
    ) -> Result<Box<dyn Write>, HeadTailError> {
        match filename {
            Some(name) => match OpenOptions::new()
                .append(add_mode)
                .write(true)
                .create(true)
                .truncate(!add_mode)
                .open(name)
            {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(HeadTailError::UnopenedFile(e)),
            },
            None => Ok(Box::new(std::io::stdout())),
        }
    }

    fn read_in_file(filename: Option<&str>) -> Result<Box<dyn Read>, HeadTailError> {
        match filename {
            Some(name) => match File::open(name) {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(HeadTailError::UnopenedFile(e)),
            },
            None => Ok(Box::new(std::io::stdin())),
        }
    }
    fn help() {
        println!("[SEARCH IN] [WRITE OUT]\nFlags and commands:");
        println!(
            "USAGE: [ --from        | -f  | -i | --input-file  ] (default: STDINT) /PATH/TO/INPUT/FILE \\"
        );
        println!(
            "       [ --output      | -o  |-to |               ] (default: STDOUT) /PATH/TO/OUTPUT/FILE \\"
        );
        println!("       [ --count-lines | -c       | --count       ] UNSIGNED NUMBER \\");
        println!("       [ --tail-mode   | -t OR -h | --head-mode   ] default: (HEAD-MODE) \\");
        println!("       [ --skip-empty  | -s       | -skip         ] default: (NON SKIP) \\");
        println!("       [ --add-mode    | -a       | -add          ] default: (NON ADD) \\");
        println!("       [ --help        | -he      | --help-mode   ]: help cmmand \\");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    match HeadTail::new(&args) {
        Ok(o) => if let Err(e) = o.run() { eprintln!("Head-Tail run error: {}", e) },
        Err(e) => eprintln!("Head-Tail error: {}", e),
    }
}
