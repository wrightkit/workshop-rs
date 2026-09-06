# workshop-rs Current Architecture

This directory is the routing surface for the **current** architecture contracts of `workshop-rs`.

Current architecture and implementation reality are different evidence classes:

- documents here state durable boundaries that current and future implementation must satisfy;
- source, Cargo metadata, tests, fixtures, generated data, and consumer integrations establish current implementation reality;
- [`../adr/`](../adr/) records point-in-time decisions and rationale. An accepted ADR is not by itself proof that the current implementation still matches the decision.

For substantive implementation work, resolve the smallest relevant current contract here before treating an Issue or an older ADR as implementation authority.

## Routing

| Concern | Current contract / authority |
| --- | --- |
| Repository ownership, dependency direction, WIR boundary, semantic-code/data boundary | [`core-boundaries.md`](core-boundaries.md) |
| Declared Workshop language support and current capability state | [`../language-support.md`](../language-support.md) and executable evidence |
| Catalog/localization provenance | [`../provenance.md`](../provenance.md) |
| Hero/gameplay dataset model | [`../gameplay-data.md`](../gameplay-data.md) |
| Gameplay semantic queries | [`../gameplay-query.md`](../gameplay-query.md) |
| Action layout contract | [`../action-layout.md`](../action-layout.md) |
| Architecture decision history | [`../adr/README.md`](../adr/README.md) |

Do not add versions, current feature counts, Issue progress, migration sequencing, or release state to this architecture directory. Those belong to executable metadata, Issues/PRs, releases, or other live evidence.