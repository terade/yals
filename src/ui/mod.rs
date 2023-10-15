mod size;

use std::os::unix::fs::MetadataExt;

use crate::filetree::{Directory, File, FileTree};
use crate::Args;
use ansi_term::Color::Blue;
use size::PrettySize;

const PERM_EXECUTE: u32 = 1;
const PERM_WRITE: u32 = 2;
const PERM_READ: u32 = 4;

const PERM_USER_SHIFT: u32 = 6;
const PERM_GROUP_SHIFT: u32 = 3;
const PERM_OTHER_SHIFT: u32 = 0;

trait ToString {
    fn to_string(&self, args: &Args) -> String;
}

impl ToString for File {
    fn to_string(&self, args: &Args) -> String {
        if args.long {
            format!("{} {}", permission_string(self), self.name())
        } else {
            format!("{}", self.name())
        }
    }
}

impl ToString for Directory {
    fn to_string(&self, args: &Args) -> String {
        let name = self.as_ref().name();
        if args.long {
            format!("{} {}", permission_string(self), name)
        } else {
            format!("{}", Blue.paint(name))
        }
    }
}

impl FileTree {
    pub fn ls_print(tree: &FileTree, args: &Args) {
        //Self::ls_print_dir(tree, args);
    }

    fn ls_print_dir(directory: Directory, args: &Args) {
        let mut backlog = Vec::new();
        if !directory.as_ref().name().is_empty() {
            println!("{}:", directory.to_string(args));
        }

        for file in directory.into_iter() {
            match file {
                Self::DirNode(dir_entry) if args.recursive => backlog.push(dir_entry),
                _ => (),
            }
        }
    }

    fn ls_print_directory(tree: &FileTree, args: &Args) {}
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
    res += &permission_string_by_group(mode, PERM_USER_SHIFT);
    res += &permission_string_by_group(mode, PERM_GROUP_SHIFT);
    res += &permission_string_by_group(mode, PERM_OTHER_SHIFT);

    res = format!("{} {} {} {}", res, user_name, group_name, metadata.size());

    res
}

fn permission_string_by_group(mode: u32, shift: u32) -> String {
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
