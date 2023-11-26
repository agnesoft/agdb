pub(crate) fn unquote(value: &str) -> &str {
    value.trim_start_matches('"').trim_end_matches('"')
}
