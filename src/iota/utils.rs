use std::borrow::Cow;
use std::io::{fs, File, FileMode, FileAccess, TempDir};

use buffer::Buffer;

#[cfg(test)]
pub fn data_from_str(s: &'static str) -> String {
    String::from_str(s)
}

pub fn char_width(c: char, is_cjk: bool, tab_width: uint, position: uint) -> Option<uint> {
    if c == '\t' {
        Some(tab_width - position%tab_width)
    } else {
        c.width(is_cjk)
    }
}

pub fn str_width(s: &str, is_cjk: bool, tab_width: uint) -> uint {
    s.chars().fold(0, |acc, c|
        acc + char_width(c, is_cjk, tab_width, acc).unwrap_or(0)
    )
}

pub fn save_buffer(buffer: &Buffer) {
    let lines = &buffer.lines;
    let path = match buffer.file_path {
        Some(ref p) => Cow::Borrowed(p),
        None => {
            // TODO: prompt user for file name here
            Cow::Owned(Path::new("untitled"))
        },
    };

    let tmpdir = match TempDir::new_in(&Path::new("."), "iota") {
        Ok(d) => d,
        Err(e) => panic!("file error: {}", e)
    };

    let tmppath = tmpdir.path().join(Path::new("tmpfile"));

    let mut file = match File::open_mode(&tmppath, FileMode::Open, FileAccess::Write) {
        Ok(f) => f,
        Err(e) => panic!("file error: {}", e)
    };

    for line in lines.iter() {
        let mut data = line.data.clone();
        data.push('\n');
        let result = file.write(data.as_bytes());

        if result.is_err() {
            // TODO(greg): figure out what to do here.
            panic!("Something went wrong while writing the file");
        }
    }

    if let Err(e) = fs::rename(&tmppath, &*path) {
        panic!("file error: {}", e);
    }

}
