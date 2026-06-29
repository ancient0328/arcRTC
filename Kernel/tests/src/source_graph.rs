//! dependency / source-shape gate 用の小さな helper です。

use crate::{files_named, impl_relative, implementation_root, read_file};

pub fn implementation_files_named(file_name: &str) -> Vec<(String, String)> {
    files_named(&implementation_root(), file_name)
        .into_iter()
        .filter(|path| !impl_relative(path).starts_with("target/"))
        .map(|path| {
            let relative = impl_relative(&path);
            let content = read_file(&path);
            (relative, content)
        })
        .collect()
}

pub fn count_forbidden_markers(content: &str, forbidden: &[&str]) -> usize {
    forbidden
        .iter()
        .filter(|marker| content.contains(**marker))
        .count()
}
