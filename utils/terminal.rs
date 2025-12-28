use std::{
    env, 
    fs::{self, File, OpenOptions}, 
    io::{self, Write, stdin, stdout}, 
    sync::{Arc, Mutex}, 
    thread::{self, JoinHandle}
};
mod cmd;

fn report_code(command: &str, code: bool, shu_his: Arc<Mutex<File>>) -> io::Result<()>{
    println!("exit code: {}", if code {0} else {1});
    if let Ok(mut f) = shu_his.lock() {
        writeln!(f, "{} {}", command, if code {""} else {"ERROR"})?
    }
    Ok(())
}

fn process_terminated() {
    println!("tranks for using our terminal!");
}

fn main() -> io::Result<()> { 
    let mut command = String::new();
    let shu_his = 
        Arc::new(Mutex::new(OpenOptions::new()
        .create(true)
        .truncate(false)
        .append(true)
        .read(true)
        .write(true)
        .open(".shu_history")?));
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let mut thread_mode = false;
    let now_dir = Arc::new(env::current_dir()?);
    let mut iter = 0;
    'mainloop: loop {
        if iter % 10 == 0 && threads.len() != 0 {
            iter = 0;
            threads.retain(|th| th.is_finished());
        }
        iter+=1;
        if let Some(name) = now_dir.file_stem() {
            print!("[{}]~$ ", name.display());
        } else {
            print!("[???]~$ ");
        }
        stdout().flush().expect("can't flush stdout");
        stdin().read_line(&mut command).expect("can't read line");

        let mut command_tr = command.trim();
        if command_tr.len() == 0 {continue;}
        if command_tr.ends_with(" &") {
            thread_mode = true;
            command_tr = command_tr.trim_matches('&');
        }
        let mut code;
        for trimmed_command in command_tr.split("&&").map(|x| x.trim()) {
            code = true;
            let shu_arc= Arc::clone(&shu_his);
            let now_dir_arc = Arc::clone(&now_dir);
            match trimmed_command {
                "exit"=> {
                    process_terminated();
                    report_code("exit", true, shu_arc)?;
                    break 'mainloop;
                }
                "history" => {
                    match fs::read_to_string(".shu_history") {
                        Ok(his) => {
                            print!("{}", his);
                            code = true;
                        }
                        Err(e) => {
                            eprintln!("shu: history file error: {}", e);
                            code = false;
                        }
                    }
                    report_code("history", code, shu_arc)?;
                }
                "pwd" => {
                    println!("{}", now_dir_arc.display());
                    code = true;
                    report_code("pwd", true, shu_arc)?;
                }

                _ => if !thread_mode {
                    code = cmd::todo(trimmed_command);
                    report_code(trimmed_command, code, shu_arc)?;
                } else {
                    let command_clone = trimmed_command.to_owned();
                    let thread_nowdir = Arc::clone(&now_dir);
                    let thread_shu = Arc::clone(&shu_his);
                    let answer = thread::spawn(move || {
                        let code = cmd::todo(&command_clone);
                        if let Err(e) = report_code(&command_clone, code, thread_shu) {
                            eprintln!("shu: error with write: {}", e);
                        }
                        println!("ghost process ends: {}", &command_clone); 
                        if let Some(name) = thread_nowdir.file_stem() {
                            print!("[{}]~$ ", name.display());
                            stdout().flush().expect("can't flush stdout");
                        }
                    });
                    threads.push(answer);
                    thread_mode=false;
                }
            }
            if !code {break;}
        }
        command.clear();
    }
    
    for thread in threads {
        if thread.join().is_err() {
            eprintln!("error with thread")
        }
    };
    Ok(())
}

