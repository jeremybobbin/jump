use std::{
    fs::{
        self,
        read_dir
    },
    io::{
        self,
        Read,
        Write,
        BufReader,
        BufRead,
    },
    path::{
        PathBuf
    },
    env::{
        args,
        var,
    },
    process::exit
};

fn main() {
    let query: String = args()
        .nth(1)
        .expect("Requires search string as first argument.")
        .to_lowercase();

    // Directory to search, default to home
    let to_search = var("SEARCH")
        .or(var("HOME"))
        .or(var("PWD"))
        .expect("Expecting either SEARCH, HOME or PWD variable.");

    let mut dirs = Vec::new();

    jump(to_search, &query, &mut dirs)
        .expect("Something went wrong");

    // Sort by path length 
    dirs.sort_by(|a, b| a.as_os_str().len().cmp(&b.as_os_str().len()));

    let mut chosen: Option<&PathBuf> = None;

    match dirs.len() {
        0 => {
            eprintln!("No directories found.");
            exit(1);
        },
        1 => {
            chosen = dirs.get(0);
        },
        _ => {
            let value = prompt_user(&mut dirs);
            chosen = dirs.get(value - 1);
        }
    }

    if let Some(dir) = chosen {
        println!("{}", dir.as_path().display());
    } else {
        eprintln!("That doesn't exist.");
        exit(1);
    }
}


fn prompt_user(dirs: &mut Vec<PathBuf>) -> usize {
    for (dir, i) in dirs.iter().zip(1..) {
        eprintln!("{}. {}", i, dir.as_path().display());
    }
    get_input()
}

fn get_input() -> usize {
    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking so that it doesn't always need to lock and unlock
    let mut input = String::with_capacity(2);
    loop {
        stdin.read_line(&mut input);
        match input.trim().parse() {
            Ok(v) => return v,
            Err(err) => {
                eprintln!("Bad. {}", err);
                input.clear();
            }
        }
    }
}

fn jump<P: Into<PathBuf>>(path: P, query: &str, matching: &mut Vec<PathBuf>) -> io::Result<()> {
    let path = path.into();
    let dirs = read_dir(path.clone())?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false));

    for dir in dirs {
        let f_n = dir.file_name();
        match f_n.to_str() {
            Some(file_name) => {
                if file_name.starts_with('.') {
                    continue;
                }
                if file_name.to_lowercase().contains(query) {
                    matching.push(dir.path());
                }
            },
            None => continue
        };

        jump(dir.path(), query, matching);
    }
    Ok(())
}
