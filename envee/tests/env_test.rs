use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use tempfile::tempdir;

use envee::env::Resolver;

#[test]
fn test_basic() {
    let actual = resolve(&["name = \"john\""]).unwrap();
    let expected: HashMap<String, String> = [("NAME".into(), "john".into())].into();
    assert_eq!(expected, actual);
}

#[test]
fn test_nested() {
    let actual = resolve(&[
        "me.name = \"jane\"",
        "me.age = 75",
        "[server]",
        "ip = \"127.0.0.1\"",
        "port = 8080",
        "[aws]",
        "region = \"us-east-1\"",
        "[aws.s3]",
        "bucket = \"bucket\"",
        "prefix = \"some/prefix/\"",
    ])
    .unwrap();
    let expected: HashMap<String, String> = [
        ("ME_NAME".into(), "jane".into()),
        ("ME_AGE".into(), "75".into()),
        ("SERVER_IP".into(), "127.0.0.1".into()),
        ("SERVER_PORT".into(), "8080".into()),
        ("AWS_REGION".into(), "us-east-1".into()),
        ("AWS_S3_BUCKET".into(), "bucket".into()),
        ("AWS_S3_PREFIX".into(), "some/prefix/".into()),
    ]
    .into();
    assert_eq!(expected, actual);
}

#[test]
fn test_expansion() {
    unsafe {
        std::env::set_var("TEST", "EXPANDED");
    }
    let actual = resolve(&[
        "a.a = \"${TEST}\"",
        "a.b = \"${A_A}\"",
        "a.c = \"${A_B}\"",
        "a.d = \"${A_C}\"",
    ])
    .unwrap();
    let expected: HashMap<String, String> = [
        ("A_A".into(), "EXPANDED".into()),
        ("A_B".into(), "EXPANDED".into()),
        ("A_C".into(), "EXPANDED".into()),
        ("A_D".into(), "EXPANDED".into()),
    ]
    .into();
    assert_eq!(expected, actual);
}

fn resolve(lines: &[&str]) -> Result<HashMap<String, String>> {
    let root = tempdir().unwrap();
    assert!(root.path().is_dir());
    let path = root.path().join("test.toml");
    let mut file = File::create(&path).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
    Resolver::new(vec![path]).get()
}
