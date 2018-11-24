declare-option -docstring %{Whether to automatically update the buffer on changes} bool parinfer_enabled yes

declare-option -hidden str parinfer_previous_text
declare-option -hidden str parinfer_previous_cursor_char_column
declare-option -hidden str parinfer_previous_cursor_line
declare-option -hidden str parinfer_previous_timestamp
declare-option -hidden int parinfer_cursor_char_column
declare-option -hidden int parinfer_cursor_line

define-command -params .. \
    -docstring %{parinfer [<switches>]: reformat buffer with parinfer-rust
Modes:
    -indent  Preserve indentation and fix parentheses (default).
    -paren   Preserve parentheses and fix indentation.
    -smart   Try to be smart about what to fix.} \
    parinfer %{
    eval -draft -save-regs '/"|^@' -no-hooks %{
        set buffer parinfer_cursor_char_column %val{cursor_char_column}
        set buffer parinfer_cursor_line %val{cursor_line}
        exec '\%'
        eval -draft -no-hooks %sh{
            mode=indent
            while [ $# -ne 0 ]; do
                case "$1" in
                    -if-enabled) [ $kak_opt_parinfer_enabled = true ] || exit 0;;
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
                export kak_opt_parinfer_previous_cursor_char_column="${kak_opt_parinfer_cursor_char_column}"
                export kak_opt_parinfer_previous_cursor_line="${kak_opt_parinfer_cursor_line}"
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
                        (ENVIRON["kak_opt_parinfer_cursor_char_column"] - 1),
                        (ENVIRON["kak_opt_parinfer_cursor_line"] - 1),
                        (ENVIRON["kak_opt_parinfer_previous_cursor_char_column"] - 1),
                        (ENVIRON["kak_opt_parinfer_previous_cursor_line"] - 1),
                        json_encode(ENVIRON["kak_opt_parinfer_previous_text"]) \
                        | "parinfer-rust --input-format=json --output-format=kakoune";
                }' </dev/null
        }
        evaluate-commands %{
            set-option buffer parinfer_previous_text %val{selection}
            set-option buffer parinfer_previous_timestamp %val{timestamp}
            set-option buffer parinfer_previous_cursor_char_column %val{cursor_char_column}
            set-option buffer parinfer_previous_cursor_line %val{cursor_line}
        }
    }
    evaluate-commands %sh{
        line=$kak_opt_parinfer_cursor_line
        column=$kak_opt_parinfer_cursor_char_column
        set -- $kak_selections_desc
        case "$1" in
            *,${line}.${column}) exit;;
        esac
        shift
        echo "select ${line}.${column},${line}.${column} $@"
    }
}

hook -group parinfer global WinSetOption filetype=clojure %{
    parinfer -if-enabled -paren
    hook -group parinfer window NormalKey .* %{ parinfer -if-enabled -smart }
    hook -group parinfer window InsertChar .* %{ parinfer -if-enabled -smart }
    hook -group parinfer window InsertDelete .* %{ parinfer -if-enabled -smart }
}
hook -group parinfer global WinSetOption parinfer_enabled=true %{
    parinfer -paren
}
