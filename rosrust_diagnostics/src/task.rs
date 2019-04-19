use crate::Status;

pub trait Task {
    fn name(&self) -> &str;
    fn run(&self, status: &mut Status);
}
