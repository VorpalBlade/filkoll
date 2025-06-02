command_not_found_handler() {
  local ret cmd="$1"

  CLICOLOR_FORCE=1 filkoll binary --cmd-not-found-handler --no-fuzzy-if-exact -- "$cmd" 1>&2
  ret="$?"
  if [[ "$ret" -eq 2 ]]; then
    printf 'zsh: command not found: %s\n' "$cmd"
  fi 1>&2

  return 127
}
