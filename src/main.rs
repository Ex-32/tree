/*
MIT License

Copyright (c) 2022 Jenna Fligor

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::env;
use std::fs;
use std::path;
use std::vec;

const VERSION: &str = "1.0.0";

fn main() {

    let args: clap::ArgMatches = clap::Command::new("tree")
        .author("Jenna Fligor <jenna@fligor.net>")
        .version(VERSION)
        .about("\nGraphically displays the directory structure of a path\n\
                  (silently ignores contents of unreadable directories)")
        .arg(clap::Arg::new("path")
            .takes_value(true)
            .help("path to root directory of tree (defaults to current directory)"))
        .arg(clap::Arg::new("files")
            .short('f')
            .long("files")
            .takes_value(false)
            .help("Displays the names of the files in each directory"))
        .arg(clap::Arg::new("ascii")
            .short('a')
            .long("ascii")
            .takes_value(false)
            .help("Uses ASCII instead of extended characters"))
        .get_matches();

    let path: path::PathBuf;
    if args.is_present("path") {
        match args.value_of("path") {
            Some(path_arg) => {
                path = path::PathBuf::from(path_arg);
            },
            None => {
                eprintln!("error: unable to parse path");
                std::process::exit(1);
            }
        }
    } else {
        path = match env::current_dir() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("error: unable to get current directory ({})", error);
                std::process::exit(1);
            },
        };
    }
    let path = match path.canonicalize() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: unable to canonicalize path ({})", error);
            std::process::exit(1);
        }
    };

    let metadata = match path.metadata() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: unable to get metadata ({})", error);
            std::process::exit(1);
        }
    };

    if !metadata.is_dir() {
        eprintln!("error: path is not a directory");
        std::process::exit(1);
    }

    let name = match path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => path.to_string_lossy(),
    };

    let format_str: vec::Vec<&str> ;
    if args.is_present("ascii") {
        format_str = vec::Vec::from(["\\---","+---","    ","|   "]);
    } else {
        format_str = vec::Vec::from(["└───","├───","    ","│   "]);
    }

    println!("{}",name);
    print_subtree(&path, args.is_present("files"), &vec::Vec::new(), &format_str);

}

fn print_subtree(path: &path::Path, show_files: bool, prefix: &vec::Vec<bool>, format_str: &vec::Vec<&str>) {

    let dir_iter = match fs::read_dir(path) {
        Ok(value) => value,
        Err(_) => return,
    };

    let mut entries = vec::Vec::<(fs::DirEntry,bool)>::new() ;
    for entry in dir_iter {
        match entry {
            Ok(value) => {
                let (is_dir, is_file) = match value.metadata() {
                    Ok(value) => (value.is_dir(), (value.is_file()||value.is_symlink())),
                    Err(_) => {
                        (false,false)
                    },
                };
                if !is_dir && !is_file {
                    continue;
                } else if is_dir || show_files {
                    entries.push((value,is_dir));
                }
            },
            Err(_) => continue,
        };
    }
    entries.shrink_to_fit();
    entries.sort_unstable_by_key(|(entry, _)| entry.path());

    let entries_count = entries.len();
    for (i, (entry, is_dir)) in entries.iter().enumerate() {
        let path = entry.path();

        let name = String::from(match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => path.to_string_lossy(),
        });

        let mut new_prefix = prefix.clone();
        new_prefix.push(i == entries_count-1);

        let max_depth = new_prefix.len()-1;
        for (i, last_entry) in new_prefix.iter().enumerate() {
            if i == max_depth {
                if *last_entry {
                    print!("{}", format_str[0]);
                } else {
                    print!("{}", format_str[1]);
                }
            } else {
                if *last_entry {
                    print!("{}", format_str[2]);
                } else {
                    print!("{}", format_str[3]);
                }
            }
        }

        println!("{}",&name);
        if *is_dir {
            print_subtree(&path, show_files, &new_prefix, format_str);
        }
    }
}
