declare-option -hidden str parinfer_previous_text
declare-option -hidden str parinfer_previous_cursor_char_column
declare-option -hidden str parinfer_previous_cursor_line
declare-option -hidden str parinfer_previous_timestamp

define-command -params .. \
    -docstring %{parinfer [<switches>]: reformat buffer with parinfer-rust
Modes:
    -indent  Preserve indentation and fix parentheses (default).
    -paren   Preserve parentheses and fix indentation.
    -smart   Try to be smart about what to fix.} \
    parinfer %{
    eval -draft -save-regs '/"|^@' %{
        exec '%'
        eval -draft %sh{
            mode=indent
            while [ $# -ne 0 ]; do
                case "$1" in
                    -smart) mode=smart;;
                    -paren) mode=paren;;
                    -indent) mode=indent;;
                    -*) printf 'fail "unknown switch %s"' "$1"
                esac
                shift
            done
            export mode
            if [ -z "${kak_opt_parinfer_previous_timestamp}" ]; then
                export kak_opt_parinfer_previous_text="${kak_selection}"
                export kak_opt_parinfer_previous_cursor_char_column="${kak_cursor_char_column}"
                export kak_opt_parinfer_previous_cursor_line="${kak_cursor_line}"
            fi
            exec awk '
                # -- JSON encoding of strings --
                BEGIN{
                    for (i = 0; i <= 31; i++) {
                        CODES[sprintf("%c", i)] = sprintf("\\u%04X", i);
                    }
                    CODES["\n"] = "\\n";
                    CODES["\\"] = "\\\\";
                    CODES["\""] = "\\\"";
                }
                function json_encode(data) {
                    result = "";
                    for (i = 1; i <= length(data); i++) {
                        char = substr(data,i,1);
                        if ((char) in CODES) {
                            char = CODES[char];
                        }
                        result = result char;
                    }
                    return result;
                }
                # -- Main program --
                BEGIN{
                    printf \
                        "{\n" \
                        "    \"mode\": \"%s\",\n" \
                        "    \"text\": \"%s\",\n" \
                        "    \"options\": {\n" \
                        "        \"cursorX\": %d,\n" \
                        "        \"cursorLine\": %d,\n" \
                        "        \"prevCursorX\": %d,\n" \
                        "        \"prevCursorLine\": %d,\n" \
                        "        \"prevText\": \"%s\"\n" \
                        "    }\n" \
                        "}\n", \
                        ENVIRON["mode"],
                        json_encode(ENVIRON["kak_selection"]),
                        (ENVIRON["kak_cursor_char_column"] - 1),
                        (ENVIRON["kak_cursor_line"] - 1), \
                        (ENVIRON["kak_opt_parinfer_previous_cursor_char_column"] - 1),
                        (ENVIRON["kak_opt_parinfer_previous_cursor_line"] - 1),
                        json_encode(ENVIRON["kak_opt_parinfer_previous_text"]) \
                        | "parinfer-rust --input-format=json --output-format=kakoune";
                }'
        }
        evaluate-commands %{
            set-option buffer parinfer_previous_text %val{selection}
            set-option buffer parinfer_previous_timestamp %val{timestamp}
            set-option buffer parinfer_previous_cursor_char_column %val{cursor_char_column}
            set-option buffer parinfer_previous_cursor_line %val{cursor_line}
        }
    }
}

hook -group parinfer global WinSetOption filetype=clojure %{
    parinfer -paren
    hook -group parinfer window NormalIdle '' %{
        parinfer -smart
    }
    hook -group parinfer window InsertChar .* %{
        parinfer -smart
    }
    hook -group parinfer window InsertDelete .* %{
        parinfer -smart
    }
}
