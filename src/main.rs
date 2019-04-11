use std::{
    fs::read_dir,
    process::exit,
    path::PathBuf,
    io::{
        self,
        BufRead,
    },
    env::{
        args,
        var,
    },
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

    let mut dirs: Vec<PathBuf> = Vec::new();

    jump(to_search, &query, &mut dirs)
        .expect("Something went wrong");

    // Sort by path length 
    dirs.sort_by(|a, b| a.as_os_str().len().cmp(&b.as_os_str().len()));

    let chosen: Option<&PathBuf>;

    match dirs.len() {
        0 => {
            eprintln!("No directories found.");
            exit(1);
        },
        1 => {
            // If there's only one, just output that dir
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

// Output indexed dir list, get user responds
fn prompt_user(dirs: &mut Vec<PathBuf>) -> usize {
    for (dir, i) in dirs.iter().zip(1..dirs.len()).rev() {
        eprintln!("{}. {}", i, dir.as_path().display());
    }
    get_input()
}

fn get_input() -> usize {
    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking so that it doesn't always need to lock and unlock
    let mut input = String::with_capacity(2);
    loop {
        if let Err(_) = stdin.read_line(&mut input) {
            continue;
        }
        match input.trim().parse() {
            Ok(v) => return v,
            Err(err) => {
                eprintln!("Bad. {}", err);
                input.clear();
            }
        }
    }
}

// Recurse dirs in path, push dirs that contain query string onto vector of matching dirs.
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

        // Will likely only err in the case that it can't
        // read the directory due to permissions.
        // That's fine.
        jump(dir.path(), query, matching);
    }
    Ok(())
}
