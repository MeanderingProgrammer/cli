mod common;

use std::path::PathBuf;

use common::TestData;
use copy_folder::transfer::Transfer;

#[test]
fn test_empty() {
    test(&[], &[]);
}

#[test]
fn test_file() {
    test(&["file.txt"], &[]);
}

#[test]
fn test_nested_file() {
    test(&["a/file.txt"], &[]);
}

#[test]
fn test_many() {
    test(
        &[
            "file-1.txt",
            "file-2.txt",
            "a/file-1.txt",
            "a/file-2.txt",
            "a/b/files-1.txt",
            "a/b/c/files-1.txt",
            "a/b/c/d/e/f/g/h/files-1.txt",
        ],
        &[],
    );
}

#[test]
fn test_identical() {
    test(&["file-1.txt", "file-2.txt"], &["file-1.txt", "file-2.txt"]);
}

#[test]
fn test_disjoint() {
    test(&["file-1.txt", "file-2.txt"], &["file-3.txt", "file-4.txt"]);
}

#[test]
fn test_some_overlap() {
    test(&["file-1.txt", "file-2.txt"], &["file-1.txt", "file-3.txt"]);
}

fn test(src: &[&str], dest: &[&str]) {
    let src = TestData::new(src);
    let dest = TestData::new(dest);

    // data already in destination should never change
    let mut expected = src.initial();
    expected.extend(dest.initial());

    // run transfer for all source files to destination
    Transfer::new(src.path(), dest.path())
        .run(src.initial().keys().map(PathBuf::from))
        .unwrap();

    // compare contents in destination after transfer
    assert_eq!(expected, dest.current());
}
