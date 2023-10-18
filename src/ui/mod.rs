mod size;

use std::os::unix::fs::MetadataExt;

use crate::filetree::{Directory, File, FileTree};
use crate::Args;
use ansi_term::Color::{Blue, White};
use chrono::prelude::{DateTime, Local};
use size::PrettySize;
use std::fs::Metadata;

const PERM_EXECUTE: u32 = 1;
const PERM_WRITE: u32 = 2;
const PERM_READ: u32 = 4;

const PERM_USER_SHIFT: u32 = 6;
const PERM_GROUP_SHIFT: u32 = 3;
const PERM_OTHER_SHIFT: u32 = 0;

const BLOCK_SIZE: u64 = 1024;

trait ToString {
    fn to_string(&self, args: &Args, padding_link: usize, padding_size: usize) -> String;
}

impl ToString for File {
    // first concentrate on non long version
    fn to_string(&self, args: &Args, padding_link: usize, padding_size: usize) -> String {
        let mut res = String::new();
        let name = if self.metadata().is_dir() {
            format!("{}/", Blue.paint(self.name().to_string()))
        } else {
            White.paint(self.name()).to_string()
        };
        if args.size {
            res = format!("{} ", self.as_ref().metadata().blocks() * 512 / BLOCK_SIZE);
        }

        if args.long {
            res += &permission_string(self, args, padding_link, padding_size);
            res += " ";
        }

        res += &name;
        res
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
        let mut max_link = 0_f64;
        let mut max_size_representation = 0;

        if args.recursive {
            println!("{}:", in_dir);
        }

        if args.long {
            println!(
                "total {}{}",
                (directory.total_size() / BLOCK_SIZE) as usize,
                if args.human_readable { "K" } else { "" }
            );
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

            let pretty_size_len =
                PrettySize::from_bytes_with_style(metadata.size(), args.human_readable)
                    .to_string()
                    .len();

            if pretty_size_len > max_size_representation {
                max_size_representation = pretty_size_len;
            }
            if (metadata.nlink() as f64) > max_link {
                max_link = metadata.nlink() as f64;
            }
        }

        // TODO refactor everything in this function after this
        let padding_link = max_link.log10().floor() as usize;
        let padding_size = max_size_representation;

        let output = directory
            .into_iter()
            .map(|file| {
                file.unwrap_as_file()
                    .to_string(args, padding_link, padding_size)
            })
            .collect::<Vec<String>>();

        if !output.is_empty() {
            println!("{}", output.join(if args.long || args.one_file_per_line { "\n" } else { "  " }));
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

fn permission_string<T: AsRef<File>>(
    file: T,
    args: &Args,
    padding_link: usize,
    padding_size: usize,
) -> String {
    let metadata = file.as_ref().metadata();
    let mode = metadata.mode();
    let user_uid = metadata.uid();
    let group_gid = metadata.gid();
    let nlinks = metadata.nlink();
    let mut res = String::new();

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

    res = format!(
        "{} {}{} {} {} {} {}",
        res,
        " ".repeat(padding_link - (nlinks as f64).log10().floor() as usize),
        nlinks,
        user_name,
        group_name,
        get_size_string(metadata, padding_size, args),
        get_modified_string(metadata)
    );

    res
}

fn get_size_string(metadata: &Metadata, padding: usize, args: &Args) -> String {
    let rep = PrettySize::from_bytes_with_style(metadata.size(), args.human_readable).to_string();
    let padding = " ".repeat(padding - rep.len());

    format!("{}{}", padding, rep)
}

fn get_modified_string(metadata: &Metadata) -> String {
    match metadata.modified() {
        Ok(modified) => {
            let dt: DateTime<Local> = modified.into();
            format!("{}", dt.format("%d. %b %R"))
        }
        Err(_err) => String::from("time not supported on this platform"),
    }
}

fn get_permission_string_by_group(mode: u32, shift: u32) -> String {
    format!(
        "{}{}{}",
        if (mode & (PERM_READ << shift)) != 0 {
            "r"
        } else {
            "-"
        },
        if (mode & (PERM_WRITE << shift)) != 0 {
            "w"
        } else {
            "-"
        },
        if (mode & (PERM_EXECUTE << shift)) != 0 {
            "x"
        } else {
            "-"
        }
    )
}
