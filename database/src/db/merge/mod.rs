pub mod vorgang;

pub enum MergeState<T> {
    AmbiguousMatch(Vec<T>),
    OneMatch(T),
    NoMatch,
}

#[cfg(test)]
pub(crate) fn display_strdiff(expected: &str, got: &str) -> String {
    use similar::ChangeTag;
    let diff = similar::TextDiff::from_lines(expected, got);
    let mut s = "Line |+-| Change\n-----|--|-------".to_string();
    let mut diffiter = diff
        .iter_all_changes()
        .filter(|x| x.tag() != ChangeTag::Equal);
    let mut current_sign = ChangeTag::Equal;
    while let Some(el) = diffiter.next() {
        let sign = match el.tag() {
            ChangeTag::Equal => continue,
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
        };
        if el.tag() != current_sign {
            s = format!(
                "{}\n{}{:04}| {}| {}",
                s,
                if el.old_index().is_some() { "e" } else { "g" },
                el.old_index().unwrap_or(el.new_index().unwrap_or(0)),
                sign,
                el.value().trim()
            );
            current_sign = el.tag();
        } else {
            s = format!("{}{}", s, el.value());
        }
    }
    format!("{}\n-----------------------------------", s)
}
