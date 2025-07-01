use dirs::home_dir;
use std::env::{current_dir, set_current_dir};
use std::error::Error;
use std::io::{Write, stdin, stdout};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn formatted_pwd(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    if let Some(home) = home_dir() {
        let home_str = home.to_string_lossy();

        if path_str.starts_with(&*home_str) {
            return path_str.replacen(&*home_str, "~", 1);
        }
    }

    path_str.to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        if let Ok(directory) = current_dir() {
            let pwd = formatted_pwd(directory.as_path());
            print!("{} $ ", pwd);
        } else {
            print!("$ ");
        }
        let _ = stdout().flush();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let mut commands = buf.trim().split(" | ").peekable();
        let mut previous_child = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split(char::is_whitespace);

            let program = parts.next().unwrap();
            let mut args = parts;

            match program {
                "exit" => return Ok(()),
                "cd" => {
                    let home = home_dir().unwrap();
                    let path = args.next().map_or(home.as_path(), Path::new);
                    if let Err(e) = set_current_dir(path) {
                        eprintln!("{}", e);
                    }
                }
                program => {
                    let stdin = previous_child.map_or(Stdio::inherit(), |child: Child| {
                        child.stdout.unwrap().into()
                    });
                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };
                    let child = Command::new(program)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();
                    match child {
                        Ok(child) => previous_child = Some(child),
                        Err(e) => {
                            eprintln!("{}", e);
                            previous_child = None
                        }
                    };
                }
            }
        }

        if let Some(mut tail) = previous_child {
            tail.wait();
        }
    }
}
