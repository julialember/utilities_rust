use std::{
    env, fs::{self, File, OpenOptions}, 
    io::{self, Write, stdin, stdout},
    path::PathBuf, sync::{Arc, Mutex, RwLock}, 
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

fn showdir(now_dir: &Arc<RwLock<PathBuf>>, fulldir: bool) -> bool{
    if let Ok(pth) = now_dir.read() { 
        if fulldir {
            println!("{}", pth.display());
        }
        else if let Some(name) = pth.file_stem() {
            print!("[{}]~$ ", name.display());
        }
        else {
            print!("[???]~$ ");
        }
        true
    } else {
        false
    }
}

fn full_dir(now_dir: &Arc<RwLock<PathBuf>>) -> Option<PathBuf> {
    if let Ok(path) = now_dir.read() {
        Some(path.clone())
    } else {None}
}

enum NewDir<'a>  {
    StrDir(&'a str),
    PathDir(PathBuf),
}

fn changedir(now_dir: &Arc<RwLock<PathBuf>>, new_dir: NewDir) -> bool {
    if let Ok(mut path) = now_dir.write() {
        match new_dir {
            NewDir::StrDir(new_dir) => match path.join(new_dir).canonicalize() {
                Ok(abs) => {
                    *path = abs;
                    true
                }
                Err(e) => {
                    eprintln!("shu: cd: {}", e);
                    false
                }
            },
            NewDir::PathDir(new_dir) => {
                *path = new_dir;
                true
            },
            
        }
    }else {false}
}

fn main() -> io::Result<()> { 
    let mut command = String::new();
    let shu_his = 
        Arc::new(Mutex::new(OpenOptions::new()
        .create(true)
        .truncate(false)
        .append(true)
        .read(true)
        .open(".shu_history")?));
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let mut thread_mode = false;
    let now_dir = Arc::new(RwLock::new(env::current_dir()?));
    let mut iter = 0;
    'mainloop: loop {
        if iter % 10 == 0 && !threads.is_empty() {
            iter = 0;
            threads.retain(|th| th.is_finished());
        }
        iter+=1;
        
        showdir(&now_dir, false);
        stdout().flush().expect("can't flush stdout");
        stdin().read_line(&mut command).expect("can't read line");

        let mut command_tr = command.trim();
        if command_tr.is_empty() {continue;}
        if command_tr.ends_with(" &") {
            thread_mode = true;
            command_tr = command_tr.trim_matches('&');
        }
        let mut code;
        for trimmed_command in command_tr.split("&&").map(|x| x.trim()) {
            let dir_clone = if let Some(dir) = full_dir(&now_dir) {
                dir
            } else {
                println!("error with getting dir!");
                continue;
            };
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
                },
                "clearHIS" => {
                    let code = match shu_arc.lock() {
                        Ok(file) => file.set_len(0).is_ok(), 
                        Err(_) => false,
                    };
                    report_code("clearHis", code, shu_arc)?;
                }
                "clear" => {
                    print!("{}[2J", 27 as char);
                    print!("{}[1;1H", 27 as char);
                    report_code("clear", true, shu_arc)?;
                }
                i if i.starts_with("cd ") || i == "cd" => {
                    code = match i.split_once(' ') {
                        Some((_, new_dir)) if !new_dir.is_empty() => {
                            changedir(&now_dir, NewDir::StrDir(new_dir))
                        } 
                        _ => if let Some(home) = env::home_dir() {
                            changedir(&now_dir, NewDir::PathDir(home))
                        } else {false}
                    };
                    report_code(i, code, shu_arc)?;
                }
                "pwd" => {
                    code = showdir(&now_dir_arc, true);
                    report_code("pwd", code, shu_arc)?;
                }

                _ => if !thread_mode {
                    code = cmd::todo(trimmed_command, dir_clone);
                    report_code(trimmed_command, code, shu_arc)?;
                } else {
                    let command_clone = trimmed_command.to_owned();
                    let thread_shu = Arc::clone(&shu_his);
                    let thread_dir = Arc::clone(&now_dir);
                    let answer = thread::spawn(move || {
                        let code = cmd::todo(&command_clone, dir_clone);
                        if let Err(e) = report_code(&command_clone, code, thread_shu) {
                            eprintln!("shu: error with write: {}", e);
                        }
                        println!("ghost process ends: {}", &command_clone); 
                        showdir(&thread_dir, false);
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

