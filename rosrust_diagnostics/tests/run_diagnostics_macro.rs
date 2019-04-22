use rosrust_diagnostics::{run_diagnostics, Level, Status, Task};

struct SubsystemNoop;

impl Task for SubsystemNoop {
    fn run(&self, _: &mut Status) {}
}

struct SubsystemPreset {
    level: Level,
    message: String,
    additions: Vec<(String, String)>,
}

impl Task for SubsystemPreset {
    fn run(&self, status: &mut Status) {
        status.set_summary(self.level, &self.message);
        for (key, value) in &self.additions {
            status.add(key.clone(), value);
        }
    }
}

struct System {
    pub task_noop: SubsystemNoop,
    pub task1: SubsystemPreset,
    pub task2: SubsystemPreset,
    pub task3: SubsystemPreset,
}

impl Task for System {
    fn run(&self, status: &mut Status) {
        run_diagnostics!(
            status,
            self.task_noop,
            self.task1,
            self.task2,
            self.task3,
            self.task_noop
        );
    }
}

#[test]
fn oks_get_ignored_when_issues_arise() {
    let system = System {
        task_noop: SubsystemNoop,
        task1: SubsystemPreset {
            level: Level::Warn,
            message: "foo".into(),
            additions: vec![("one".into(), "1".into())],
        },
        task2: SubsystemPreset {
            level: Level::Error,
            message: "bar".into(),
            additions: vec![("two".into(), "2".into())],
        },
        task3: SubsystemPreset {
            level: Level::Ok,
            message: "baz".into(),
            additions: vec![("three".into(), "3".into())],
        },
    };

    let mut target = Status::default();
    target.set_summary(Level::Ok, "start");

    system.run(&mut target);

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
    let system = System {
        task_noop: SubsystemNoop,
        task1: SubsystemPreset {
            level: Level::Ok,
            message: "foo".into(),
            additions: vec![("one".into(), "1".into())],
        },
        task2: SubsystemPreset {
            level: Level::Ok,
            message: "bar".into(),
            additions: vec![("two".into(), "2".into())],
        },
        task3: SubsystemPreset {
            level: Level::Ok,
            message: "baz".into(),
            additions: vec![("three".into(), "3".into())],
        },
    };

    let mut target = Status::default();
    target.set_summary(Level::Ok, "start");

    system.run(&mut target);

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
