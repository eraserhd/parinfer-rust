
define-command -docstring "format bufffer with parinfer-rust" parinfer %{
    eval -draft -save-regs '/"|^@' %{
        exec -save-regs '' 'Z%'
        eval -draft %sh{
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
                        "    \"mode\": \"indent\",\n" \
                        "    \"text\": \"%s\",\n" \
                        "    \"options\": {\n" \
                        "        \"cursorX\": %d,\n" \
                        "        \"cursorLine\": %d\n" \
                        "    }\n" \
                        "}\n", \
                        json_encode(ENVIRON["kak_selection"]),
                        (ENVIRON["kak_cursor_char_column"] - 1),
                        (ENVIRON["kak_cursor_line"] - 1) \
                        | "parinfer-rust --input-format=json --output-format=kakoune";
                }'
        }
        exec -save-regs '' z
    }
}
