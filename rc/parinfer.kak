declare-option -docstring %{Whether to automatically update the buffer on changes} bool parinfer_enabled no

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
    eval -draft -save-regs '/"|^@' -no-hooks %{
        exec '\%'
        eval -draft -no-hooks %sh{
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
            elif [ "$mode" = smart ] &&
                 [ "${kak_opt_parinfer_previous_timestamp}" = "$kak_timestamp" ]; then
                exit 0
            fi
            exec awk '
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
                        (ENVIRON["kak_cursor_line"] - 1),
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
    evaluate-commands %sh{
        if [ $kak_opt_parinfer_enabled = true ]; then
            parinfer -paren
        fi
    }
    hook -group parinfer window NormalIdle '' %{
        evaluate-commands %sh{
            if [ $kak_opt_parinfer_enabled = true ]; then
                parinfer -smart
            fi
        }
    }
    hook -group parinfer window InsertChar .* %{
        evaluate-commands %sh{
            if [ $kak_opt_parinfer_enabled = true ]; then
                parinfer -smart
            fi
        }
    }
    hook -group parinfer window InsertDelete .* %{
        evaluate-commands %sh{
            if [ $kak_opt_parinfer_enabled = true ]; then
                parinfer -smart
            fi
        }
    }
}
