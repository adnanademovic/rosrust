#[macro_export]
macro_rules! run_diagnostics {
    ($status: expr, $($tasks:expr),*) => {
        {
            let mut runner = $crate::CompositeTaskRunner::new($status);
            $(
                runner.run(&$tasks);
            )*
            runner.finish();
        }
    };
}
