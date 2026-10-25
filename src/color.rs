#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Default,
    Ansi(u8),
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn parse(value: &str) -> Result<Self, String> {
        let value = value.trim();

        if value.is_empty()
            || value.eq_ignore_ascii_case("default")
            || value.eq_ignore_ascii_case("none")
        {
            return Ok(Color::Default);
        }

        if let Some(hex) = value.strip_prefix('#') {
            return parse_hex(hex);
        }

        if let Some(color) = named(value) {
            return Ok(color);
        }

        if let Ok(index) = value.parse::<u8>() {
            return Ok(Color::Ansi256(index));
        }

        Err(format!("invalid color '{}'", value))
    }

    pub fn prefix(&self) -> String {
        match *self {
            Color::Default => String::new(),
            Color::Ansi(code) => format!("\x1b[{}m", code),
            Color::Ansi256(index) => format!("\x1b[38;5;{}m", index),
            Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
        }
    }

    pub fn paint(&self, text: &str, enabled: bool) -> String {
        if !enabled || matches!(self, Color::Default) {
            return text.to_string();
        }

        format!("{}{}\x1b[0m", self.prefix(), text)
    }
}

fn named(value: &str) -> Option<Color> {
    let code = match value.to_ascii_lowercase().as_str() {
        "black" => 30,
        "red" => 31,
        "green" => 32,
        "yellow" => 33,
        "blue" => 34,
        "magenta" => 35,
        "cyan" => 36,
        "white" => 37,
        "bright_black" => 90,
        "bright_red" => 91,
        "bright_green" => 92,
        "bright_yellow" => 93,
        "bright_blue" => 94,
        "bright_magenta" => 95,
        "bright_cyan" => 96,
        "bright_white" => 97,
        _ => return None,
    };

    Some(Color::Ansi(code))
}

fn parse_hex(hex: &str) -> Result<Color, String> {
    if !hex.is_ascii() {
        return Err(format!(
            "invalid hex color '#{}' (expected #RGB or #RRGGBB using ASCII hex digits)",
            hex
        ));
    }

    match hex.len() {
        3 => {
            let r = hex_digit(&hex[0..1])?;
            let g = hex_digit(&hex[1..2])?;
            let b = hex_digit(&hex[2..3])?;
            Ok(Color::Rgb(r * 17, g * 17, b * 17))
        }
        6 => {
            let r = hex_pair(&hex[0..2])?;
            let g = hex_pair(&hex[2..4])?;
            let b = hex_pair(&hex[4..6])?;
            Ok(Color::Rgb(r, g, b))
        }
        _ => Err(format!(
            "invalid hex color '#{}' (expected #RGB or #RRGGBB)",
            hex
        )),
    }
}

fn hex_digit(digit: &str) -> Result<u8, String> {
    u8::from_str_radix(digit, 16).map_err(|_| format!("invalid hex digit '{}'", digit))
}

fn hex_pair(pair: &str) -> Result<u8, String> {
    u8::from_str_radix(pair, 16).map_err(|_| format!("invalid hex pair '{}'", pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(Color::parse("#5FB88F").unwrap(), Color::Rgb(95, 184, 143));
        assert_eq!(Color::parse("#fff").unwrap(), Color::Rgb(255, 255, 255));
        assert_eq!(
            Color::parse("  #7cc4ff  ").unwrap(),
            Color::Rgb(124, 196, 255)
        );
    }

    #[test]
    fn parses_named_and_palette_colors() {
        assert_eq!(Color::parse("green").unwrap(), Color::Ansi(32));
        assert_eq!(Color::parse("BRIGHT_CYAN").unwrap(), Color::Ansi(96));
        assert_eq!(Color::parse("141").unwrap(), Color::Ansi256(141));
    }

    #[test]
    fn parses_default_color() {
        assert_eq!(Color::parse("").unwrap(), Color::Default);
        assert_eq!(Color::parse("default").unwrap(), Color::Default);
        assert_eq!(Color::parse("none").unwrap(), Color::Default);
    }

    #[test]
    fn rejects_invalid_colors() {
        assert!(Color::parse("#12").is_err());
        assert!(Color::parse("#gggggg").is_err());
        assert!(Color::parse("not_a_color").is_err());
        assert!(Color::parse("256").is_err());
    }

    #[test]
    fn rejects_non_ascii_hex_without_panicking() {
        assert!(Color::parse("#€").is_err());
        assert!(Color::parse("#€€").is_err());
        assert!(Color::parse("#é").is_err());
        assert!(Color::parse("#１２３").is_err());
    }

    #[test]
    fn builds_ansi_prefixes() {
        assert_eq!(Color::Default.prefix(), "");
        assert_eq!(Color::Ansi(96).prefix(), "\x1b[96m");
        assert_eq!(Color::Ansi256(141).prefix(), "\x1b[38;5;141m");
        assert_eq!(Color::Rgb(95, 184, 143).prefix(), "\x1b[38;2;95;184;143m");
    }

    #[test]
    fn paint_respects_the_enabled_flag() {
        assert_eq!(Color::Default.paint("hello", true), "hello");
        assert_eq!(Color::Rgb(1, 2, 3).paint("hello", false), "hello");
        assert_eq!(
            Color::Rgb(1, 2, 3).paint("hello", true),
            "\x1b[38;2;1;2;3mhello\x1b[0m"
        );
    }
}
