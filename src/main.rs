/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::io::Result;
use std::path::{Path, PathBuf};

fn main() {
    use std::env::args_os;

    let input = args_os().skip(1); // first element is a garbage/undefined value;
                                   // everything after is a command line given arg.

    handle_paths(input);
}

fn handle_paths(paths: impl Iterator<Item = impl Into<PathBuf>>) {
    use std::thread;
    //TODO: deal with directories, [ErrorKind::IsADirectory] is still experimental

    let mut threads = Vec::new();
    threads.reserve_exact(hint_from_iter(&paths));

    for path in paths.map(std::convert::Into::into) {
        threads.push(thread::spawn(move || {
            handle_file(&path).unwrap_or_else(|e| eprintln!("{path:?}: {e}"));
        }));
    }
    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap());
}

fn handle_file(path: impl AsRef<Path>) -> Result<()> {
    use std::fs::File;
    use std::io::{Read, Seek, Write};

    let mut text = String::new();

    let mut file = File::options().read(true).write(true).open(path)?;

    // TODO: deal with [io::ErrorKind::OutOfMemory]
    file.read_to_string(&mut text)?;
    correct_spaces(&mut text);
    file.set_len(text.len() as u64)?;
    file.rewind()?;
    file.write_all(text.as_bytes())
}

fn correct_spaces(text: &mut String) {
    let evil_sigils = ["!", "=", "<", ">"]; // Using a HashSet seems to balloon the instruction count; Last checked on rustc 1.78.0
    let mut i = 1usize;
    'Outer: loop {
        if i >= text.len() {
            return;
        }
        if text.get(i..=i).unwrap_or_default() == "<"
            && !evil_sigils.contains(&text.get(i + 1..=i + 1).unwrap_or_default())
        // Do not enter on Wikipedia comments, or equations (e.g. <!-- comment -->, <=)
        {
            loop {
                i += 1;
                if i >= text.len() {
                    return;
                }
                if text.get(i - 1..=i).unwrap_or_default() == "</"
                // This also deals with self-closing blocks like '</ref name="">'
                {
                    loop {
                        i += 1;
                        if i >= text.len() {
                            return;
                        }
                        if text.get(i - 1..i).unwrap_or_default() == ">" {
                            continue 'Outer;
                        }
                    }
                }
            }
        }
        if text.get(i - 1..=i).unwrap_or_default() == "  " {
            let space_start = i - 1;
            let space_end;
            loop {
                i += 1;
                if text.get(i..=i).unwrap_or_default() != " " {
                    space_end = i - 1;
                    break;
                }
            }
            text.replace_range(space_start..=space_end, " ");
            i = space_start;
            #[cfg(debug_assertions)]
            println!("removed spaces '{space_start}–{space_end}'");
        }

        i += 1;
    }
}

fn hint_from_iter(iter: &impl Iterator) -> usize {
    let hint = iter.size_hint();
    match hint.1 {
        None => hint.0,
        Some(size) => size,
    }
}
