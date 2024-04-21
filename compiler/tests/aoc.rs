#![cfg(unix)]

use compiler::compile;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use test_each_file::test_each_path;

fn aoc([program_path, output_path]: [&Path; 2]) {
    let tempdir = TempDir::with_prefix("spike-aoc").unwrap();
    let program = fs::read_to_string(program_path).unwrap();
    let mut input = PathBuf::from(program_path);
    input.set_file_name("input");

    compile(
        &program,
        program_path.file_name().unwrap().to_str().unwrap(),
        &tempdir.path().join("output"),
    )
    .unwrap();

    // Make output executable.
    Command::new("chmod")
        .current_dir(&tempdir)
        .arg("+x")
        .arg("./output")
        .output()
        .unwrap();

    let create_child = || {
        Command::new("./output")
            .current_dir(&tempdir)
            .stdin(Stdio::from(File::open(&input).unwrap()))
            .stdout(Stdio::piped())
            .spawn()
    };

    // Wait for output to be executable.
    let child = loop {
        if let Ok(child) = create_child() {
            break child;
        }
    };

    assert_eq!(
        child.wait_with_output().unwrap().stdout,
        fs::read(output_path).unwrap()
    );
}

test_each_path! { for ["sp", "out"] in "./programs/aoc/" => aoc }
