use crate::filetree::{Directory, FileTree};
use crate::Args;
use anyhow::{anyhow, Result};
use std::cmp::Ordering;

impl FileTree {
    pub fn sort_by(&mut self, args: &Args) -> Result<()> {
        if let FileTree::DirNode(dir) = self {
            dir.sort_by(args);
            return Ok(());
        }

        Err(anyhow!("not applied to an Directory!"))
    }
}

fn sort_by_from(args: &Args) -> Box<dyn (FnMut(&FileTree, &FileTree) -> Ordering)> {
    Box::new(|a: &FileTree, b: &FileTree| -> Ordering {
        let a_name = prepare_name_for_compare(a.unwrap_as_file().name());
        let b_name = prepare_name_for_compare(b.unwrap_as_file().name());

        a_name.cmp(&b_name)
    })
}

fn prepare_name_for_compare(name: &str) -> String {
    let name = name.to_lowercase();
    let name_position = name.chars().position(|c| c.is_ascii_lowercase());

    if let Some(name_position) = name_position {
        name[name_position..].to_string()
    } else {
        name.to_string()
    }
}

impl Directory {
    fn sort_by(&mut self, args: &Args) {
        for entry in self.mut_entries() {
            if let FileTree::DirNode(dir) = entry {
                dir.mut_entries().sort_by(sort_by_from(args));
            }
        }

        self.mut_entries().sort_by(sort_by_from(args));
    }
}
