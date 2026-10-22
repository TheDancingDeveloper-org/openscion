#!/usr/bin/env bash
# Round-trip check: for every replayed commit, point a ref at its changeset,
# `opnscn clone` it, and compare paths, file types, exec bits, symlink targets
# and bytes against `git checkout` of the same commit. Also checks author and
# message on the changeset record. Usage: verify.sh <replay dir>
set -uo pipefail
WORK=$1; SRC=$WORK/src; OUT=$WORK/verify; mkdir -p "$OUT"
ok=0; bad=0
manifest() { (cd "$1" && find . -path ./.git -prune -o -path ./.opnscn -prune -o \( -type f -o -type l \) -printf '%P %y %m\n' |
  awk '{x=($3 ~ /[1357]../)?"x":"-"; print $1, $2, ($2=="l"?"l":x)}' | sort); }
while IFS=$'\t' read -r sha id n; do
  [ "$id" = - ] && continue
  ref=verify-${RUN:-1}/$sha
  curl -sf -X POST -H 'content-type: application/json' \
    -d "{\"expected\":null,\"new\":\"$id\"}" "$OPENSCION_SERVER/v1/refs/$ref" >/dev/null || { echo "ref $sha"; bad=$((bad+1)); continue; }
  rm -rf "$OUT/wc"; opnscn clone --ref "$ref" "$OUT/wc" >/dev/null || { echo "clone $sha"; bad=$((bad+1)); continue; }
  git -C "$SRC" checkout -q -f "$sha"; git -C "$SRC" clean -qfdx
  if ! diff <(manifest "$SRC") <(manifest "$OUT/wc") >"$OUT/$sha.manifest.diff" ||
     ! diff -r --no-dereference -x .git -x .opnscn "$SRC" "$OUT/wc" >"$OUT/$sha.content.diff"; then
    echo "MISMATCH $sha"; bad=$((bad+1)); continue
  fi
  rec=$(curl -sf "$OPENSCION_SERVER/v1/changesets/$id")
  want_author=$(git -C "$SRC" log -1 --format='%an <%ae>' "$sha")
  want_msg=$(git -C "$SRC" log -1 --format=%B "$sha")
  got=$(jq -r '.author' <<<"$rec"); gotmsg=$(jq -r '.message' <<<"$rec")
  if [ "$got" != "$want_author" ] || [ "$gotmsg" != "$want_msg" ]; then echo "META $sha"; bad=$((bad+1)); continue; fi
  ok=$((ok+1)); rm -f "$OUT/$sha".*.diff
done < "$WORK/map.tsv"
echo "verified=$ok mismatched=$bad"
