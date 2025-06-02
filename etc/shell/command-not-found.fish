function fish_command_not_found
    set cmd $argv

    CLICOLOR_FORCE=1 filkoll binary --cmd-not-found-handler --no-fuzzy-if-exact -- "$cmd" 1>&2
    set ret $status
    if test "$ret" -eq 2
        printf 'fish: Unknown command: %s\n' "$cmd" 1>&2
    end
end
