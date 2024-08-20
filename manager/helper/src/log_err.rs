#[macro_export]
macro_rules! log_err {
    ($result:expr) => {
        if let Err(e) = &$result {
            //tracing::error!("{}", e);
        }
    };
}
