command_not_found_handle () {
  local pkgs cmd=$1
  local FUNCNEST=10

  set +o verbose

  mapfile -t pkgs < <(CLICOLOR_FORCE=1 filkoll binary -- "$cmd" 2>/dev/null)

  if (( ${#pkgs[*]} )); then
    printf '%s may be found in the following packages:\n' "$cmd"
    printf '  %s\n' "${pkgs[@]}"
  else
    printf "bash: %s: command not found\n" "$cmd"
  fi >&2

  return 127
}
