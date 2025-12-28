use std::{
    env, 
    io::{self, Write, stdin, stdout}
};
mod cmd;

fn main() -> io::Result<()> { 
    let mut command = String::new();
    let now_dir = env::current_dir()?;
    loop {
        if let Some(name) = now_dir.file_stem() {
            print!("[{}]~$ ", name.display());
        } else {
            print!("[???]~$ ");
        }
        stdout().flush().expect("can't flush stdout");

        stdin().read_line(&mut command).expect("can't read line");
        if command.trim().len() == 0 {continue;}
        match command.trim() {
            "exit"=> {
                process_terminated();
                break;
            }
            "pwd" => println!("{}", now_dir.display()),
            _     => {
                println!("exit code: {}", if !cmd::todo(&command) {1} else {0});
            }
        }
        command.clear();
    }
    
    Ok(())
}

fn process_terminated() {
    println!("tranks for using our terminal!");
}
