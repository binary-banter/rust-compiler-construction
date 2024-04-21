#![cfg(unix)]

use compiler::compile;
use compiler::utils::split_test::{split_test, str_to_int};
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use test_each_file::test_each_file;

fn integration([test]: [&str; 1]) {
    let (input, expected_output, expected_return, _) = split_test(test);

    let tempdir = TempDir::with_prefix("spike-integration").unwrap();

    compile(test, "<test>", &tempdir.path().join("output")).unwrap();

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
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    };

    // Wait for output to be executable.
    let mut child = loop {
        if let Ok(child) = create_child() {
            break child;
        }
    };

    let mut stdin = child.stdin.take().unwrap();

    // Write all inputs to stdin of child process.
    for num in input {
        writeln!(stdin, "{num}").unwrap();
    }

    let out = child.wait_with_output().unwrap();

    // Assert the program exits with its return value.
    assert_eq!(
        out.status.code().unwrap() as i64 & 0xFF,
        expected_return & 0xFF
    );

    // Assert all output was as expected.
    for (got, expected) in out.stdout.lines().map(Result::unwrap).zip(expected_output) {
        assert_eq!(str_to_int(&got), expected);
    }
}

test_each_file! { for ["sp"] in "./programs/good" => integration }
