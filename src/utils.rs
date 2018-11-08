/// Print error and all its causes to STDERR. Could be changed into normal logging calls later.
pub fn log_error_trace(error: &dyn failure::Fail) {
    eprintln!("ERROR: {}", error);
    let mut indent = String::new();
    for cause in error.iter_causes() {
        indent.push_str("  ");
        eprintln!("{}Caused by: {}", indent, cause);
    }
}
