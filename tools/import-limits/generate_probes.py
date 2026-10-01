#!/usr/bin/env python3
"""Generate the parametric client-import probe fixtures under
`crates/workshop-rs/tests/fixtures/import-limits/`.

Each probe is a complete raw en-US Workshop program that isolates one
structural dimension at a time (per-rule element total, per-action
argument-tree size, argument depth, argument width) while staying below the
global 32768-element budget. The non-generated fixtures in that directory
(`reproducer-bastion-prophet.ws`, `prophet-action-single.ws`) are verbatim
extractions from a real project build; see the fixture README.

The generation parameters are CLI flags so the same script can emit a
bisection ladder when a client test result narrows the boundary. The
committed fixtures are the default-parameter outputs; regenerate with:

    python3 tools/import-limits/generate_probes.py

The script only writes deterministic text; it performs no client claims.
"""

from __future__ import annotations

import argparse
from pathlib import Path

OUT_DIR = Path(__file__).resolve().parents[2] / "crates/workshop-rs/tests/fixtures/import-limits"


def program(variables: str, rules: list[str]) -> str:
    return "variables {\n" + variables + "}\n\n" + "\n".join(rules)


def rule(name: str, actions: list[str], condition: str = "True") -> str:
    body = "\n".join(f"        {a}" for a in actions)
    return (
        f'rule ("{name}") {{\n'
        "    event {\n"
        "        Ongoing - Global;\n"
        "    }\n"
        "    conditions {\n"
        f"        {condition};\n"
        "    }\n"
        "    actions {\n"
        f"{body}\n"
        "    }\n"
        "}\n"
    )


def set_global(value: str) -> str:
    return f"Set Global Variable(probe, {value});"


def array_of(n: int) -> str:
    return "Array(" + ", ".join(str(i) for i in range(n)) + ")"


def and_tree(leaves: int) -> str:
    """A balanced `And` tree: many value nodes at low depth."""
    level = ["True"] * leaves
    while len(level) > 1:
        pairs = [f"And({a}, {b})" for a, b in zip(level[::2], level[1::2])]
        if len(level) % 2:
            pairs.append(level[-1])
        level = pairs
    return level[0]


def if_chain(depth: int) -> str:
    """A right-nested `If-Then-Else` chain: high depth at low node count."""
    value = "0"
    for _ in range(depth):
        value = f"If-Then-Else(Compare(1, <, 2), {value}, 0)"
    return value


def prophet_flat_action() -> str:
    """Same action call and arity as the extracted reproducer, with flat args."""
    return (
        "Create In-World Text(All Players(Team 2), Null, Event Player, 1, "
        "Do Not Clip, Visible To Position String and Color, "
        "Color(White), Default Visibility);"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--actions", type=int, default=160,
                        help="actions per large rule (each holds a 10-number array)")
    parser.add_argument("--rules", type=int, default=4,
                        help="rule count for the split-total control")
    parser.add_argument("--array-len", type=int, default=500,
                        help="elements in the wide single-argument array")
    parser.add_argument("--depth", type=int, default=48,
                        help="nesting depth of the deep-argument probe")
    parser.add_argument("--leaves", type=int, default=256,
                        help="leaves of the balanced And-tree probe")
    args = parser.parse_args()

    acc = "    global:\n        0: probe\n"
    actions = [set_global(array_of(10))] * args.actions

    probes = {
        "control-minimal.ws": program(
            acc, [rule("control: minimal", [set_global("0")])]
        ),
        "rule-elements-large.ws": program(
            acc, [rule("probe: large single rule", actions)]
        ),
        "rule-elements-split.ws": program(
            acc,
            [
                rule(
                    f"probe: split total {i}",
                    actions[i::args.rules],
                )
                for i in range(args.rules)
            ],
        ),
        "arg-array-wide.ws": program(
            acc, [rule("probe: wide flat argument", [set_global(array_of(args.array_len))])]
        ),
        "arg-depth-deep.ws": program(
            acc, [rule("probe: deep argument", [set_global(if_chain(args.depth))])]
        ),
        "arg-nodes-large.ws": program(
            acc, [rule("probe: wide value tree", [set_global(and_tree(args.leaves))])]
        ),
        "prophet-rule-flat.ws": program(
            "    global:\n        0: prophetSlotTextsCreated\n        1: eventName\n"
            "    player:\n        0: eventNextIndex\n        1: eventType\n"
            "        2: eventId\n        3: playerTitleAndColor\n        4: rgb_vect\n",
            [
                rule(
                    "[Event/先知] Create per-slot prophet next-event texts",
                    [prophet_flat_action()] * 8
                    + ["Set Global Variable(prophetSlotTextsCreated, True);"],
                    "Global.prophetSlotTextsCreated == False;\n"
                    "        Count Of(All Players(Team 2)) > Null",
                )
            ],
        ),
    }

    for name, text in probes.items():
        (OUT_DIR / name).write_text(text)
        print(f"wrote {OUT_DIR / name}")


if __name__ == "__main__":
    main()
