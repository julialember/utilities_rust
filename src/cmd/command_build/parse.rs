use super::build::BuildError;
use std::{
    fs::{File, OpenOptions},
    io::{self, PipeReader, Write},
    path::Path,
};

pub struct CommandBackPack<'a> {
    pub stdout: Box<dyn Write + 'a>,
    pub stderr: Box<dyn Write + 'a>,
}

pub enum InputFile<'a> {
    Stdin,
    Pipe(&'a PipeReader),
    File(&'a Path, &'a str),
}

impl<'a> CommandBackPack<'a> {
    pub fn read_in_file(path: &Path, filename: &'a str) -> Result<File, BuildError<'a>> {
        let path = path.join(filename);
        match File::open(&path) {
            Ok(file) => Ok(file),
            Err(e) => Err(BuildError::UnopenedFile(path, e)),
        }
    }

    fn read_out_file(
        path: &Path,
        filename: &'a str,
        add_mode: bool,
    ) -> Result<Box<dyn Write + 'a>, BuildError<'a>> {
        let path = path.join(filename);
        match OpenOptions::new()
            .append(add_mode)
            .write(true)
            .create(true)
            .truncate(!add_mode)
            .open(&path)
        {
            Ok(file) => Ok(Box::new(file)),
            Err(e) => Err(BuildError::UnopenedFile(path, e)),
        }
    }

    pub fn get_next<'b>(args: &'b [&'a str], i: usize) -> Result<&'a str, BuildError<'a>> {
        if i + 1 >= args.len() {
            Err(BuildError::NoArgument(args[i]))
        } else {
            Ok(args[i + 1])
        }
    }

    pub fn new(
        args: Vec<&'a str>,
        path: &Path,
    ) -> Result<
        (
            Self,
            Vec<&'a str>,
            (Option<PipeReader>, Option<Vec<&'a str>>),
        ),
        BuildError<'a>,
    > {
        let mut args_left = Vec::new();
        let mut i: usize = 1;
        let mut stdout_name = None;
        let mut stderr_name = None;
        let mut pipe_part = (None, None);
        let mut add_mode = false;
        let mut err_add_mode = false;
        while args.len() > i {
            match args[i] {
                ">" | "--output" | "-out" => {
                    stdout_name = Some(Self::get_next(&args, i)?);
                    i += 1;
                }
                ">>" => {
                    stdout_name = Some(Self::get_next(&args, i)?);
                    i += 1;
                    add_mode = true;
                }
                "|" | "--pipe" | "--pipe-mode" => {
                    if i + 1 < args.len() {
                        pipe_part.1 = Some(Vec::from(&args[i + 1..]));
                        break;
                    }
                }
                "--err" | "--stderr" | "2>" | "--error" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i += 1;
                }
                "2>>" => {
                    stderr_name = Some(Self::get_next(&args, i)?);
                    i += 1;
                    err_add_mode = true;
                }
                "-add" | "--add-mode" => add_mode = true,
                _ => args_left.push(args[i]),
            }
            i += 1;
        }
        Ok((
            Self {
                stderr: if let Some(name) = stderr_name {
                    Box::new(Self::read_out_file(path, name, err_add_mode)?)
                } else {
                    Box::new(io::stderr())
                },
                stdout: if pipe_part.1.is_some() {
                    match io::pipe() {
                        Ok((pipe_re, pipe_wr)) => {
                            pipe_part.0 = Some(pipe_re);
                            Box::new(pipe_wr)
                        }
                        Err(e) => return Err(BuildError::PipeError(e)),
                    }
                } else if let Some(name) = stdout_name {
                    Box::new(Self::read_out_file(path, name, add_mode)?)
                } else {
                    Box::new(io::stdout())
                },
            },
            args_left,
            pipe_part,
        ))
    }
}

pub fn split_args(command: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = None;
    let mut brace_depth = 0;
    let mut chars = command.trim().chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '#' if in_quotes.is_none() => break,
            '\'' | '"' if in_quotes.is_none() => in_quotes = Some(ch),
            q if Some(q) == in_quotes => in_quotes = None,
            '{' if in_quotes.is_none() => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' if in_quotes.is_none() => {
                brace_depth = (brace_depth as i32 - 1).max(0) as usize;
                current.push(ch);
            }

            ' ' | '\t' | '|' | '>' if in_quotes.is_none() && brace_depth == 0 => {
                if !current.is_empty() {
                    args.extend(expand_braces(&current));
                    current.clear();
                }

                if ch == '|' || ch == '>' {
                    let mut op = ch.to_string();
                    if let Some(&next) = chars.peek()
                        && ch == '>'
                        && (next == '>' || next == '=')
                    {
                        op.push(chars.next().unwrap());
                    }
                    args.push(op);
                }
            }

            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        args.extend(expand_braces(&current));
    }

    args
}

fn expand_braces(input: &str) -> Vec<String> {
    if let Some(start) = input.find('{') {
        let mut depth = 0;
        let mut end = None;

        for (i, ch) in input.char_indices().skip(start) {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
            }
        }

        if let Some(end_idx) = end {
            let prefix = &input[..start];
            let suffix = &input[end_idx + 1..];
            let content = &input[start + 1..end_idx];

            let parts = split_brace_content(content);
            let mut result = Vec::new();

            for part in parts {
                let full_word = format!("{}{}{}", prefix, part, suffix);
                result.extend(expand_braces(&full_word));
            }
            return result;
        }
    }
    vec![input.to_string()]
}

fn split_brace_content(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in content.chars() {
        if (ch == ',' || ch == ' ') && depth == 0 {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            if ch == '{' {
                depth += 1;
            }
            if ch == '}' {
                depth -= 1;
            }
            current.push(ch);
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}
