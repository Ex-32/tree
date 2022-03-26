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

const VERSION: &str = "1.0.1";

fn main() {

    // define and parse command args using clap library
    let args: clap::ArgMatches = clap::Command::new("tree")
        .author("Jenna Fligor <jenna@fligor.net>")
        .version(VERSION)
        .about("\nGraphically displays the directory structure of a path\n\
                  (silently ignores contents of unreadable directories)")
        .arg(clap::Arg::new("path")
            .takes_value(true)
            .help("path to root directory of tree (defaults to current \
                   directory)"))
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

    // get the search path either from the optional positional argument or from
    // getting the current working directory
    let path: path::PathBuf;
    if args.is_present("path") {
        match args.value_of("path") {
            Some(path_arg) => {
                path = path::PathBuf::from(path_arg);
            },
            None => {
                eprintln!("error: optional positional argument <path> is both \
                           set and has no value");
                std::process::exit(1);
            }
        }
    } else {
        path = match env::current_dir() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("error: unable to get current directory: {}", error);
                std::process::exit(1);
            },
        };
    }

    // canonicalize the search path, this ensures the path valid as well as
    // resolving symlinks
    let path = match path.canonicalize() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: unable to canonicalize \"{}\": {}",
                      path.to_string_lossy(), error);
            std::process::exit(1);
        }
    };

    // extract important metadata, like for example, is what this path refers to
    // a directory
    let metadata = match path.metadata() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: unable to get metadata of \"{}\": {}",
                      path.to_string_lossy(), error);
            std::process::exit(1);
        }
    };

    // directory sanity check
    if !metadata.is_dir() {
        eprintln!("error: \"{}\" is not a directory", path.to_string_lossy());
        std::process::exit(1);
    }

    // get filename, fallback to full path
    let name = match path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => path.to_string_lossy(),
    };

    // set str used for formatting based on wether the ascii flag was set
    let format_str: vec::Vec<&str> ;
    if args.is_present("ascii") {
        format_str = vec::Vec::from(["\\---","+---","    ","|   "]);
    } else {
        format_str = vec::Vec::from(["└───","├───","    ","│   "]);
    }

    // print root folder name with no prefix and start recursive subtree print
    println!("{}",name);
    print_subtree(&path, args.is_present("files"), &vec::Vec::new(),
                  &format_str);

}

// recursively prints directory entries with formatting based on prefix
fn print_subtree(path: &path::Path, show_files: bool, prefix: &vec::Vec<bool>,
                 format_str: &vec::Vec<&str>) {

    // read directory contents into iterator
    let dir_iter = match fs::read_dir(path) {
        Ok(value) => value,
        Err(_) => return,
    };

    // create a vector of directory entry, boolean pairs; the bool value stores
    // wether or not the entry is a directory
    let mut entries = vec::Vec::<(fs::DirEntry,bool)>::new() ;
    // iterate over the directory contents iterator, depending on wether or not
    // the show files flag was used, the non-directory files may be discarded
    for entry in dir_iter {
        match entry {
            Ok(value) => {
                let (is_dir, is_file) = match value.metadata() {
                    Ok(value) => (value.is_dir(),
                                  (value.is_file()||value.is_symlink())),
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
    // reclaim unused memory now that we're done adding to entries, and then
    // sort lexicographically based on path (which since they should all have
    // the same pathname is equivalent to sorting by filename)
    entries.shrink_to_fit();
    entries.sort_unstable_by_key(|(entry, _)| entry.path());

    // storing length and using .enumerate() is so that it can check if it's
    // last item in the vector, for formatting reasons
    let entries_count = entries.len();
    for (i, (entry, is_dir)) in entries.iter().enumerate() {
        let path = entry.path(); // shadow path with path of entry

         // get filename, fallback to full path
        let name = String::from(match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => path.to_string_lossy(),
        });

        // clone the prefix and push a true to it if it's the last item in the
        // vector, otherwise push false
        let mut new_prefix = prefix.clone();
        new_prefix.push(i == entries_count-1);

        // use the formatting prefix to format the path structure before the
        // filename
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

        // print filename, and then recurse if it's a directory
        println!("{}",&name);
        if *is_dir {
            print_subtree(&path, show_files, &new_prefix, format_str);
        }
    }
}
