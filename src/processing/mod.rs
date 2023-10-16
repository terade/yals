use crate::filetree::{Directory, FileTree};
use crate::Args;
use anyhow::{anyhow, Result};
use std::cmp::Ordering;

impl FileTree {
    pub fn sort(&mut self, args: &Args) -> Result<()> {
        if let FileTree::DirNode(dir) = self {
            dir.sort_by(args);
            return Ok(());
        }

        Err(anyhow!("not applied to an Directory!"))
    }
}

fn sort_by_from(args: &Args) -> Box<dyn (FnMut(&FileTree, &FileTree) -> Ordering)> {
    Box::new(|a: &FileTree, b: &FileTree| -> Ordering {
        a.unwrap_as_file().name().cmp(b.unwrap_as_file().name())
    })
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
