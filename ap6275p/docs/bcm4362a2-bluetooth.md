# BCM4362A2 Bluetooth PatchRAM reconstruction

StableKite `main` removes `ap6275p/BCM4362A2.hcd` and carries source in `../bcm4362a2-patch/`.

## Stage-18 correction

Stages 6–17 used legacy HCD SHA-256 `3e4a1eddaf80f3e45f99e9c77b3cd84c85f605540da5f4f92300b80bca6d67ec` (73136 bytes). The current Orange Pi HCD is SHA-256 `f7adf14413063f14b0204684fb67ddcd2ae6bca3120343cb9a5cab86a1a545c3` (91900 bytes). They are distinct PatchRAM programs.

The HCD WRITE_RAM records are not ordered by destination address. Earlier Stage-17 code incorrectly tested contiguity in stream order. Stage 18 fixes this by sorting recovered WRITE_RAM extents before checking gap/overlap-free coverage.

| Profile | WRITE_RAM | Payload bytes | Code | Data | Config | Launch |
| --- | ---: | ---: | --- | --- | --- | --- |
| Legacy | 462 | 69895 | 414 writes, `[0x160400,0x16e7a8)` | 8 writes, `[0x221d9c,0x2224cc)` | 40 writes, `[0x240000,0x24262f)` | one sentinel |
| Current Orange Pi | 575 | 87868 | 518 writes, `[0x160800,0x17298c)` | 9 writes, `[0x221d9c,0x222578)` | 48 writes, `[0x240000,0x242dd4)` | one sentinel |

Stage 18 freezes the current structural profile but does not assert that legacy patched-function addresses or meanings survived unchanged. Stage 19A permits only unique complete-body byte-identity anchors; current-image call-graph/decompiler claims remain pending verified IDA evidence.

## Stage-19 exact relocation anchors

Stage-18 batch IDA evidence is invalid: its log contains only the unaccepted-license diagnostic. Stage 19 therefore begins with exact full-function byte identity against the current reconstructed PatchRAM code region.

Among 266 legacy IDA functions of at least 8 bytes, 44 have one exact full-byte match in the current executable region, 9 have multiple exact matches, and 213 have no exact match. The following named/structural anchors are unique:

| Legacy | Current | Bytes | Meaning retained from exact body |
| --- | --- | ---: | --- |
| `0x160800` | `0x160800` | 368 | unnamed structural patch anchor |
| `bt_index_stride20` `0x162f4c` | `0x163668` | 12 | returns 20 × an 8-bit global index |
| `bt_set_global_60` `0x169104` | `0x16b7a4` | 10 | stores constant 60 to a PC-relative current-image global and returns zero |
| `bt_u32_gt_2` `0x16cc80` | `0x170178` | 10 | unsigned argument > 2 helper |
| `bt_find_first_slot_state1` `0x16e010` | `0x1720c0` | 24 | loop/selection helper; callees still require current-image traversal |

The legacy `bt_object_state_is_2` at `0x1604d4` lies before the current PatchRAM code start (`0x160800`) and is not claimed to exist in the current HCD. Other legacy names are likewise not transferred unless current evidence supports them.

## Stage-20 current control-flow anchors

Stage 20 verifies direct Thumb branch instructions on the **current** PatchRAM code. It does not infer targets from a global address delta.

The exact `bt_find_first_slot_state1` body at `0x1720c0` contains a verified `BL` at `0x1720c6`; decoding the current instruction gives target `0x172044`. This proves the current helper destination for that call site, but does not claim that the helper body is unchanged from legacy `sub_16DF94`.

The 368-byte exact structural anchor at `0x160800` remains at the same current address. Stage 20 verifies 19 direct `BL` instructions inside it. They reach eight distinct ROM-side destinations:

`0x202c0`, `0x202e8`, `0x21f0c`, `0x222a4`, `0x224a8`, `0x43d2c`, `0x443fc`, and `0x45dd4`.

These addresses are current direct-call boundaries proven from current instruction bytes. No semantic label is assigned to the ROM routines until separately recovered.

For repeated exact bodies, all 44 globally unique >=8-byte BT matches form a monotonic spine. Three of the nine globally repeated exact bodies have exactly one occurrence between their nearest monotonic neighbors and are recorded as context-disambiguated structural relocations.

## Stage-21 helper-body verification

Following the Stage-20 `bt_find_first_slot_state1` call target to current `0x172044`, Stage 21 verifies all 62 helper code bytes against legacy `sub_16DF94` after canonicalizing only the three direct `BL` immediates. The normalized hashes are identical and all three calls still reach the same ROM addresses: `0x3d24`, `0xf8cac`, and `0x94c0`.

The helper's literal pool is checked separately. The context/guard word remains `0x200890`; the 7-byte per-slot record base moves from legacy `0x222e42` to current `0x222fd6`. The instruction sequence still computes `7 * index`, so the eight indices scanned by `bt_find_first_slot_state1` correspond to current record starts `0x222fd6 .. 0x223007` in steps of seven bytes.

This establishes the current helper's record-addressing/control-flow shape. The exact vendor identities of ROM routines `0x3d24`, `0xf8cac`, and `0x94c0` remain unresolved; Stage 21 therefore does not rename those ROM boundaries.

## Stage-22 slot-helper semantics

Stage 21 proves that current helper `0x172044` has the same complete instruction skeleton as legacy `sub_16DF94`, while independently verifying the relocated record base `0x222fd6`.

The helper's three direct ROM calls remain at the same absolute addresses and call-site offsets:

- `0x3d24` — memset-like: zeroes a 7-byte local comparison buffer;
- `0xf8cac` — memcmp-like: compares `record_base + 7 * index` with that zero buffer;
- `0x94c0` — stack-guard terminal sink, reached only if the canary changes.

Therefore the helper returns `1` exactly when the selected 7-byte slot record is all zero (apart from the terminal stack-canary failure path). Combined with current `bt_find_first_slot_state1` at `0x1720c0`, the higher-level behavior is now reconstructed as: **return the first empty slot in indices 0..7, or 8 if none is empty**.

The Rust source adds `bt_slot_record_is_empty` and `bt_find_first_empty_slot` as allocation-free libre equivalents.

## Stage-23 slot-table lifecycle

Stage 23 expands the Stage-22 empty-slot semantics into the current table lifecycle. Five legacy functions have exactly one relocation-normalized complete-body match in the current PatchRAM code, with current literal words and direct branch targets independently re-read:

| Role | Legacy | Current | Current evidence |
| --- | --- | --- | --- |
| find matching key+payload | `0x16dfdc` | `0x17208c` | payload base `0x222fd7`, `memcmp-like 0xf8cac` |
| clear all eight records | `0x16e2b0` | `0x1723c0` | table base `0x222fd6`, `memset-like 0x3d24` |
| insert if absent | `0x16e2c0` | `0x1723d0` | lookup `0x17208c`, empty-slot scan `0x1720c0`, `memcpy-like 0x3db4` |
| remove matching record | `0x16e348` | `0x172458` | lookup `0x17208c`, `memset-like 0x3d24` |
| reset-table tail | `0x16e370` | `0x172480` | ends by clearing 56-byte table; other reset side effects remain outside the libre model |

The recovered record is exactly seven bytes: one tag byte followed by six payload bytes. The reconstructed source preserves firmware return behavior: insertion returns `0` both for an existing duplicate and a successful new insert, returns `17` when all eight slots are occupied; removal returns `1` when a record was cleared and `0` when absent.

A deliberate edge case is preserved: key `0` with a six-byte zero payload matches an already-empty all-zero record.
