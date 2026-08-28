//! Shape tolerance for the values an operation parameter accepts.
//!
//! An operation that takes a list of refs accepts several shapes for the same
//! meaning — the "forgiving input" contract a tool's schema advertises under
//! `x-forgiving-input`. One of those shapes needs a reading that every surface
//! must share, and this module owns it.
//!
//! A shell argument carries exactly one string, and so does a JSON string. A
//! caller who means several entries therefore writes them as a JSON array
//! inside that one value — `--tags '["red","blue"]'`. Both the CLI argument
//! extractor ([`crate::cli_gen`]) and an operation's own parameter reader call
//! [`expand_list_entry`], so one value means the same list on the command line
//! as it does over MCP.

/// Expand one raw list value into the entries it names.
///
/// A string that parses as a JSON array of strings expands to those strings.
/// Every other string is one entry, kept whole — bracket characters are legal
/// inside a ref, and a JSON array that holds anything but strings names no
/// entries at all.
///
/// So `["red","blue"]` names two entries, `red` names one, and `[1,2]` names
/// itself. The tests below hold each of those readings.
pub fn expand_list_entry(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_else(|_| vec![raw.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stringified_array_expands_to_its_elements() {
        assert_eq!(expand_list_entry(r#"["red","blue"]"#), ["red", "blue"]);
    }

    #[test]
    fn empty_stringified_array_expands_to_nothing() {
        let empty: [String; 0] = [];
        assert_eq!(expand_list_entry("[]"), empty);
    }

    #[test]
    fn a_plain_ref_is_one_entry() {
        assert_eq!(
            expand_list_entry("01K9ZEPKJ35S76KF7E9HS5742J"),
            ["01K9ZEPKJ35S76KF7E9HS5742J"]
        );
    }

    #[test]
    fn a_value_that_is_not_a_string_array_is_kept_whole() {
        for raw in ["[not json", "[1,2]", r#"{"a":"b"}"#, ""] {
            assert_eq!(expand_list_entry(raw), [raw]);
        }
    }
}
