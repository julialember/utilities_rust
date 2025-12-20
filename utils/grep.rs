use std::{env, fmt, fs::{File, OpenOptions}, io::{self, stdin, BufRead, BufReader, Write}};

#[derive(Debug)]
enum GrepError {
    UnexpectedArg(String),
    NoArgument(String),
    UnopenedFile(io::Error),
    WriteError(io::Error),
    NoPattern,
    Help,
}

impl std::fmt::Display for GrepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedArg(s) => writeln!(f, "unexpected arg: {}", s),
            Self::UnopenedFile(s) => writeln!(f, "can't open the file: {}", s),
            Self::NoArgument(s) => writeln!(f, "no argument after: {}", s),
            Self::NoPattern => writeln!(f, "no pattern"),
            Self::WriteError(s) => writeln!(f, "error with write into file: {}", s),
            Self::Help => writeln!(f, "message: Just helping"),
        }
    }
}

struct Grep{
    pattern: String,
    inputfile: Option<File>,
    outfile: Box<dyn Write>,
    count: bool,
    ignore_case: bool,
    line_number: bool,
}

impl Grep {
    fn run(mut self) -> Result<(), GrepError> {
        if self.ignore_case {
            self.pattern = self.pattern.to_lowercase()
        }
        match self.inputfile {
            Some(input) => {
                let buffer = BufReader::new(input);
                if self.count {
                    match writeln!(self.outfile, "{}",  
                        buffer.lines().flatten().filter(|line| 
                            (self.ignore_case && line.to_lowercase().contains(&self.pattern)) || 
                            (!self.ignore_case && line.contains(&self.pattern))).count()) {
                            Err(e) => return Err(GrepError::WriteError(e)),
                            Ok(_) => return Ok(()),
                        } 
                } 
                for (numero, line) in buffer.lines().flatten().enumerate() {
                    if (self.ignore_case && line.to_lowercase().contains(&self.pattern))
                            || (!self.ignore_case && line.contains(&self.pattern)) {
                        let line = if self.line_number {format!("{}. {}\n", numero, line)} 
                            else {line};
                        if let Err(e) = self.outfile.write_all(line.as_bytes()) {
                            return Err(GrepError::WriteError(e));
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
                    if (self.ignore_case && buffer.to_lowercase().contains(&self.pattern)) 
                        || (!self.ignore_case && buffer.contains(&self.pattern))
                    {
                        if self.count {line_count+=1} 
                        else if let Err(e) = write!(self.outfile, "{}", 
                            if self.line_number {format!("{}{}", line_number, buffer)} 
                            else { buffer.to_owned()}) {
                            return Err(GrepError::WriteError(e))
                        }
                    } 
                    line_number+=1;
                    buffer.clear();
                }
                if self.count && 
                    let Err(e) = writeln!(self.outfile, "{}", line_count) {
                        return Err(GrepError::WriteError(e)) 
                }
            }
        }
        Ok(())
    }
    
    fn new(args: &Vec<String>) -> Result<Self, GrepError> {
        let mut i = 1;
        let mut add_mode: bool = false;
        let mut pattern: Option<&str> = None;
        let mut outfile_name: Option<&str> = None;
        let mut input_name: Option<&str> = None;
        let mut ignore_case = false;
        let mut line_number = false;
        let mut count = false;
        while i < args.len() {
            if args[i].starts_with('-') {
                match args[i].trim() {
                    "-o" | "--output" | "--outfile" | "--to" => {
                        if i + 1 >= args.len() {
                            return Err(GrepError::NoArgument(args[i].clone()));
                        } else {
                            i += 1;
                            outfile_name = Some(&args[i]);
                        }
                    }
                    "-in" | "--input-file" | "-f" | "--from" => {
                        if i + 1 >= args.len() {
                            return Err(GrepError::NoArgument(args[i].clone()));
                        } else {
                            i += 1;
                            input_name = Some(&args[i])
                        }
                    }
                    "-p" | "--pattern" | "--pat"  => {
                        if i + 1 >= args.len() {
                            return Err(GrepError::NoArgument(args[i].clone()));
                        } else {
                            i += 1;
                            pattern = Some(&args[i])
                        }
                    }
                    "-c" | "--count" | "--count-lines" => count = true,
                    "-he" | "--help" | "--help-mode" => {
                        Self::help();
                        return Err(GrepError::Help);
                    }
                    "-a" | "--add-mode" | "--add" => add_mode = true,
                    "-l" | "--line" | "--line-number" => line_number = true,
                    "-i" | "--ignore-case" | "--ignore" => ignore_case = true,
                    _ => return Err(GrepError::UnexpectedArg(args[i].clone())),
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
            None => Err(GrepError::NoPattern),
            Some(pattern) => Ok(Self {
                count,
                pattern: pattern.to_owned(),
                line_number,
                ignore_case,
                outfile: Self::read_out_file(outfile_name, add_mode)?,
                inputfile: Self::read_in_file(input_name)?,
            }
        )}
    }

    fn read_out_file(
        filename: Option<&str>,
        add_mode: bool,
    ) -> Result<Box<dyn Write>, GrepError> {
        match filename {
            Some(name) => match OpenOptions::new()
                .append(add_mode)
                .write(true)
                .create(true)
                .truncate(!add_mode)
                .open(name)
            {
                Ok(file) => Ok(Box::new(file)),
                Err(e) => Err(GrepError::UnopenedFile(e)),
            },
            None => Ok(Box::new(std::io::stdout())),
        }
    }

    fn read_in_file(filename: Option<&str>) -> Result<Option<File>, GrepError> {
        match filename {
            Some(name) => match File::open(name) {
                Ok(file) => Ok(Some(file)),
                Err(e) => Err(GrepError::UnopenedFile(e)),
            },
            None => Ok(None),
        }
    }

    fn help() {
        println!("[SEARCH IN] [PATTERN] [WRITE OUT]\nFlags and commands:");
        println!(
            "USAGE: [ --from        | -f  | -in | --input-file  ] (default: STDIN) /PATH/TO/INPUT/FILE \\"
        );
        println!(
            "       [ --output      | -o  |-to |               ] (default: STDOUT) /PATH/TO/OUTPUT/FILE \\"
        );
        println!("       [--pattern      | -p        | --pat         ]: NECESSARILY PART \\");
        println!("       [ --count-lines | -c       | --count       ] default: (NON COUNT) \\");
        println!("       [ --line-number | -l       | -line         ] default: (NON NUMBER) \\");
        println!("       [ --ignore-case | -i       | -ignore       ] default: (NON IGNORE) \\");
        println!("       [ --help        | -he      | --help-mode   ]: help cmmand \\");
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    match Grep::new(&args) {
        Ok(g) => if let Err(e) = g.run() {
            eprintln!("grep error: {}", e); 
        },
        Err(e) => eprintln!("grep error: {}", e),
    }
}
