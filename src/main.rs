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
    if reduce_spaces(&mut text).is_none() {
        return Ok(());
    }
    file.set_len(text.len() as u64)?;
    file.rewind()?;
    file.write_all(text.as_bytes())?;
    file.flush()
}

fn reduce_spaces(text: &mut String) -> Option<()> {
    let evil_sigils = ["!", "=", "<", ">"]; // Using a HashSet seems to balloon the instruction count; Last checked on rustc 1.78.0
    let mut i = 1usize;
    let mut ret = None;
    let mut level = 0usize;
    'Outer: loop {
        if unlikely(i >= text.len()) {
            break;
        }
        // Skip html blocks
        if text.get(i..=i).unwrap_or_default() == "<" {
            i += 1;
            if evil_sigils.contains(&text.get(i..=i).unwrap_or_default()) {
                // Don't skip on Wikipedia comments, or equations (e.g. <!-- comment -->, <=)
                continue;
            }
            i += 1;
            let start = i;
            loop {
                if unlikely(i >= text.len()) {
                    break 'Outer;
                }
                match text.get(i..=i).unwrap_or_default() {
                    "\n" => {
                        i = start;
                        continue 'Outer;
                    }
                    ">" => {
                        if text.get(i - 1..=i - 1).unwrap_or_default() != "/" {
                            // if it's not a self-closing tag, increase level
                            level += 1;
                        }
                        i += 1;
                        continue 'Outer;
                    }
                    _ => {
                        i += 1;
                        continue;
                    }
                }
            }
        }
        if level > 0 && text.get(i - 1..=i).unwrap_or_default() == "</" {
            i += 1;
            let start = i;
            loop {
                if unlikely(i >= text.len()) {
                    break 'Outer;
                }
                match text.get(i..=i).unwrap_or_default() {
                    "\n" => {
                        i = start;
                        continue 'Outer;
                    }
                    ">" => {
                        level -= 1;
                        i += 1;
                        continue 'Outer;
                    }
                    _ => {
                        i += 1;
                        continue;
                    }
                }
            }
        }
        if level == 0 && text.get(i - 1..=i).unwrap_or_default() == "  " {
            let space_start = i - 1;
            let space_end;
            loop {
                i += 1;
                if text.get(i..=i).unwrap_or_default() != " " {
                    space_end = i;
                    break;
                }
            }
            text.replace_range(space_start..space_end, " ");
            i = space_start;
            ret = Some(());
            #[cfg(debug_assertions)]
            println!("removed spaces '{space_start}–{space_end}'");
        }
        i += 1;
    }
    ret
}

fn hint_from_iter(iter: &impl Iterator) -> usize {
    let hint = iter.size_hint();
    match hint.1 {
        None => hint.0,
        Some(size) => size,
    }
}

#[inline]
#[cold]
fn cold() {}

#[inline]
fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline]
fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}
