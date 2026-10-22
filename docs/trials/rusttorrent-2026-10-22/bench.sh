#!/usr/bin/env bash
# opnscn vs git on a replayed copy of rustTorrent. Appends
#   scenario <tab> tool <tab> run <tab> ms [<tab> note]
# to results.tsv. Usage: bench.sh <trial dir> <runs>
# Needs: OPENSCION_SERVER pointing at a server holding the replay on `main`,
# <trial dir>/rt.git (bare mirror), opnscn and history.py.
set -euo pipefail
T=$1; RUNS=${2:-5}; W=$T/bench; R=$T/results.tsv
GITSRV=$W/git-server.git; GURL=file://$GITSRV
HOT=crates/librtbit/src/session/mod.rs
SLICE=crates/librtbit   # one crate, web UI included (crates/librtbit/webui)
AGENT_CRATES=(librtbit-bencode librtbit-buffers librtbit-clone-to-owned librtbit-core
              librtbit-dht librtbit-lsd librtbit-peer-protocol librtbit-tracker-comms)
rm -rf "$W"; mkdir -p "$W"
cp -a "$T/rt.git" "$GITSRV"
git -C "$GITSRV" config uploadpack.allowFilter true
git -C "$GITSRV" config uploadpack.allowAnySHA1InWant true
git -C "$GITSRV" config --unset remote.origin.mirror || true
export GIT_AUTHOR_NAME=bench GIT_AUTHOR_EMAIL=bench@example.invalid
export GIT_COMMITTER_NAME=bench GIT_COMMITTER_EMAIL=bench@example.invalid
export OPENSCION_AUTHOR="bench <bench@example.invalid>"

now() { date +%s%N; }
ms() { awk -v a="$1" -v b="$2" 'BEGIN{printf "%.1f", (b-a)/1e6}'; }
rec() { printf '%s\t%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" "${5:-}" >> "$R"; }
# timed <scenario> <tool> <run> <cmd...>
timed() { local s e; s=$(now); "${@:4}" >/dev/null 2>&1; e=$(now); rec "$1" "$2" "$3" "$(ms "$s" "$e")"; }

for r in $(seq 1 "$RUNS"); do
  d=$W/r$r; mkdir -p "$d"
  # --- full clone
  timed full-clone git-full-history "$r" git clone -q --no-local "$GURL" "$d/g-full"
  timed full-clone git-depth1 "$r" git clone -q --depth 1 "$GURL" "$d/g-d1"
  timed full-clone opnscn "$r" opnscn clone "$d/o-full"
  # --- sparse clone of one crate (+ its web UI)
  timed sparse-clone git-partial "$r" sh -c "git clone -q --filter=blob:none --sparse '$GURL' '$d/g-sp' && git -C '$d/g-sp' sparse-checkout set $SLICE"
  timed sparse-clone git-partial-depth1 "$r" sh -c "git clone -q --depth 1 --filter=blob:none --sparse '$GURL' '$d/g-sp1' && git -C '$d/g-sp1' sparse-checkout set $SLICE"
  timed sparse-clone opnscn "$r" opnscn clone --path "$SLICE" "$d/o-sp"
  # --- commit one changed file (git commit is local; push is what publishes it)
  echo "// bench $r" >> "$d/g-full/$HOT"
  timed commit-one-file git-commit "$r" git -C "$d/g-full" commit -qam "bench $r"
  timed commit-one-file git-push "$r" git -C "$d/g-full" push -q origin HEAD:main
  echo "// bench $r" >> "$d/o-full/$HOT"
  timed commit-one-file opnscn "$r" opnscn commit --dir "$d/o-full" -m "bench $r"
  # --- history queries on a hot file (git: local full clone; opnscn: client-side walk of the API)
  timed log-path git "$r" git -C "$d/g-full" log --first-parent --oneline -- "$HOT"
  timed log-path opnscn-api-walk "$r" python3 -I "$T/history.py" log "$OPENSCION_SERVER" main "$HOT"
  timed diff-20 git "$r" git -C "$d/g-full" diff --name-only HEAD~20 HEAD
  timed diff-20 opnscn-api-walk "$r" python3 -I "$T/history.py" diff "$OPENSCION_SERVER" main 20
  timed blame git "$r" git -C "$d/g-full" blame -s "$HOT"
  rm -rf "$d"
done
echo "single-op scenarios done"
