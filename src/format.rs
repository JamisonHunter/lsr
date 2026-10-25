use std::path::Path;

const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
const BASE: f64 = 1_000.0;

/// Formats a logical byte count using decimal SI units.
///
/// 1 kB is 1,000 bytes. Values below 1 kB are shown as whole bytes.
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1_000 {
        return format!("{} B", bytes);
    }

    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= BASE && unit < UNITS.len() - 1 {
        value /= BASE;
        unit += 1;
    }

    let mut rendered = format!("{value:.1}");

    // Avoid outputs such as "1000.0 kB" when one decimal place would round
    // up to the next unit.
    if rendered == "1000.0" && unit < UNITS.len() - 1 {
        value /= BASE;
        unit += 1;
        rendered = format!("{value:.1}");
    }

    format!("{rendered} {}", UNITS[unit])
}

/// Escapes characters that could break the output layout or inject terminal
/// control sequences, while leaving printable Unicode names intact.
pub fn escape_name(name: &str) -> String {
    let mut escaped = String::with_capacity(name.len());

    for character in name.chars() {
        if character == '\\' {
            escaped.push_str("\\\\");
        } else if character.is_control() {
            escaped.extend(character.escape_default());
        } else {
            escaped.push(character);
        }
    }

    escaped
}

pub fn escape_path(path: &Path) -> String {
    escape_name(&path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_below_one_kilobyte() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(999), "999 B");
    }

    #[test]
    fn format_bytes_unit_boundaries() {
        assert_eq!(format_bytes(1_000), "1.0 kB");
        assert_eq!(format_bytes(1_000_000), "1.0 MB");
        assert_eq!(format_bytes(1_000_000_000), "1.0 GB");
        assert_eq!(format_bytes(1_000_000_000_000), "1.0 TB");
    }

    #[test]
    fn format_bytes_rounds_to_one_decimal() {
        assert_eq!(format_bytes(1_550), "1.6 kB");
        assert_eq!(format_bytes(1_000_050), "1.0 MB");
    }

    #[test]
    fn format_bytes_promotes_after_rounding() {
        assert_eq!(format_bytes(999_999), "1.0 MB");
        assert_eq!(format_bytes(999_999_999_999), "1.0 TB");
    }

    #[test]
    fn format_bytes_caps_at_terabytes() {
        assert!(format_bytes(u64::MAX).ends_with(" TB"));
    }

    #[test]
    fn escape_name_leaves_normal_text_alone() {
        assert_eq!(escape_name("normal.txt"), "normal.txt");
        assert_eq!(escape_name("héllo"), "héllo");
    }

    #[test]
    fn escape_name_escapes_control_characters() {
        assert_eq!(escape_name("a\nb"), "a\\nb");
        assert_eq!(escape_name("a\tb"), "a\\tb");
        assert_eq!(escape_name("a\rb"), "a\\rb");
        assert_eq!(escape_name("a\x1b[31mred"), "a\\u{1b}[31mred");
    }

    #[test]
    fn escape_name_escapes_backslashes() {
        assert_eq!(escape_name("a\\b"), "a\\\\b");
        assert_eq!(escape_name("a\\nb"), "a\\\\nb");
    }
}
