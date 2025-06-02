command_not_found_handle () {
  local ret cmd=$1
  local FUNCNEST=10

  set +o verbose

  CLICOLOR_FORCE=1 filkoll binary --cmd-not-found-handler --no-fuzzy-if-exact -- "$cmd" 1>&2
  ret="$?"

  if [[ "$ret" -eq 2 ]]; then
    printf "bash: %s: command not found\n" "$cmd"
  fi 1>&2

  return 127
}
