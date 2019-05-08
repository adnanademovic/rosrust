use rosrust_diagnostics::{Level, Status};

#[test]
fn test_init_empty() {
    let status = Status::default();
    assert_eq!(status.level, Level::Ok);
    assert_eq!(status.message, "");
    assert!(status.values.is_empty());
}

#[test]
fn test_init_lvl_msg() {
    let status = Status::new(Level::Warn, "test");
    assert_eq!(status.level, Level::Warn);
    assert_eq!(status.message, "test");
    assert!(status.values.is_empty());
}

#[test]
fn test_summary_lvl_msg() {
    let mut status = Status::default();
    status.set_summary(Level::Warn, "test");
    assert_eq!(status.level, Level::Warn);
    assert_eq!(status.message, "test");
}

#[test]
fn test_summary_dmsg() {
    let mut status = Status::new(Level::Ok, "ok");
    status.copy_summary(&Status::new(Level::Warn, "warn"));
    assert_eq!(status.level, Level::Warn);
    assert_eq!(status.message, "warn");
}

#[test]
fn test_clear_summary() {
    let mut status = Status::new(Level::Ok, "ok");
    status.clear_summary();
    assert_eq!(status.level, Level::Ok);
    assert_eq!(status.message, "");
}

#[test]
fn test_merge_summary_lvl_msg() {
    let mut status = Status::new(Level::Ok, "ok");
    status.merge_summary(Level::Warn, "warn");
    assert_eq!(status.level, Level::Warn);
    assert_eq!(status.message, "warn");
    status.merge_summary(Level::Error, "err");
    assert_eq!(status.level, Level::Error);
    assert_eq!(status.message, "warn; err");
}

#[test]
fn test_merge_summary_dmsg() {
    let mut status = Status::new(Level::Ok, "ok");
    status.merge_summary_with(&Status::new(Level::Warn, "warn"));
    assert_eq!(status.level, Level::Warn);
    assert_eq!(status.message, "warn");
    status.merge_summary_with(&Status::new(Level::Error, "err"));
    assert_eq!(status.level, Level::Error);
    assert_eq!(status.message, "warn; err");
}

#[test]
fn test_add() {
    let mut status = Status::default();
    status.add("key", "val");
    assert_eq!(status.values[0].key, "key");
    assert_eq!(status.values[0].value, "val");
}

#[test]
fn test_extensive() {
    let mut status = Status::default();

    status.set_summary(Level::Warn, "dummy");
    assert_eq!(
        "dummy", status.message,
        "Status::set_summary failed to set message"
    );
    assert_eq!(
        Level::Warn,
        status.level,
        "Status::set_summary failed to set level"
    );

    status.add("toto", format!("{:.1}", 5.0));
    status.add("baba", 5);
    status.add("foo", "bar");
    status.add("bool", true);
    status.add("bool2", false);

    assert_eq!(
        "5.0", status.values[0].value,
        "Bad value, adding a string with add"
    );
    assert_eq!(
        "5", status.values[1].value,
        "Bad value, adding a value with add"
    );
    assert_eq!(
        "bar", status.values[2].value,
        "Bad value, adding a string with add"
    );
    assert_eq!(
        "toto", status.values[0].key,
        "Bad label, adding a string with add"
    );
    assert_eq!(
        "baba", status.values[1].key,
        "Bad label, adding a value with add"
    );
    assert_eq!(
        "foo", status.values[2].key,
        "Bad label, adding a string with add"
    );

    assert_eq!(
        "bool", status.values[3].key,
        "Bad label, adding a true bool key with add"
    );
    assert_eq!(
        "true", status.values[3].value,
        "Bad value, adding a true bool with add"
    );

    assert_eq!(
        "bool2", status.values[4].key,
        "Bad label, adding a false bool key with add"
    );
    assert_eq!(
        "false", status.values[4].value,
        "Bad value, adding a false bool with add"
    );
}
