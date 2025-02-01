use crate::types;
use crate::types::*;
use std::env;
use std::io;
use std::io::Read;

pub enum InputType {
    Json,
    Kakoune,
    Text,
}

pub enum OutputType {
    Json,
    Kakoune,
    Text,
}

enum Language {
    Clojure,
    Janet,
    Lisp,
    Racket,
    Guile,
    Scheme,
    Hy,
}

pub struct Options {
    matches: getopts::Matches,
}

struct YesNoDefaultOption {
    name: &'static str,
    description: &'static str,
}

impl YesNoDefaultOption {
    fn add(&self, options: &mut getopts::Options) {
        options.optflag("", self.name, self.description);
        options.optflag(
            "",
            &format!("no-{}", self.name),
            &format!("do not {}", self.description),
        );
    }
}

const JANET_LONG_STRINGS_OPTION: YesNoDefaultOption = YesNoDefaultOption {
    name: "janet-long-strings",
    description: "recognize ``` janet-style long strings ```",
};
const LISP_BLOCK_COMMENTS_OPTION: YesNoDefaultOption = YesNoDefaultOption {
    name: "lisp-block-comments",
    description: "recognize #| lisp-style block commments |#.",
};
const LISP_VLINE_SYMBOLS_OPTION: YesNoDefaultOption = YesNoDefaultOption {
    name: "lisp-vline-symbols",
    description: "recognize |lisp-style vline symbol|s.",
};
const GUILE_BLOCK_COMMENTS_OPTION: YesNoDefaultOption = YesNoDefaultOption {
    name: "guile-block-comments",
    description: "recognize #!/guile/block/comments \\n!# )",
};
const SCHEME_SEXP_COMMENTS: YesNoDefaultOption = YesNoDefaultOption {
    name: "scheme-sexp-comments",
    description: "recognize #;( scheme sexp comments )",
};
const HY_BRACKET_STRINGS_OPTION : YesNoDefaultOption = YesNoDefaultOption {
    name: "hy-bracket-strings",
    description: "recognize #[hy-style[ bracket strings ]hy-style]```",
};

fn options() -> getopts::Options {
    let mut options = getopts::Options::new();
    options.optopt("", "comment-char", "(default: ';')", "CC");
    options.optopt("", "string-delimiters", "(default: '\"')", "DELIM");
    options.optflag("h", "help", "show this help message");
    options.optopt(
        "",
        "input-format",
        "'json', 'text' (default: 'text')",
        "FMT",
    );
    GUILE_BLOCK_COMMENTS_OPTION.add(&mut options);
    HY_BRACKET_STRINGS_OPTION.add(&mut options);
    JANET_LONG_STRINGS_OPTION.add(&mut options);
    options.optopt(
        "l",
        "language",
        "'clojure', 'guile', 'hy', 'janet', 'lisp', 'racket', 'scheme' (default: 'clojure')",
        "LANG",
    );
    LISP_BLOCK_COMMENTS_OPTION.add(&mut options);
    LISP_VLINE_SYMBOLS_OPTION.add(&mut options);
    options.optopt(
        "m",
        "mode",
        "parinfer mode (indent, paren, or smart) (default: smart)",
        "MODE",
    );
    options.optopt(
        "",
        "output-format",
        "'json', 'kakoune', 'text' (default: 'text')",
        "FMT",
    );
    SCHEME_SEXP_COMMENTS.add(&mut options);
    options
}

pub fn usage() -> String {
    options().usage("Usage: parinfer-rust [options]")
}

fn parse_language(language: Option<String>) -> Language {
    match language {
        Some(ref s) if s == "clojure" => Language::Clojure,
        Some(ref s) if s == "guile" => Language::Guile,
        Some(ref s) if s == "hy" => Language::Hy,
        Some(ref s) if s == "janet" => Language::Janet,
        Some(ref s) if s == "lisp" => Language::Lisp,
        Some(ref s) if s == "racket" => Language::Racket,
        Some(ref s) if s == "scheme" => Language::Scheme,
        None => Language::Clojure,
        // Unknown language.  LanguageFeatures kind of work for most lisps
        Some(_) => Language::Clojure,
    }
}

struct LanguageFeatures {
    lisp_vline_symbols: bool,
    lisp_block_comments: bool,
    guile_block_comments: bool,
    scheme_sexp_comments: bool,
    janet_long_strings: bool,
    hy_bracket_strings: bool,
}

impl LanguageFeatures {
    fn for_language(language: Language) -> Self {
        match language {
            Language::Clojure => Self {
                lisp_vline_symbols: false,
                lisp_block_comments: false,
                guile_block_comments: false,
                scheme_sexp_comments: false,
                janet_long_strings: false,
                hy_bracket_strings: false,
            },
            Language::Janet => Self {
                lisp_vline_symbols: false,
                lisp_block_comments: false,
                guile_block_comments: false,
                scheme_sexp_comments: false,
                janet_long_strings: true,
                hy_bracket_strings: false,
            },
            Language::Lisp => Self {
                lisp_vline_symbols: true,
                lisp_block_comments: true,
                guile_block_comments: false,
                scheme_sexp_comments: false,
                janet_long_strings: false,
                hy_bracket_strings: false,
            },
            Language::Racket => Self {
                lisp_vline_symbols: true,
                lisp_block_comments: true,
                guile_block_comments: false,
                scheme_sexp_comments: true,
                janet_long_strings: false,
                hy_bracket_strings: false,
            },
            Language::Guile => Self {
                lisp_vline_symbols: true,
                lisp_block_comments: true,
                guile_block_comments: true,
                scheme_sexp_comments: true,
                janet_long_strings: false,
                hy_bracket_strings: false,
            },
            Language::Scheme => Self {
                lisp_vline_symbols: true,
                lisp_block_comments: true,
                guile_block_comments: false,
                scheme_sexp_comments: true,
                janet_long_strings: false,
                hy_bracket_strings: false,
            },
            Language::Hy => Self {
                lisp_vline_symbols: false,
                lisp_block_comments: false,
                guile_block_comments: false,
                scheme_sexp_comments: false,
                janet_long_strings: false,
                hy_bracket_strings: true,
            },
        }
    }
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Options, String> {
        options()
            .parse(args)
            .map(|m| Options { matches: m })
            .map_err(|e| e.to_string())
    }

    pub fn want_help(&self) -> bool {
        self.matches.opt_present("h")
    }

    fn mode(&self) -> &'static str {
        match self.matches.opt_str("m") {
            None => "smart",
            Some(ref s) if s == "i" || s == "indent" => "indent",
            Some(ref s) if s == "p" || s == "paren" => "paren",
            Some(ref s) if s == "s" || s == "smart" => "smart",
            _ => panic!("invalid mode specified for `-m`"),
        }
    }

    fn input_type(&self) -> InputType {
        match self.matches.opt_str("input-format") {
            None => InputType::Text,
            Some(ref s) if s == "text" => InputType::Text,
            Some(ref s) if s == "json" => InputType::Json,
            Some(ref s) if s == "kakoune" => InputType::Kakoune,
            Some(ref s) => panic!("unknown input format `{}`", s),
        }
    }

    pub fn output_type(&self) -> OutputType {
        match self.matches.opt_str("output-format") {
            None => OutputType::Text,
            Some(ref s) if s == "text" => OutputType::Text,
            Some(ref s) if s == "json" => OutputType::Json,
            Some(ref s) if s == "kakoune" => OutputType::Kakoune,
            Some(ref s) => panic!("unknown output fomrat `{}`", s),
        }
    }

    fn comment_char(&self) -> char {
        match self.matches.opt_str("comment-char") {
            None => ';',
            Some(ref s) if s.chars().count() == 1 => s.chars().next().unwrap(),
            Some(ref _s) => panic!("comment character must be a single character"),
        }
    }

    fn string_delimiters(&self) -> Vec<String> {
        let delims = self.matches.opt_strs("string-delimiters");
        if delims.is_empty() {
            vec!["\"".to_string()]
        } else {
            delims
        }
    }

    fn invertible_flag(&self, name: &str) -> Option<bool> {
        if self.matches.opt_present(name) {
            Some(true)
        } else if self.matches.opt_present(&format!("no-{}", name)) {
            Some(false)
        } else {
            None
        }
    }

    fn janet_long_strings(&self) -> Option<bool> {
        self.invertible_flag("janet-long-strings")
    }

    fn lisp_vline_symbols(&self) -> Option<bool> {
        self.invertible_flag("lisp-vline-symbols")
    }

    fn lisp_block_comments(&self) -> Option<bool> {
        self.invertible_flag("lisp-block-comments")
    }

    fn guile_block_comments(&self) -> Option<bool> {
        self.invertible_flag("guile-block-comments")
    }

    fn scheme_sexp_comments(&self) -> Option<bool> {
        self.invertible_flag("scheme-sexp-comments")
    }

    fn hy_bracket_strings(&self) -> Option<bool> {
        self.invertible_flag("hy-bracket-strings")
    }

    pub fn request(&self, input: &mut dyn Read) -> io::Result<Request> {
        match self.input_type() {
            InputType::Text => {
                let LanguageFeatures {
                    lisp_vline_symbols,
                    lisp_block_comments,
                    guile_block_comments,
                    scheme_sexp_comments,
                    janet_long_strings,
                    hy_bracket_strings,
                } = LanguageFeatures::for_language(parse_language(self.matches.opt_str("language")));
                let mut text = String::new();
                input.read_to_string(&mut text)?;
                Ok(Request {
                    mode: String::from(self.mode()),
                    text,
                    options: types::Options {
                        changes: vec![],
                        cursor_x: None,
                        cursor_line: None,
                        prev_text: None,
                        prev_cursor_x: None,
                        prev_cursor_line: None,
                        comment_char: self.comment_char(),
                        string_delimiters: self.string_delimiters(),
                        selection_start_line: None,
                        lisp_vline_symbols: self.lisp_vline_symbols().unwrap_or(lisp_vline_symbols),
                        lisp_block_comments: self
                            .lisp_block_comments()
                            .unwrap_or(lisp_block_comments),
                        guile_block_comments: self
                            .guile_block_comments()
                            .unwrap_or(guile_block_comments),
                        scheme_sexp_comments: self
                            .scheme_sexp_comments()
                            .unwrap_or(scheme_sexp_comments),
                        janet_long_strings: self.janet_long_strings().unwrap_or(janet_long_strings),
                        hy_bracket_strings: self.hy_bracket_strings().unwrap_or(hy_bracket_strings),
                    },
                })
            }
            InputType::Kakoune => {
                let LanguageFeatures {
                    lisp_vline_symbols,
                    lisp_block_comments,
                    guile_block_comments,
                    scheme_sexp_comments,
                    janet_long_strings,
                    hy_bracket_strings,
                } = LanguageFeatures::for_language(parse_language(env::var("kak_opt_filetype").ok()));
                Ok(Request {
                    mode: String::from(self.mode()),
                    text: env::var("kak_selection").unwrap(),
                    options: types::Options {
                        changes: vec![],
                        cursor_x: env::var("kak_opt_parinfer_cursor_char_column")
                            .map(|s| s.parse::<Column>().unwrap() - 1)
                            .ok(),
                        cursor_line: env::var("kak_opt_parinfer_cursor_line")
                            .map(|s| s.parse::<LineNumber>().unwrap() - 1)
                            .ok(),
                        prev_text: env::var("kak_opt_parinfer_previous_text").ok(),
                        prev_cursor_x: env::var("kak_opt_parinfer_previous_cursor_char_column")
                            .map(|s| s.parse::<Column>().unwrap() - 1)
                            .ok(),
                        prev_cursor_line: env::var("kak_opt_parinfer_previous_cursor_line")
                            .map(|s| s.parse::<LineNumber>().unwrap() - 1)
                            .ok(),
                        comment_char: self.comment_char(),
                        string_delimiters: self.string_delimiters(),
                        selection_start_line: None,
                        lisp_vline_symbols,
                        lisp_block_comments,
                        guile_block_comments,
                        scheme_sexp_comments,
                        janet_long_strings,
                        hy_bracket_strings,
                    },
                })
            }
            InputType::Json => {
                let mut text = String::new();
                input.read_to_string(&mut text)?;
                Ok(serde_json::from_str(&text)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn for_args(args: &[&str]) -> Request {
        let input = Vec::new();
        let string_args = args
            .iter()
            .map(|&s| String::from(s))
            .collect::<Vec<String>>();
        let request = Options::parse(&string_args)
            .expect("unable to parse options")
            .request(&mut input.as_slice())
            .expect("unable to make request");
        request
    }

    #[test]
    fn language_option_sets_defaults() {
        let clojure = for_args(&["--language=clojure"]);
        let scheme = for_args(&["--language=scheme"]);
        let janet = for_args(&["--language=janet"]);
        let hy = for_args(&["--language=hy"]);

        assert!(!clojure.options.lisp_vline_symbols);
        assert!(scheme.options.lisp_vline_symbols);
        assert!(!hy.options.lisp_vline_symbols);

        assert!(!clojure.options.janet_long_strings);
        assert!(!scheme.options.janet_long_strings);
        assert!(janet.options.janet_long_strings);
        assert!(!hy.options.janet_long_strings);

        assert!(!clojure.options.hy_bracket_strings);
        assert!(!scheme.options.hy_bracket_strings);
        assert!(!janet.options.hy_bracket_strings);
        assert!(hy.options.hy_bracket_strings);
    }

    #[test]
    fn lisp_vline_symbols() {
        assert!(!for_args(&[]).options.lisp_vline_symbols);
        assert!(
            for_args(&["--language=lisp"]).options.lisp_vline_symbols
        );
        assert!(
            for_args(&["--lisp-vline-symbols"])
                .options
                .lisp_vline_symbols
        );
        assert!(
            !for_args(&["--language=lisp", "--no-lisp-vline-symbols"])
                .options
                .lisp_vline_symbols
        );
    }

    #[test]
    fn lisp_block_comments() {
        assert!(!for_args(&[]).options.lisp_block_comments);
        assert!(
            for_args(&["--language=lisp"]).options.lisp_block_comments
        );
        assert!(
            for_args(&["--lisp-block-comments"])
                .options
                .lisp_block_comments
        );
        assert!(
            !for_args(&["--language=lisp", "--no-lisp-block-comments"])
                .options
                .lisp_block_comments
        );
    }
}
