mod size;

use std::os::unix::fs::MetadataExt;

use crate::filetree::{Directory, File, FileTree};
use crate::Args;
use ansi_term::Color::{Blue, White};
use size::PrettySize;

const PERM_EXECUTE: u32 = 1;
const PERM_WRITE: u32 = 2;
const PERM_READ: u32 = 4;

const PERM_USER_SHIFT: u32 = 6;
const PERM_GROUP_SHIFT: u32 = 3;
const PERM_OTHER_SHIFT: u32 = 0;

trait ToString {
    fn to_string(&self, args: &Args, padding: usize) -> String;
}

impl ToString for File {
    // first concentrate on non long version
    fn to_string(&self, args: &Args, padding: usize) -> String {
        let name = if self.metadata().is_dir() {
            format!("{}/", Blue.paint(self.name().to_string()))
        } else {
            White.paint(self.name()).to_string()
        };

        if args.long {
            format!("{} {}", permission_string(self), self.name())
        } else {
            name.to_string()
        }
    }
}

impl FileTree {
    pub fn ls_print(tree: &FileTree, args: &Args) {
        match tree {
            FileTree::DirNode(dir) => Self::ls_print_dir(dir, ".", args),
            _ => (),
        }
    }

    fn ls_print_dir(directory: &Directory, in_dir: &str, args: &Args) {
        let mut backlog = Vec::new();
        let mut max_link = 0;
        let mut max_size = 0;

        if args.recursive {
            println!("{}:", in_dir);
        }

        for file in directory.into_iter() {
            match file {
                Self::DirNode(dir_entry) if args.recursive => backlog.push(dir_entry),
                _ => (),
            }
            let metadata = match file {
                Self::DirNode(file) => file.as_ref().metadata(),
                Self::FileNode(file) => file.as_ref().metadata(),
                Self::LinkNode(file) => file.as_ref().metadata(),
            };

            if metadata.size() < max_size {
                max_size = metadata.size();
            }
            if metadata.nlink() < max_link {
                max_link = metadata.nlink();
            }
        }

        // TODO refactor everything in this function after this
        let padding_link = 0;
        let padding_size = 0;

        let output = directory
            .into_iter()
            .map(|file| file.unwrap_as_file().to_string(args, padding_link))
            .collect::<Vec<String>>();

        if !output.is_empty() {
            println!("{}", output.join(if args.long { "\n" } else { "  " }));
        }

        if !backlog.is_empty() && args.recursive {
            println!();
        }

        for (num, entry) in backlog.iter().enumerate() {
            let relative_path_current_dir = format!("{}/{}", in_dir, entry.as_ref().name());
            Self::ls_print_dir(entry, &relative_path_current_dir, args);

            if num < (backlog.len() - 1) {
                println!();
            }
        }
    }
}

fn permission_string<T: AsRef<File>>(file: T) -> String {
    let metadata = file.as_ref().metadata();
    let mode = metadata.mode();
    let user_uid = metadata.uid();
    let group_gid = metadata.gid();
    let mut res = String::from("");

    let user_bind = users::get_user_by_uid(user_uid).unwrap();
    let user_name = user_bind.name().to_str().unwrap();

    let group_bind = users::get_group_by_gid(group_gid).unwrap();
    let group_name = group_bind.name().to_str().unwrap();

    res += if metadata.is_dir() {
        "d"
    } else if metadata.is_symlink() {
        "l"
    } else {
        "-"
    };
    res += &get_permission_string_by_group(mode, PERM_USER_SHIFT);
    res += &get_permission_string_by_group(mode, PERM_GROUP_SHIFT);
    res += &get_permission_string_by_group(mode, PERM_OTHER_SHIFT);

    res = format!("{} {} {} {}", res, user_name, group_name, metadata.size());

    res
}

fn get_permission_string_by_group(mode: u32, shift: u32) -> String {
    let mut res = String::new();

    res += if (mode & (PERM_READ << shift)) != 0 {
        "r"
    } else {
        "-"
    };
    res += if (mode & (PERM_WRITE << shift)) != 0 {
        "w"
    } else {
        "-"
    };
    res += if (mode & (PERM_EXECUTE << shift)) != 0 {
        "x"
    } else {
        "-"
    };

    res
}
