/// this file is copied from https://github.com/kardeiz/sanitize-filename/blob/master/src/lib.rs
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref ILLEGAL_RE: Regex = Regex::new(r#"[/\?<>\\:\*\|":]"#).unwrap();
    static ref CONTROL_RE: Regex = Regex::new(r#"[\x00-\x1f\x80-\x9f]"#).unwrap();
    static ref RESERVED_RE: Regex = Regex::new(r#"^\.+$"#).unwrap();
    static ref WINDOWS_RESERVED_RE: Regex =
        RegexBuilder::new(r#"(?i)^(con|prn|aux|nul|com[0-9]|lpt[0-9])(\..*)?$"#)
            .case_insensitive(true)
            .build()
            .unwrap();
    static ref WINDOWS_TRAILING_RE: Regex = Regex::new(r#"^\.+$"#).unwrap();
}

#[derive(Clone)]
pub struct SanitizeOption<'a> {
    pub windows: bool,
    pub replacement: &'a str,
}

impl<'a> Default for SanitizeOption<'a> {
    fn default() -> Self {
        SanitizeOption {
            windows: cfg!(windows),
            replacement: "_",
        }
    }
}

pub fn sanitize<S: AsRef<str>>(name: S) -> String {
    sanitize_with_options(name, SanitizeOption::default())
}

pub fn sanitize_with_options<S: AsRef<str>>(name: S, options: SanitizeOption) -> String {
    let SanitizeOption {
        windows,
        replacement,
    } = options;
    let name = name.as_ref();

    let name = ILLEGAL_RE.replace_all(&name, replacement);
    let name = CONTROL_RE.replace_all(&name, replacement);
    let name = RESERVED_RE.replace(&name, replacement);

    if windows {
        let name = WINDOWS_RESERVED_RE.replace(&name, replacement);
        let name = WINDOWS_TRAILING_RE.replace(&name, replacement);
        name.into()
    } else {
        name.into()
    }
}

#[cfg(test)]
mod tests {
    // From https://github.com/parshap/node-sanitize-filename/blob/master/test.js
    static NAMES: &'static [&'static str] = &[
        "the quick brown fox jumped over the lazy dog",
        "résumé",
        "hello\u{0000}world",
        "hello\nworld",
        "semi;colon.js",
        ";leading-semi.js",
        "slash\\.js",
        "slash/.js",
        "col:on.js",
        "star*.js",
        "question?.js",
        "quote\".js",
        "singlequote'.js",
        "brack<e>ts.js",
        "p|pes.js",
        "plus+.js",
        "'five and six<seven'.js",
        " space at front",
        "space at end ",
        ".period",
        "period.",
        "relative/path/to/some/dir",
        "/abs/path/to/some/dir",
        "~/.\u{0000}notssh/authorized_keys",
        "",
        "h?w",
        "h/w",
        "h*w",
        ".",
        "..",
        "./",
        "../",
        "/..",
        "/../",
        "*.|.",
        "./",
        "./foobar",
        "../foobar",
        "../../foobar",
        "./././foobar",
        "|*.what",
        "LPT9.asdf",
    ];

    static NAMES_CLEANED: &'static [&'static str] = &[
        "the quick brown fox jumped over the lazy dog",
        "résumé",
        "helloworld",
        "helloworld",
        "semi;colon.js",
        ";leading-semi.js",
        "slash.js",
        "slash.js",
        "colon.js",
        "star.js",
        "question.js",
        "quote.js",
        "singlequote'.js",
        "brackets.js",
        "ppes.js",
        "plus+.js",
        "'five and sixseven'.js",
        " space at front",
        "space at end ",
        ".period",
        "period.",
        "relativepathtosomedir",
        "abspathtosomedir",
        "~.notsshauthorized_keys",
        "",
        "hw",
        "hw",
        "hw",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        ".foobar",
        "..foobar",
        "....foobar",
        "...foobar",
        ".what",
        "",
    ];

    #[test]
    fn it_works() {
        let options = super::SanitizeOption {
            windows: true,
            replacement: "",
        };

        for (idx, name) in NAMES.iter().enumerate() {
            assert_eq!(
                super::sanitize_with_options(name, options.clone()),
                NAMES_CLEANED[idx]
            );
        }
    }
}
