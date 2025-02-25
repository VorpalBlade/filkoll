function fish_command_not_found
    set cmd $argv

    if set pkgs (CLICOLOR_FORCE=1 filkoll binary -- "$cmd" 2>/dev/null)
        printf '%s may be found in the following packages:\n' "$cmd"
        for pkg in $pkgs
            printf '  %s\n' "$pkg"
        end
    end
end
