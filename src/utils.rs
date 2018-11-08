/// Print error and all its causes to STDERR. Could be changed into normal logging calls later.
pub fn log_error_trace(error: &dyn failure::Fail) {
    let mut indent = String::new();

    error!("ERROR: {}", error);
    if let Some(backtrace) = error.backtrace() {
        error!("  Backtrace:");
        for line in backtrace.to_string().split("\n") {
            error!("  {}", line);
        }
    }

    for cause in error.iter_causes() {
        indent.push_str("  ");
        error!("{}Caused by: {}", indent, cause);
        if let Some(backtrace) = cause.backtrace() {
            error!("{}  Backtrace:", indent);
            for line in backtrace.to_string().split("\n") {
                error!("{}  {}", indent, line);
            }
        }
    }
}

/// Print error and all its causes to STDERR if given Result is an Err.
pub fn log_error_trace_if_err<T>(result: &Result<T, impl failure::Fail>) {
    match result {
        Ok(_) => {}
        Err(error) => log_error_trace(error),
    }
}
