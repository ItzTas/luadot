_luadot_git() {
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

_luadot_names() {
    local -a names
    local flag=--list
    [[ $1 == task ]] && flag=--names
    names=(${(f)"$($words[1] $1 $flag 2>/dev/null)"})
    (( $#names )) || return 1
    _describe -t $1 $1 names
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
            setup | doc | task)
                [[ $words[CURRENT] == -* ]] && break
                _luadot_names $words[index] && return
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
