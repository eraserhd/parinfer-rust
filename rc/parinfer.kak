define-command -docstring "parinfer-enable-window [<mode>]: enable Parinfer for current window.
Modes:
    -indent  Preserve indentation and fix parentheses.
    -paren   Preserve parentheses and fix indentation.
    -smart   Try to be smart about what to fix (default)." \
parinfer-enable-window -params ..1 %{
    require-module parinfer
    try %{
        parinfer -paren
        # if parinfer -paren fails, we don't set parinfer_enable to true
        # thus making sure that hooks do not break our code
        set-option window parinfer_enabled true
    } catch %{
        # set up recovery hooks that will re-enable parinfer when parens are balanced
        hook -group parinfer-try-paren window NormalKey .* parinfer-try-paren
        hook -group parinfer-try-paren window InsertIdle .* parinfer-try-paren
        echo -debug %val{error}
    }
    evaluate-commands %sh{
    printf "%s\n" "hook -group parinfer window NormalKey .* %{ parinfer-try-mode ${1:--smart} }
                   hook -group parinfer window InsertChar (?!\n).* %{ parinfer-try-mode ${1:--smart} }
                   hook -group parinfer window InsertDelete .* %{ parinfer-try-mode ${1:--smart} }"
    }
}

define-command -docstring "parinfer-disable-window: disable Parinfer for current window." \
parinfer-disable-window %{
    remove-hooks window parinfer
    remove-hooks window parinfer-try-paren
    set-option window parinfer_enabled false
}

provide-module parinfer %{

declare-option -docstring "Whether to automatically update the buffer on changes" \
bool parinfer_enabled false

declare-option -docstring "Display parinfer-rust errors in echo area" \
bool parinfer_display_errors true

declare-option -docstring "Currently Parinfer active mode" \
str parinfer_current_mode

declare-option -hidden str parinfer_previous_text
declare-option -hidden str parinfer_previous_cursor_char_column
declare-option -hidden str parinfer_previous_cursor_line
declare-option -hidden str parinfer_previous_timestamp
declare-option -hidden int parinfer_cursor_char_column
declare-option -hidden int parinfer_cursor_line
declare-option -hidden str parinfer_select_switches 'unknown'

define-command -override -docstring "parinfer [<switches>]: reformat buffer with parinfer-rust.
Switches:
    -if-enabled  Check 'parinfer_enabled' option before applying changes.
    -indent      Preserve indentation and fix parentheses (default).
    -paren       Preserve parentheses and fix indentation.
    -smart       Try to be smart about what to fix." \
parinfer -params ..2 %{
    evaluate-commands -draft -save-regs '/"|^@' -no-hooks %{
        set buffer parinfer_cursor_char_column %val{cursor_char_column}
        set buffer parinfer_cursor_line %val{cursor_line}
        execute-keys '\%'
        evaluate-commands -draft -no-hooks %sh{
            mode=indent
            while [ $# -ne 0 ]; do
                case "$1" in
                    -if-enabled) [ "$kak_opt_parinfer_enabled" = "true" ] || exit 0;;
                    -smart)  mode=smart;;
                    -paren)  mode=paren;;
                    -indent) mode=indent;;
                    *)       printf "fail %%{unknown switch '%s'}\n" "$1"
                             exit;;
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
            if [ "${kak_opt_parinfer_select_switches}" = unknown ]; then
                if [ -n "${kak_selections_display_column_desc}" ]; then
                    export kak_opt_parinfer_select_switches='-display-column'
                else
                    export kak_opt_parinfer_select_switches=''
                fi
                printf 'set-option global parinfer_select_switches "%s"\n' "$kak_opt_parinfer_select_switches"
            fi
            # VARIABLES USED:
            # kak_opt_filetype,
            # kak_opt_parinfer_cursor_char_column,
            # kak_opt_parinfer_cursor_line,
            # kak_opt_parinfer_previous_text,
            # kak_opt_parinfer_previous_cursor_char_column,
            # kak_opt_parinfer_previous_cursor_line,
            # kak_opt_parinfer_select_switches,
            # kak_selection
            exec parinfer-rust --mode=$mode --input-format=kakoune --output-format=kakoune
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
        if [ -n "$kak_selections_display_column_desc" ]; then
            set -- $kak_selections_display_column_desc
        else
            set -- $kak_selections_desc
        fi
        case "$1" in
            *,${line}.${column}) exit;;
        esac
        shift
        echo "select ${kak_opt_parinfer_select_switches} ${line}.${column},${line}.${column} $@"
    }
}

# parinfer-try-paren: try to enable paren mode
define-command -hidden parinfer-try-paren %{ try %{
    parinfer -paren
    set-option window parinfer_enabled true
    remove-hooks window parinfer-try-paren
}}

# try apply modification accordingly to mode
# report if failed
define-command -hidden parinfer-try-mode -params 1 %{ evaluate-commands %sh{
    mode="$1"
    [ "$kak_opt_parinfer_enabled" = "true" ] && printf "%s\n" "set-option window parinfer_current_mode '${mode#-}'"
    [ "$kak_opt_parinfer_display_errors" = "true" ] && catch='catch %{ echo -markup "{Error}%val{error}" }' || catch='catch %{ echo -debug %val{error} }'
    printf "%s\n" "try %{ parinfer -if-enabled $mode } $catch"
}}

}

