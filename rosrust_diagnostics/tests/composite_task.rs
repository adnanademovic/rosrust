use rosrust_diagnostics::{CompositeTask, FunctionExt, Level, Status, Task};

#[test]
fn oks_get_ignored_when_issues_arise() {
    let task_noop = (|_: &mut Status| {}).into_task("task_noop");

    let task1 = (|status: &mut Status| {
        status.set_summary(Level::Warn, "foo");
        status.add("one", 1);
    })
    .into_task("my_task1");

    let task2 = (|status: &mut Status| {
        status.set_summary(Level::Error, "bar");
        status.add("two", 2);
    })
    .into_task("my_task2");

    let task3 = (|status: &mut Status| {
        status.set_summary(Level::Ok, "baz");
        status.add("three", 3);
    })
    .into_task("my_task3");

    let mut target = Status::default();
    target.set_summary(Level::Ok, "start");

    let mut task = CompositeTask::new("composite_task");
    task.add_task(task_noop.clone());
    task.add_task(task1);
    task.add_task(task2);
    task.add_task(task3);
    task.add_task(task_noop);

    task.run(&mut target);

    assert_eq!(&target.message, "foo; bar");
    assert_eq!(target.level, Level::Error);
    assert_eq!(target.values.len(), 3);
    assert_eq!(target.values[0].key, "one");
    assert_eq!(target.values[0].value, "1");
    assert_eq!(target.values[1].key, "two");
    assert_eq!(target.values[1].value, "2");
    assert_eq!(target.values[2].key, "three");
    assert_eq!(target.values[2].value, "3");
}

#[test]
fn oks_get_perserved_when_there_are_no_issues() {
    let task_noop = (|_: &mut Status| {}).into_task("task_noop");

    let task1 = (|status: &mut Status| {
        status.set_summary(Level::Ok, "foo");
        status.add("one", 1);
    })
    .into_task("my_task1");

    let task2 = (|status: &mut Status| {
        status.set_summary(Level::Ok, "bar");
        status.add("two", 2);
    })
    .into_task("my_task2");

    let task3 = (|status: &mut Status| {
        status.set_summary(Level::Ok, "baz");
        status.add("three", 3);
    })
    .into_task("my_task3");

    let mut target = Status::default();
    target.set_summary(Level::Ok, "start");

    let mut task = CompositeTask::new("composite_task");
    task.add_task(task_noop.clone());
    task.add_task(task1);
    task.add_task(task2);
    task.add_task(task3);
    task.add_task(task_noop);

    task.run(&mut target);

    assert_eq!(&target.message, "start; foo; bar; baz; start");
    assert_eq!(target.level, Level::Ok);
    assert_eq!(target.values.len(), 3);
    assert_eq!(target.values[0].key, "one");
    assert_eq!(target.values[0].value, "1");
    assert_eq!(target.values[1].key, "two");
    assert_eq!(target.values[1].value, "2");
    assert_eq!(target.values[2].key, "three");
    assert_eq!(target.values[2].value, "3");
}
