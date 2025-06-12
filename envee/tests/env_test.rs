use std::fs::File;
use std::io::Write;

use tempfile::tempdir;

use envee::env::Resolver;

#[test]
fn test_basic() {
    test(&["name = \"john\""], &[("NAME", "john")]);
}

#[test]
fn test_nested() {
    test(
        &[
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
        ],
        &[
            ("ME_NAME", "jane"),
            ("ME_AGE", "75"),
            ("SERVER_IP", "127.0.0.1"),
            ("SERVER_PORT", "8080"),
            ("AWS_REGION", "us-east-1"),
            ("AWS_S3_BUCKET", "bucket"),
            ("AWS_S3_PREFIX", "some/prefix/"),
        ],
    );
}

#[test]
fn test_expansion() {
    unsafe {
        std::env::set_var("KEY", "a");
    }
    test(
        &[
            "a = \"${KEY}\"",
            "b = \"${A}b\"",
            "c = \"${B}c\"",
            "d = \"${E}d\"",
            "e = \"${C}e\"",
        ],
        &[
            ("A", "a"),
            ("B", "ab"),
            ("C", "abc"),
            ("D", "d"),
            ("E", "abce"),
        ],
    );
}

fn test(lines: &[&str], expected: &[(&str, &str)]) {
    let root = tempdir().unwrap();
    assert!(root.path().is_dir());
    let path = root.path().join("test.toml");
    let mut file = File::create(&path).unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
    let actual = Resolver::new(vec![path]).get().unwrap();
    let expected: Vec<_> = expected
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    assert_eq!(expected, actual);
}
