#!/usr/bin/env bash
# Replay a git repository's first-parent history into OpenScion through the
# `opnscn` client (there is no git importer yet). Writes map.tsv:
#   <git sha> <tab> <opnscn changeset id> <tab> <changes>
# Usage: replay.sh <git dir> <work dir>   (OPENSCION_SERVER selects the server)
set -euo pipefail
GIT=$1; WORK=$2
SRC=$WORK/src; WC=$WORK/wc; MAP=$WORK/map.tsv
rm -rf "$SRC" "$WC"; mkdir -p "$WC"; : > "$MAP"
git clone -q --no-checkout "$GIT" "$SRC"
first=1
git -C "$SRC" rev-list --first-parent --reverse HEAD | while read -r sha; do
  git -C "$SRC" checkout -q -f "$sha"
  git -C "$SRC" clean -qfdx
  rsync -a --modify-window=-1 --delete --exclude=/.git --exclude=/.opnscn "$SRC"/ "$WC"/
  msg=$(git -C "$SRC" log -1 --format=%B "$sha")
  author=$(git -C "$SRC" log -1 --format='%an <%ae>' "$sha")
  if [ $first = 1 ]; then
    out=$(opnscn import "$WC" -m "$msg" --author "$author")
    first=0
    id=${out##* as }; id=${id%% *}; n=${out#imported }; n=${n%% *}
  else
    if ! out=$(opnscn commit --dir "$WC" -m "$msg" --author "$author" 2>&1); then
      printf '%s\t-\t%s\n' "$sha" "ERR:$out" >> "$MAP"; continue
    fi
    id=${out##* as }; n=${out#committed }; n=${n%% *}
  fi
  printf '%s\t%s\t%s\n' "$sha" "$id" "$n" >> "$MAP"
done
