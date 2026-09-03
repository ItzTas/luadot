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

_luadot_names() {
    local names flag=--list
    [[ "$3" == task ]] && flag=--names
    names=$("$1" "$3" "$flag" 2>/dev/null) || return 1
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
        setup | doc | task)
            [[ "$2" == -* ]] && break
            _luadot_names "$1" "$2" "${COMP_WORDS[index]}" && return
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
