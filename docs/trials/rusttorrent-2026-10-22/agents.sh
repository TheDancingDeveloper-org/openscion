#!/usr/bin/env bash
# Parallel "agent" clients: each clones its own crate slice, appends a line to
# the crate's src/lib.rs, commits and lands it on main, retrying on contention.
#   git:    sparse partial clone; on a rejected push, `pull --rebase` and push again
#   opnscn: sparse clone; `opnscn commit` is a compare-and-swap with no rebase,
#           so a loser re-clones its slice, re-applies the edit, and commits again
# Usage: agents.sh <trial dir> <tool> <agents> <run> <scenario> [barrier]
#   barrier: clone untimed, then release every agent's commit at once and time
#            only the land phase (the two-changeset race)
# Appends to results.tsv: scenario, tool, run, wall ms, "attempts=<n> landed=<n>".
set -uo pipefail
T=$1; TOOL=$2; N=$3; RUN=$4; SCEN=$5; BARRIER=${6:-}
W=$T/bench; GURL=file://$W/git-server.git; R=$T/results.tsv
CRATES=(librtbit-bencode librtbit-buffers librtbit-clone-to-owned librtbit-core
        librtbit-dht librtbit-lsd librtbit-peer-protocol librtbit-tracker-comms)
D=$W/agents-$TOOL-$SCEN-$RUN; rm -rf "$D"; mkdir -p "$D"
export GIT_AUTHOR_NAME=agent GIT_AUTHOR_EMAIL=agent@example.invalid
export GIT_COMMITTER_NAME=agent GIT_COMMITTER_EMAIL=agent@example.invalid
export OPENSCION_AUTHOR="agent <agent@example.invalid>"
MAX=60

wait_go() { [ -n "$BARRIER" ] || return 0; touch "$D/ready.$1"; while [ ! -e "$D/go" ]; do sleep 0.005; done; }

agent_git() {
  local i=$1 c=${CRATES[$1]} wc=$D/a$1 a=0
  git clone -q --filter=blob:none --sparse "$GURL" "$wc" && git -C "$wc" sparse-checkout set "crates/$c" || return 1
  wait_go "$i"
  echo "// agent $i $SCEN run $RUN" >> "$wc/crates/$c/src/lib.rs"
  git -C "$wc" commit -qam "agent $i: touch $c" || return 1
  while [ $a -lt $MAX ]; do
    a=$((a+1))
    git -C "$wc" push -q origin HEAD:main 2>/dev/null && { echo "$a" > "$D/attempts.$i"; return 0; }
    git -C "$wc" pull -q --rebase origin main 2>/dev/null || return 1
  done
  return 1
}

agent_opnscn() {
  local i=$1 c=${CRATES[$1]} wc=$D/a$1 a=0
  opnscn clone --path "crates/$c" "$wc" >/dev/null || return 1
  wait_go "$i"
  while [ $a -lt $MAX ]; do
    a=$((a+1))
    if [ $a -gt 1 ]; then rm -rf "$wc"; opnscn clone --path "crates/$c" "$wc" >/dev/null || return 1; fi
    echo "// agent $i $SCEN run $RUN" >> "$wc/crates/$c/src/lib.rs"
    opnscn commit --dir "$wc" -m "agent $i: touch $c" >/dev/null 2>&1 && { echo "$a" > "$D/attempts.$i"; return 0; }
  done
  return 1
}

s=0
if [ -z "$BARRIER" ]; then s=$(date +%s%N); fi
pids=()
for i in $(seq 0 $((N-1))); do "agent_$TOOL" "$i" & pids+=($!); done
if [ -n "$BARRIER" ]; then
  while [ "$(ls "$D" | grep -c '^ready\.')" -lt "$N" ]; do sleep 0.01; done
  s=$(date +%s%N); touch "$D/go"
fi
for p in "${pids[@]}"; do wait "$p"; done
e=$(date +%s%N)
landed=$(ls "$D" | grep -c '^attempts\.' || true)
attempts=$(cat "$D"/attempts.* 2>/dev/null | awk '{s+=$1} END{print s+0}')
wall=$(awk -v a="$s" -v b="$e" 'BEGIN{printf "%.1f", (b-a)/1e6}')
printf '%s\t%s\t%s\t%s\t%s\n' "$SCEN" "$TOOL" "$RUN" "$wall" "attempts=$attempts landed=$landed/$N" >> "$R"
rm -rf "$D"
