use crate::files::FileStatus;
use crate::output::Tone;

pub const DEFAULT_FILTER: &str = "warn";

pub const VERBOSE_FILTER: &str = "luadot=debug";

pub const TRACE_FILTER: &str = "luadot=trace";

pub const UNSET: &str = "(none)";

pub const UNDECLARED: &str = "(not declared)";

pub const DEFAULT_SHELL: &str = "/bin/sh";

pub const PREVIEW_LIMIT: usize = 10;

pub const YES_FLAGS: &str = "-y or --yes";

pub const TEMPLATE_SKELETON: &str = "return \"\"\n";

pub const DIFF_PROGRAM: &str = "git";

pub const DIFF_ARGUMENTS: [&str; 1] = ["diff"];

pub const DIFF_CUSTOM: &str = "ld.on.diff";

pub const STATUS_CUSTOM: &str = "ld.on.status";

pub const CUSTOM_ENTRY: &str = "entry";

pub const CUSTOM_RENDER: &str = "render";

pub const CUSTOM_SUMMARY: &str = "summary";

pub const MANAGED_FILES: &str = "managed";

pub const GENERATED_FILES: &str = "generated";

pub const ZSH_DISPATCH: &str = "if [ \"$funcstack[1]\"";

pub const BASH_GIT_COMPLETION: &str = r##"
_luadot_git_load() {
    if declare -F _comp_load >/dev/null 2>&1; then
        _comp_load git >/dev/null 2>&1 && return 0
    fi
    if declare -F _completion_loader >/dev/null 2>&1; then
        _completion_loader git >/dev/null 2>&1 && return 0
    fi
    local script
    for script in \
        "${BASH_COMPLETION_USER_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion}/completions/git" \
        /usr/share/bash-completion/completions/git \
        /usr/local/share/bash-completion/completions/git \
        /opt/homebrew/etc/bash_completion.d/git \
        /usr/local/etc/bash_completion.d/git \
        /etc/bash_completion.d/git; do
        [[ -r "$script" ]] || continue
        source "$script" >/dev/null 2>&1 && return 0
    done
    return 1
}

_luadot_git() {
    local command=$1 index=$2
    shift 2

    local spec
    spec=$(complete -p git 2>/dev/null)
    if [[ -z "$spec" ]]; then
        _luadot_git_load
        spec=$(complete -p git 2>/dev/null)
    fi
    [[ "$spec" =~ -F[[:space:]]+([^[:space:]]+) ]] || return 1

    local completer="${BASH_REMATCH[1]}"
    declare -F "$completer" >/dev/null 2>&1 || return 1

    local repo dir
    repo=$("$command" config repo 2>/dev/null)
    [[ -n "$repo" ]] && dir=$(git -C "$repo" rev-parse --absolute-git-dir 2>/dev/null)
    if [[ -n "$dir" ]]; then
        local -x GIT_DIR="$dir" GIT_WORK_TREE="$repo"
    fi

    local -a forwarded=(git "$@" "${COMP_WORDS[@]:index+1}")
    local cword=$((COMP_CWORD - index + $#))
    local head="${forwarded[*]:0:cword}"
    local line="${forwarded[*]}"
    ((cword >= ${#forwarded[@]})) && line="$line "

    local -a COMP_WORDS=("${forwarded[@]}")
    local COMP_CWORD=$cword
    local COMP_LINE=$line
    local COMP_POINT=$((${#head} + 1 + ${#forwarded[cword]}))

    compopt -o bashdefault -o default -o nospace 2>/dev/null
    "$completer" git "${forwarded[cword]}" "${forwarded[cword - 1]}"
}

_luadot_setup() {
    local names
    names=$("$1" setup --list 2>/dev/null) || return 1
    [[ -n "$names" ]] || return 1
    mapfile -t COMPREPLY < <(compgen -W "$names" -- "$2")
}

_luadot_complete() {
    local index
    for ((index = 1; index < COMP_CWORD; index++)); do
        case "${COMP_WORDS[index]}" in
        -*) ;;
        git)
            _luadot_git "$1" "$index"
            return
            ;;
        push)
            _luadot_git "$1" "$index" push
            return
            ;;
        setup)
            [[ "$2" == -* ]] && break
            _luadot_setup "$1" "$2" && return
            break
            ;;
        *) break ;;
        esac
    done
    _luadot "$@"
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _luadot_complete -o nosort -o bashdefault -o default luadot
else
    complete -F _luadot_complete -o bashdefault -o default luadot
fi
"##;

pub const ZSH_GIT_COMPLETION: &str = r##"_luadot_git() {
    local index=$1
    shift

    local repo dir
    repo=$($words[1] config repo 2>/dev/null)
    [[ -n $repo ]] && dir=$(git -C $repo rev-parse --absolute-git-dir 2>/dev/null)
    if [[ -n $dir ]]; then
        local -x GIT_DIR=$dir GIT_WORK_TREE=$repo
    fi

    words=(git "$@" "${words[@]:$index}")
    (( CURRENT -= index - 1 - $# ))
    _normal
}

_luadot_setup() {
    local -a names
    names=(${(f)"$($words[1] setup --list 2>/dev/null)"})
    (( $#names )) || return 1
    _describe -t setups setup names
}

functions[_luadot_clap]=$functions[_luadot]

_luadot() {
    local index
    for (( index = 2; index < CURRENT; index++ )); do
        case $words[index] in
            -*) ;;
            git)
                _luadot_git $index
                return
                ;;
            push)
                _luadot_git $index push
                return
                ;;
            setup)
                [[ $words[CURRENT] == -* ]] && break
                _luadot_setup && return
                break
                ;;
            *) break ;;
        esac
    done
    _luadot_clap "$@"
}

if [ "$funcstack[1]" = "_luadot" ]; then
    _luadot "$@"
else
    compdef _luadot luadot
fi
"##;

pub const FISH_GIT_COMPLETION: &str = r##"
function __luadot_git_completions
    set -l tokens (commandline --current-process --cut-at-cursor --tokenize)
    set -l total (count $tokens)
    set -l index 2
    while test $index -le $total; and string match -qr -- '^-' $tokens[$index]
        set index (math $index + 1)
    end
    test $index -le $total; or return
    set -l repo ($tokens[1] config repo 2>/dev/null)
    set -l dir
    test -n "$repo"; and set dir (git -C $repo rev-parse --absolute-git-dir 2>/dev/null)
    test -n "$dir"; and set -lx GIT_DIR $dir
    test -n "$dir"; and set -lx GIT_WORK_TREE $repo
    set -l forwarded git
    test $tokens[$index] = push; and set -a forwarded push
    test (math $index + 1) -le $total; and set -a forwarded $tokens[(math $index + 1)..-1]
    set -l line (string escape -- $forwarded | string join ' ')
    set -l token (commandline --current-token --cut-at-cursor)
    complete --do-complete "$line $token"
end

function __luadot_setup_names
    set -l tokens (commandline --current-process --cut-at-cursor --tokenize)
    $tokens[1] setup --list 2>/dev/null
end

complete -c luadot -n "__fish_seen_subcommand_from git push" -f -a "(__luadot_git_completions)"
complete -c luadot -n "__fish_seen_subcommand_from setup" -f -a "(__luadot_setup_names)"
"##;

pub const STATUS_LABELS: [(FileStatus, &str, Tone); 5] = [
    (FileStatus::Synced, "synced", Tone::Good),
    (FileStatus::Missing, "missing", Tone::Warning),
    (FileStatus::Unlinked, "unlinked", Tone::Warning),
    (FileStatus::Differs, "differs", Tone::Bad),
    (FileStatus::Unreadable, "unreadable", Tone::Warning),
];

pub const STATUS_HEAD: &str = "On repository";

pub const STATUS_GENERATED_HEAD: &str = "Generated from templates";

pub const STATUS_CLEAN: &str = "nothing to apply, every managed file is synced";

pub const STATUS_GENERATED_CLEAN: &str = "nothing to apply, every generated file is synced";

pub const STATUS_SECTIONS: [(FileStatus, &str, &str); 4] = [
    (
        FileStatus::Missing,
        "Files not on the system:",
        "(use \"luadot apply <path>...\" to write them)",
    ),
    (
        FileStatus::Unlinked,
        "Files not linked:",
        "(use \"luadot apply <path>...\" to link them)",
    ),
    (
        FileStatus::Differs,
        "Files that differ:",
        "(use \"luadot diff <path>...\" to see what changed)",
    ),
    (
        FileStatus::Unreadable,
        "Files luadot may not read:",
        "(use \"luadot apply <path>...\" with the privilege to read them)",
    ),
];
