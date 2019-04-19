use crate::{Status, Task};

pub struct FunctionTask<F>
where
    F: Fn(&mut Status),
{
    name: String,
    function: F,
}

impl<F> FunctionTask<F>
where
    F: Fn(&mut Status),
{
    #[inline]
    pub fn new(name: &str, function: F) -> Self {
        Self {
            name: name.into(),
            function,
        }
    }
}

impl<F> Task for FunctionTask<F>
where
    F: Fn(&mut Status),
{
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn run(&self, status: &mut Status) {
        (self.function)(status)
    }
}

pub trait FunctionExt: Sized + Fn(&mut Status) {
    fn into_task(self, name: &str) -> FunctionTask<Self>;
}

impl<F> FunctionExt for F
where
    F: Fn(&mut Status),
{
    fn into_task(self, name: &str) -> FunctionTask<Self> {
        FunctionTask::new(name, self)
    }
}
