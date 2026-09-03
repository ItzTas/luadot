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

function __luadot_names
    set -l tokens (commandline --current-process --cut-at-cursor --tokenize)
    set -l flag --list
    test $argv[1] = task; and set flag --names
    $tokens[1] $argv[1] $flag 2>/dev/null
end

complete -c luadot -n "__fish_seen_subcommand_from git push" -f -a "(__luadot_git_completions)"
complete -c luadot -n "__fish_seen_subcommand_from setup" -f -a "(__luadot_names setup)"
complete -c luadot -n "__fish_seen_subcommand_from task" -f -a "(__luadot_names task)"
complete -c luadot -n "__fish_seen_subcommand_from doc" -f -a "(__luadot_names doc)"
