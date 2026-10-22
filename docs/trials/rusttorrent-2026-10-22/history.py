#!/usr/bin/env python3
"""Client-side stand-ins for the Log and Diff operations OpenScion does not
serve yet: walk the first-parent chain over GET /v1/changesets/{id} on one
keep-alive connection.

  history.py log  <server> <ref> <path>   changesets on the first-parent line that touched path
  history.py diff <server> <ref> <n>      paths changed between ref~n and ref
"""
import http.client
import json
import sys
from urllib.parse import urlparse


def main() -> None:
    mode, server, ref, arg = sys.argv[1:5]
    url = urlparse(server)
    conn = http.client.HTTPConnection(url.hostname, url.port)

    def get(path: str) -> dict:
        conn.request("GET", path)
        resp = conn.getresponse()
        body = resp.read()
        if resp.status != 200:
            raise SystemExit(f"{path}: {resp.status} {body[:200]!r}")
        return json.loads(body)

    cur = get(f"/v1/refs/{ref}")["target"]
    if mode == "log":
        hits = 0
        while cur:
            rec = get(f"/v1/changesets/{cur}")
            if any(c["path"] == arg for c in rec["changes"]):
                hits += 1
                print(rec["id"][:12], rec["message"].splitlines()[0])
            cur = rec["parents"][0] if rec["parents"] else None
        print(f"{hits} changesets", file=sys.stderr)
    elif mode == "diff":
        paths: set[str] = set()
        for _ in range(int(arg)):
            rec = get(f"/v1/changesets/{cur}")
            paths.update(c["path"] for c in rec["changes"])
            cur = rec["parents"][0]
        print("\n".join(sorted(paths)))
        print(f"{len(paths)} paths", file=sys.stderr)


if __name__ == "__main__":
    main()
