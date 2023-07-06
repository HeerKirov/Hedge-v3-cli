use std::path::PathBuf;

use super::Context;

pub enum ApplyInputType {
    Directory(PathBuf),
    File(PathBuf),
    Input
}

pub fn apply(_context: &Context, _input: ApplyInputType, _quiet: bool) {

}