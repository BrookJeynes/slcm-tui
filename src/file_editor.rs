use std::{error::Error, fs, io::Write};

use crate::BOOKMARKS_PATH;

pub fn delete_line_from_file(item_index: usize) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(BOOKMARKS_PATH)?;

    let mut items: Vec<&str> = contents.lines().collect();

    items.remove(item_index);
    let contents = items.join("\n");

    fs::write(BOOKMARKS_PATH, format!("{}\n", contents))?;

    Ok(())
}

pub fn append_to_file(contents: String) -> Result<(), Box<dyn Error>> {
    let mut file = fs::OpenOptions::new().append(true).open(BOOKMARKS_PATH)?;

    file.write_all(contents.as_bytes())?;
    file.flush()?;

    Ok(())
}
