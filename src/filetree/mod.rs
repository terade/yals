use anyhow::anyhow;
use std::{
    fs::{DirEntry, Metadata},
    io,
    os::unix::prelude::MetadataExt,
    path::PathBuf,
};

#[derive(Debug)]
pub enum FileTree {
    DirNode(Directory),
    FileNode(File),
    LinkNode(SymLink),
}

#[derive(Debug)]
pub struct File {
    name: String,
    metadata: Metadata, // metadata covers size and permissions
}

#[derive(Debug)]
pub struct SymLink {
    file: File,
    target: String,
}

#[derive(Debug)]
pub struct Directory {
    file: File,
    entries: Vec<FileTree>,
    total_size: u64,
    count: usize,
}

impl FileTree {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::DirNode(_) => true,
            _ => false,
        }
    }
    pub fn is_file(&self) -> bool {
        match self {
            Self::FileNode(_) => true,
            _ => false,
        }
    }
    pub fn is_symlink(&self) -> bool {
        match self {
            Self::LinkNode(_) => true,
            _ => false,
        }
    }
    pub fn unwrap_as_file(&self) -> &File {
        match self {
            Self::DirNode(file) => file.as_ref(),
            Self::FileNode(file) => file.as_ref(),
            Self::LinkNode(file) => file.as_ref(),
        }
    }
}

impl AsRef<File> for Directory {
    fn as_ref(&self) -> &File {
        &self.file
    }
}

impl AsRef<File> for File {
    fn as_ref(&self) -> &File {
        self
    }
}

impl AsRef<File> for SymLink {
    fn as_ref(&self) -> &File {
        &self.file
    }
}

impl File {
    fn new(name: String, metadata: Metadata) -> Self {
        Self { name, metadata }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    // ok if I reuse file in Directory and Symlink I run into problems with
    fn from(entry: DirEntry) -> anyhow::Result<Self> {
        let name = match entry.path().file_name().map(|name| name.to_str()) {
            Some(Some(name)) => String::from(name),
            _ => return Err(anyhow!("cant convert to string")),
        };
        let metadata = entry.metadata()?;

        Ok(Self { name, metadata })
    }

    fn is_hidden(&self) -> bool {
        self.name.starts_with('.')
    }
}

impl Directory {
    fn new(name: String, metadata: Metadata) -> Self {
        let total_size = metadata.size();
        Self {
            file: File::new(name, metadata),
            entries: Vec::new(),
            total_size,
            count: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    fn add_node(&mut self, node: FileTree) {
        let (size_increase, count_increase) = match &node {
            FileTree::FileNode(file) => (file.metadata.size(), 1),
            FileTree::DirNode(dir) => (dir.as_ref().metadata.size(), dir.count),
            _ => (0, 0),
        };

        self.total_size += size_increase;
        self.count += count_increase;
        self.entries.push(node);
    }

    fn from(current: &PathBuf, root: &PathBuf) -> io::Result<Self> {
        let name = current
            .to_str()
            .unwrap()
            .strip_prefix(root.to_str().unwrap());

        let Some(name) = name else {
            panic!("{:?} could not convert to string", current);
        };

        let metadata = std::fs::metadata(current)?;
        let total_size = metadata.size();

        Ok(Self {
            file: File::new(String::from(name), metadata),
            entries: Vec::new(),
            total_size,
            count: 0,
        })
    }

    pub fn into_iter(&self) -> core::slice::Iter<'_, FileTree> {
        self.entries.iter()
    }
}

impl SymLink {
    fn new(name: String, metadata: Metadata, target: String) -> Self {
        Self {
            file: File::new(name, metadata),
            target,
        }
    }
}

trait DisplayFileName {
    fn display_file_name(&self) -> String;
}

pub mod walker {
    use crate::Args;

    use super::{Directory, File, FileTree};
    use std::path::PathBuf;

    pub fn get_tree(from: PathBuf, args: &Args) -> anyhow::Result<FileTree> {
        walk_dir(&from, &from, args)
    }

    fn walk_dir(current: &PathBuf, root: &PathBuf, args: &Args) -> anyhow::Result<FileTree> {
        let dir_entry = current.read_dir()?;
        let mut dir = Directory::from(current, root)?;

        for file in dir_entry {
            let file = file?;
            let file_type = file.file_type()?;

            match file.file_name().to_str() {
                Some(name) if name.starts_with('.') => {
                    if !args.all {
                        continue;
                    }
                }
                _ => (),
            };

            if file_type.is_dir() {
                if args.recursive {
                    dir.add_node(walk_dir(&file.path(), current, args)?);
                } else {
                    let new_dir = Directory::from(&file.path(), root)?;
                    dir.add_node(FileTree::DirNode(new_dir));
                }
            } else if file_type.is_file() {
                dir.add_node(FileTree::FileNode(File::from(file)?));
            } else if file_type.is_symlink() {
            }
        }

        Ok(FileTree::DirNode(dir))
    }
}
