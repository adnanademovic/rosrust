use rosrust_diagnostics::{FunctionExt, Level, Status, Task};

#[test]
fn simple_function_task_works() {
    let task = (|status: &mut Status| {
        status.set_summary(Level::Warn, "foo");
        status.name = "bar".into();
        status.hardware_id = "baz".into();
        status.values.clear();
        status.add("one", 1);
        status.add("true_bool", true);
    })
    .into_task("my_task");

    let mut target = Status::default();

    task.run(&mut target);

    assert_eq!(&target.message, "foo");
    assert_eq!(target.level, Level::Warn);
    assert_eq!(&target.name, "bar");
    assert_eq!(&target.hardware_id, "baz");
    assert_eq!(target.values.len(), 2);
    assert_eq!(target.values[0].key, "one");
    assert_eq!(target.values[0].value, "1");
    assert_eq!(target.values[1].key, "true_bool");
    assert_eq!(target.values[1].value, "true");
}
