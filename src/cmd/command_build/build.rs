use std::{
    io::{self, PipeReader},
    path::{Path, PathBuf},
};

use super::command::{Command, CommandError};

pub trait CommandBuild<'a, E> {
    fn new_obj(
        args: Vec<&'a str>,
        path: &'a Path,
        pipe: Option<&'a PipeReader>,
    ) -> Result<Box<dyn Command<'a, E> + 'a>, CommandError<'a, E>>;
}

pub enum BuildError<'a> {
    UnexpectedArg(&'a str),
    NoArgument(&'a str),
    UnopenedFile(PathBuf, io::Error),
    PipeError(io::Error),
}
