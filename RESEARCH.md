# Research Log

Purpose: track sources used, what I extracted from them, and any
decisions/assumptions made while implementing.

## Sources

### S1 — Cowgod's CHIP-8 Technical Reference (v1.0)

Link: [Cowgod's CHIP-8 Technical Reference (v1.0)][S1]  
Author: Thomas P. Greene

Notes:

- Widely used community reference.
- Some behaviors differ across CHIP-8 variants, so treat details as version-dependent.

## Extracted notes (by topic)

### Memory layout

- **FACT:** Programs typically start at `0x200`. [S1]
- **FACT:** Font sprites are commonly stored low in memory. [S1]

## Decisions & assumptions

- **DECISION:** TBD
  - Why: TBD
  - Source(s): [S1]

## Open questions / gotchas

- **QUESTION:** TBD

## Change log

- 2026-02-01: Created research log and added S1.

[S1]: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
