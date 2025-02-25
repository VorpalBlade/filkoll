command_not_found_handler() {
  local pkgs cmd="$1"

  pkgs=(${(f)"$(CLICOLOR_FORCE=1 filkoll binary --no-fuzzy-if-exact -- "$cmd" 2>/dev/null)"})
  if [[ -n "$pkgs" ]]; then
    printf '%s may be found in the following packages:\n' "$cmd"
    printf '  %s\n' $pkgs[@]
  else
    printf 'zsh: command not found: %s\n' "$cmd"
  fi 1>&2

  return 127
}
