use getopts;
use std::env;
use std::io;
use std::io::Read;
use serde_json;
use types;
use types::*;

pub enum InputType {
    Json,
    Kakoune,
    Text
}

pub enum OutputType {
    Json,
    Kakoune,
    Text
}

enum Language {
    Clojure,
    Janet,
    Lisp,
    Racket,
    Guile,
    Scheme,
}

pub struct Options {
    matches: getopts::Matches
}

struct YesNoDefaultOption {
    name: &'static str,
    description: &'static str
}

impl YesNoDefaultOption {
    fn add(&self, options: &mut getopts::Options) {
        options.optflag("", self.name, self.description);
        options.optflag("", &format!("no-{}", self.name), &format!("do not {}", self.description));
    }
}

const JANET_LONG_STRINGS_OPTION : YesNoDefaultOption = YesNoDefaultOption {
    name: "janet-long-strings",
    description: "recognize ``` janet-style long strings ```",
};
const LISP_BLOCK_COMMENTS_OPTION : YesNoDefaultOption = YesNoDefaultOption {
    name: "lisp-block-comments",
    description:"recognize #| lisp-style block commments |#.",
};
const LISP_VLINE_SYMBOLS_OPTION : YesNoDefaultOption = YesNoDefaultOption {
    name: "lisp-vline-symbols",
    description:"recognize |lisp-style vline symbol|s.",
};
const GUILE_BLOCK_COMMENTS_OPTION : YesNoDefaultOption = YesNoDefaultOption {
    name: "guile-block-comments",
    description: "recognize #!/guile/block/comments \\n!# )",
};
const SCHEME_SEXP_COMMENTS : YesNoDefaultOption = YesNoDefaultOption {
    name: "scheme-sexp-comments",
    description: "recognize #;( scheme sexp comments )",
};

fn options() -> getopts::Options {
    let mut options = getopts::Options::new();
    options.optopt(  ""    , "comment-char"         , "(default: ';')", "CC");
    options.optopt(  ""    , "string-delimiters"    , "(default: '\"')", "DELIM");
    options.optflag("h"    , "help"                 , "show this help message");
    options.optopt( ""     , "input-format"         , "'json', 'text' (default: 'text')", "FMT");
    GUILE_BLOCK_COMMENTS_OPTION.add(&mut options);
    JANET_LONG_STRINGS_OPTION.add(&mut options);
    options.optopt( "l"    , "language"             , "'clojure', 'janet', 'lisp', 'racket', 'guile', 'scheme' (default: 'clojure')", "LANG");
    LISP_BLOCK_COMMENTS_OPTION.add(&mut options);
    LISP_VLINE_SYMBOLS_OPTION.add(&mut options);
    options.optopt( "m"    , "mode"                 , "parinfer mode (indent, paren, or smart) (default: smart)", "MODE");
    options.optopt( ""     , "output-format"        , "'json', 'kakoune', 'text' (default: 'text')", "FMT");
    SCHEME_SEXP_COMMENTS.add(&mut options);
    options
}

pub fn usage() -> String {
    options().usage("Usage: parinfer-rust [options]")
}

struct Defaults {
    lisp_vline_symbols: bool,
    lisp_block_comments: bool,
    guile_block_comments: bool,
    scheme_sexp_comments: bool,
    janet_long_strings: bool
}

fn parse_language(language: Option<String>) -> Language {
    match language {
        Some(ref s) if s == "clojure" => Language::Clojure,
        Some(ref s) if s == "janet"   => Language::Janet,
        Some(ref s) if s == "lisp"    => Language::Lisp,
        Some(ref s) if s == "racket"  => Language::Racket,
        Some(ref s) if s == "guile"   => Language::Guile,
        Some(ref s) if s == "scheme"  => Language::Scheme,
        None                          => Language::Clojure,
        // Unknown language.  Defaults kind of work for most lisps
        Some(_)                       => Language::Clojure,
    }
}

fn language_defaults(language: Language) -> Defaults {
    match language {
        Language::Clojure => Defaults {
            lisp_vline_symbols: false,
            lisp_block_comments: false,
            guile_block_comments: false,
            scheme_sexp_comments: false,
            janet_long_strings: false,
        },
        Language::Janet => Defaults {
            lisp_vline_symbols: false,
            lisp_block_comments: false,
            guile_block_comments: false,
            scheme_sexp_comments: false,
            janet_long_strings: true,
        },
        Language::Lisp => Defaults {
            lisp_vline_symbols: true,
            lisp_block_comments: true,
            guile_block_comments: false,
            scheme_sexp_comments: false,
            janet_long_strings: false
        },
        Language::Racket => Defaults {
            lisp_vline_symbols: true,
            lisp_block_comments: true,
            guile_block_comments: false,
            scheme_sexp_comments: true,
            janet_long_strings: false
        },
        Language::Guile => Defaults {
            lisp_vline_symbols: true,
            lisp_block_comments: true,
            guile_block_comments: true,
            scheme_sexp_comments: true,
            janet_long_strings: false
        },
        Language::Scheme => Defaults {
            lisp_vline_symbols: true,
            lisp_block_comments: true,
            guile_block_comments: false,
            scheme_sexp_comments: true,
            janet_long_strings: false
        },
    }
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Options, String> {
        options()
            .parse(args)
            .map(|m| Options {matches: m})
            .map_err(|e| e.to_string())
    }

    pub fn want_help(&self) -> bool {
        self.matches.opt_present("h")
    }

    fn mode(&self) -> &'static str {
        match self.matches.opt_str("m") {
            None => "smart",
            Some(ref s) if s == "i" || s == "indent" => "indent",
            Some(ref s) if s == "p" || s == "paren"  => "paren",
            Some(ref s) if s == "s" || s == "smart"  => "smart",
            _ => panic!("invalid mode specified for `-m`")
        }
    }

    fn input_type(&self) -> InputType {
        match self.matches.opt_str("input-format") {
            None => InputType::Text,
            Some(ref s) if s == "text" => InputType::Text,
            Some(ref s) if s == "json" => InputType::Json,
            Some(ref s) if s == "kakoune" => InputType::Kakoune,
            Some(ref s) => panic!("unknown input format `{}`", s)
        }
    }

    pub fn output_type(&self) -> OutputType {
        match self.matches.opt_str("output-format") {
            None => OutputType::Text,
            Some(ref s) if s == "text" => OutputType::Text,
            Some(ref s) if s == "json" => OutputType::Json,
            Some(ref s) if s == "kakoune" => OutputType::Kakoune,
            Some(ref s) => panic!("unknown output fomrat `{}`", s)
        }
    }

    fn comment_char(&self) -> char {
        match self.matches.opt_str("comment-char") {
            None => ';',
            Some(ref s) if s.chars().count() == 1 =>  s.chars().next().unwrap(),
            Some(ref _s) => panic!("comment character must be a single character")
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

    pub fn request(&self, input: &mut dyn Read) -> io::Result<Request> {
        match self.input_type() {
            InputType::Text => {
                let Defaults {
                    lisp_vline_symbols,
                    lisp_block_comments,
                    guile_block_comments,
                    scheme_sexp_comments,
                    janet_long_strings
                } = language_defaults(parse_language(self.matches.opt_str("language")));
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
                        force_balance: false,
                        return_parens: false,
                        comment_char: char::from(self.comment_char()),
                        string_delimiters: self.string_delimiters(),
                        partial_result: false,
                        selection_start_line: None,
                        lisp_vline_symbols: self.lisp_vline_symbols().unwrap_or(lisp_vline_symbols),
                        lisp_block_comments: self.lisp_block_comments().unwrap_or(lisp_block_comments),
                        guile_block_comments: self.guile_block_comments().unwrap_or(guile_block_comments),
                        scheme_sexp_comments: self.scheme_sexp_comments().unwrap_or(scheme_sexp_comments),
                        janet_long_strings: self.janet_long_strings().unwrap_or(janet_long_strings),
                    }
                })
            },
            InputType::Kakoune => {
                let Defaults {
                    lisp_vline_symbols,
                    lisp_block_comments,
                    guile_block_comments,
                    scheme_sexp_comments,
                    janet_long_strings
                } = language_defaults(parse_language(env::var("kak_opt_filetype").ok()));
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
                        prev_text: env::var("kak_opt_parinfer_previous_text")
                            .ok(),
                        prev_cursor_x: env::var("kak_opt_parinfer_previous_cursor_char_column")
                            .map(|s| s.parse::<Column>().unwrap() - 1)
                            .ok(),
                        prev_cursor_line: env::var("kak_opt_parinfer_previous_cursor_line")
                            .map(|s| s.parse::<LineNumber>().unwrap() - 1)
                            .ok(),
                        force_balance: false,
                        return_parens: false,
                        comment_char: char::from(self.comment_char()),
                        string_delimiters: self.string_delimiters(),
                        partial_result: false,
                        selection_start_line: None,
                        lisp_vline_symbols,
                        lisp_block_comments,
                        guile_block_comments,
                        scheme_sexp_comments,
                        janet_long_strings,
                    }
                })
            },
            InputType::Json => {
                let mut text = String::new();
                input.read_to_string(&mut text)?;
                Ok(serde_json::from_str(&text)?)
            },
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

        assert_eq!(clojure.options.lisp_vline_symbols, false);
        assert_eq!(scheme.options.lisp_vline_symbols, true);

        assert_eq!(clojure.options.janet_long_strings, false);
        assert_eq!(scheme.options.janet_long_strings, false);
        assert_eq!(janet.options.janet_long_strings, true);
    }

    #[test]
    fn lisp_vline_symbols() {
        assert_eq!(for_args(&[]).options.lisp_vline_symbols, false);
        assert_eq!(for_args(&["--language=lisp"]).options.lisp_vline_symbols, true);
        assert_eq!(for_args(&["--lisp-vline-symbols"]).options.lisp_vline_symbols, true);
        assert_eq!(for_args(&["--language=lisp", "--no-lisp-vline-symbols"]).options.lisp_vline_symbols, false);
    }

    #[test]
    fn lisp_block_comments() {
        assert_eq!(for_args(&[]).options.lisp_block_comments, false);
        assert_eq!(for_args(&["--language=lisp"]).options.lisp_block_comments, true);
        assert_eq!(for_args(&["--lisp-block-comments"]).options.lisp_block_comments, true);
        assert_eq!(for_args(&["--language=lisp", "--no-lisp-block-comments"]).options.lisp_block_comments, false);
    }
}
