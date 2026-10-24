use crate::color::Color;
use std::{env, fs, path::PathBuf};

const BUILTIN_CONFIG: &str = include_str!("../.lsr.conf");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub total: Color,
    pub directory: Color,
    pub file: Color,
    pub progress_fill: Color,
    pub progress_empty: Color,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            total: Color::Rgb(95, 184, 143),
            directory: Color::Rgb(124, 196, 255),
            file: Color::Default,
            progress_fill: Color::Ansi(96),
            progress_empty: Color::Ansi(94),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return builtin();
        };

        match fs::read_to_string(&path) {
            Ok(contents) => {
                let (config, warnings) = parse(&contents, &path.display().to_string());
                print_warnings(&warnings);
                config
            }
            Err(error) => {
                eprintln!(
                    "lsr: warning: could not read {}: {}; using built-in defaults",
                    path.display(),
                    error
                );
                builtin()
            }
        }
    }
}

fn builtin() -> Config {
    let (config, warnings) = parse(BUILTIN_CONFIG, "built-in .lsr.conf");
    print_warnings(&warnings);
    config
}

fn config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("LSR_CONFIG") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }

    let local = PathBuf::from(".lsr.conf");
    if local.is_file() {
        Some(local)
    } else {
        None
    }
}

fn parse(contents: &str, source: &str) -> (Config, Vec<String>) {
    let mut config = Config::default();
    let mut warnings = Vec::new();

    for (line_number, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            warnings.push(format!(
                "{}:{}: expected 'key = value', got '{}'",
                source,
                line_number + 1,
                line
            ));
            continue;
        };

        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();

        let Some(slot) = field_mut(&mut config, &key) else {
            warnings.push(format!(
                "{}:{}: unknown setting '{}'",
                source,
                line_number + 1,
                key
            ));
            continue;
        };

        match Color::parse(value) {
            Ok(color) => *slot = color,
            Err(error) => warnings.push(format!(
                "{}:{}: {}; keeping previous value",
                source,
                line_number + 1,
                error
            )),
        }
    }

    (config, warnings)
}

fn field_mut<'a>(config: &'a mut Config, key: &str) -> Option<&'a mut Color> {
    match key {
        "total" => Some(&mut config.total),
        "directory" => Some(&mut config.directory),
        "file" => Some(&mut config.file),
        "progress_fill" => Some(&mut config.progress_fill),
        "progress_empty" => Some(&mut config.progress_empty),
        _ => None,
    }
}

fn print_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("lsr: warning: {}", warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_config_is_valid() {
        let (config, warnings) = parse(BUILTIN_CONFIG, "built-in .lsr.conf");
        assert!(warnings.is_empty(), "warnings: {:?}", warnings);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn parses_overrides_and_keeps_defaults() {
        let (config, warnings) = parse(
            "total = #ffffff\ndirectory = 141\nprogress_fill = red\n",
            "test",
        );

        assert!(warnings.is_empty());
        assert_eq!(config.total, Color::Rgb(255, 255, 255));
        assert_eq!(config.directory, Color::Ansi256(141));
        assert_eq!(config.progress_fill, Color::Ansi(31));
        assert_eq!(config.file, Config::default().file);
        assert_eq!(config.progress_empty, Config::default().progress_empty);
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let (config, warnings) = parse(
            "# a comment\n\n   file = default   \n# another comment\n",
            "test",
        );

        assert!(warnings.is_empty());
        assert_eq!(config, Config::default());
    }

    #[test]
    fn keys_are_case_insensitive_and_trimmed() {
        let (config, warnings) = parse("  DIRECTORY   =   blue  \n", "test");

        assert!(warnings.is_empty());
        assert_eq!(config.directory, Color::Ansi(34));
    }

    #[test]
    fn warns_on_unknown_settings() {
        let (config, warnings) = parse("unknown = red\n", "test");

        assert_eq!(config, Config::default());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("unknown setting 'unknown'"));
    }

    #[test]
    fn warns_on_invalid_colors() {
        let (config, warnings) = parse("directory = not_a_color\n", "test");

        assert_eq!(config, Config::default());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("invalid color"));
    }

    #[test]
    fn warns_on_malformed_lines() {
        let (config, warnings) = parse("directory\n", "test");

        assert_eq!(config, Config::default());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("expected 'key = value'"));
    }
}
