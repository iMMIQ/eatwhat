#!/usr/bin/env python3
"""Offline catalog checks, coverage reports and conservative pack merging.

Never guesses semantic aliases. Conflicts must be resolved by an editor.
"""
import argparse
from collections import Counter
import json
import math
import sys
from pathlib import Path


def norm(value):
    return value.strip().lower()


def validate(pack):
    if set(pack) != {"schema_version", "id", "version", "sources", "dishes"}:
        raise ValueError("unexpected/missing catalog fields")
    if type(pack["schema_version"]) is not int or pack["schema_version"] != 1:
        raise ValueError("unsupported schema_version")
    def label(v):
        return isinstance(v, str) and bool(v.strip())
    if not label(pack["id"]) or not label(pack["version"]):
        raise ValueError("invalid catalog identity")
    source_ids = set()
    for source in pack["sources"]:
        if set(source) != {"id", "title", "license", "reference"} or not all(label(v) for v in source.values()):
            raise ValueError("invalid source")
        if source["id"] in source_ids:
            raise ValueError("duplicate source")
        source_ids.add(source["id"])
    ids, names = set(), {}
    required = {"id", "name", "category", "familiarity", "sources"}
    lists = {"aliases", "regions", "tags", "ingredients", "meals", "modes", "themes"}
    allowed = required | lists | {"ingredients_complete", "minutes_max", "cost_max_fen"}
    for d in pack["dishes"]:
        if not required <= set(d) or set(d) - allowed:
            raise ValueError("unexpected/missing dish fields")
        if not all(label(d[k]) for k in ["id", "name", "category"]):
            raise ValueError("invalid dish identity")
        if d["id"] in ids:
            raise ValueError(f"duplicate id: {d['id']}")
        ids.add(d["id"])
        for k in lists | {"sources"}:
            if not isinstance(d.get(k, []), list) or not all(label(v) for v in d.get(k, [])):
                raise ValueError(f"invalid list: {k}")
        if not d["sources"] or not set(d["sources"]) <= source_ids:
            raise ValueError(f"unknown/missing source: {d['id']}")
        f = d["familiarity"]
        if type(f) not in (float, int) or not math.isfinite(f) or not 0 <= f <= 1:
            raise ValueError("invalid familiarity")
        if type(d.get("ingredients_complete", False)) is not bool:
            raise ValueError("ingredients_complete must be boolean")
        if d.get("ingredients_complete") and not d.get("ingredients"):
            raise ValueError("complete ingredient list cannot be empty")
        for k in ["minutes_max", "cost_max_fen"]:
            v = d.get(k)
            if v is not None and (type(v) is not int or not 0 <= v <= 2**32-1):
                raise ValueError(f"invalid unsigned integer: {k}")
        for name in [d["id"], d["name"], *d.get("aliases", [])]:
            key = norm(name)
            if key in names and names[key] != d["id"]:
                raise ValueError(f"ambiguous alias/name/id: {name}")
            names[key] = d["id"]
    return pack


def load(path):
    return validate(json.loads(Path(path).read_text(encoding="utf-8")))


def merge(packs, pack_id, version):
    sources, dishes = {}, {}
    for pack in packs:
        validate(pack)
        for row, index in [(s, sources) for s in pack["sources"]] + [(d, dishes) for d in pack["dishes"]]:
            key = row["id"]
            if key in index and row != index[key]:
                raise ValueError(f"conflicting record: {key}")
            index[key] = row
    result = dict(schema_version=1, id=pack_id, version=version,
                  sources=sorted(sources.values(), key=lambda s: s["id"]),
                  dishes=sorted(dishes.values(), key=lambda d: d["id"]))
    return validate(result)


def main():
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("check")
    check.add_argument("pack")
    combine = sub.add_parser("merge")
    combine.add_argument("packs", nargs="+")
    combine.add_argument("--id", required=True)
    combine.add_argument("--version", required=True)
    combine.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.command == "check":
        pack = load(args.pack)
        print(json.dumps({"dishes": len(pack["dishes"]),
            "categories": dict(Counter(d["category"] for d in pack["dishes"])),
            "with_regions": sum(bool(d.get("regions")) for d in pack["dishes"]),
            "with_time": sum(d.get("minutes_max") is not None for d in pack["dishes"]),
            "complete_ingredients": sum(d.get("ingredients_complete", False) for d in pack["dishes"])},
            ensure_ascii=False, indent=2))
    else:
        result = merge([load(p) for p in args.packs], args.id, args.version)
        # Exclusive creation avoids accidentally destroying a source catalog.
        with open(args.output, "x", encoding="utf-8") as f:
            json.dump(result, f, ensure_ascii=False, indent=2)
            f.write("\n")


if __name__ == "__main__":
    main()
