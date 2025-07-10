pub fn last_n_chars(s: &str, n: usize) -> &str {
    let len = s.len();
    if len <= n {
        s
    } else {
        &s[len - n..]
    }
}