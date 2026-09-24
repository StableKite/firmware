# Provenance and Stage-18 reference correction

The StableKite fork is based on `orangepi-xunlong/firmware`. `upstream-master` remains the unmodified tracking branch; proprietary target blobs are absent from StableKite `main`.

## Current Orange Pi references

| Component | Upstream path | Size | SHA-256 | Upstream commit touching file |
| --- | --- | ---: | --- | --- |
| BCM43752A2 Wi-Fi | `ap6275p/fw_bcm43752a2_pcie_ag.bin` | 936074 | `6a2dbe01e72221defba91a52e158768d973a3c85ca2d881c924379e35ad36b23` | `44e2dfea2e1e6b1af6900b33246b424760b756c3` |
| BCM4362A2 Bluetooth | `ap6275p/BCM4362A2.hcd` | 91900 | `f7adf14413063f14b0204684fb67ddcd2ae6bca3120343cb9a5cab86a1a545c3` | `75ea6fc5f3c454861b39b33823cb6876f3eca598` |

## Legacy references used by Stages 6–17

The semantic reconstruction imported during repository centralization was built from an older evidence bundle. Those binaries are different and must not be treated as address-equivalent to the current Orange Pi package.

| Component | Legacy size | Legacy SHA-256 | Current status |
| --- | ---: | --- | --- |
| BCM43752A2 Wi-Fi | 857142 | `bfcdc3ecb5274745f3c3551abd0d9b11ede89b305837241364a055fefbf09de7` | Stage 6–17 ROM addresses/closure are legacy evidence pending current-image IDA rebaseline |
| BCM4362A2 Bluetooth | 73136 | `3e4a1eddaf80f3e45f99e9c77b3cd84c85f605540da5f4f92300b80bca6d67ec` | Stage 6–17 patch semantics are legacy evidence; Stage 18 freezes the current structural layout and starts current-image IDA rebaseline |

This distinction is intentional and fail-safe: source is retained as useful evidence, but no claim of binary equivalence is made.

## Stage-19 evidence correction

The Stage-18 result package was re-audited before semantic porting. Both current-image `ida.log` files contain only `License not yet accepted, cannot run in batch mode`, and no `report.json`, call graph, assembly export, or decompiler output exists. Stage 19 therefore invalidates the earlier "IDA evidence generated" status. Exact-byte relocation metadata is derived independently from the legacy evidence images and current Orange Pi references; fresh IDA output is accepted only when the runner verifies non-empty exported reports.

## Stage-20 evidence class

Stage 20 adds evidence without requiring IDA:

1. **Context-disambiguated exact body** — the complete legacy function body occurs multiple times globally in the current image, but exactly one occurrence lies between the nearest monotonic unique exact anchors.
2. **Exact-thunk target** — the current 4-byte Thumb branch instruction is byte-identical to the legacy thunk instruction and is decoded again at its current address.
3. **Exact-body direct-call target** — a direct Thumb `BL` instruction inside a Stage-19 exact complete-body anchor is byte-checked and decoded on the current image.

These classes identify current structure/control-flow only. They do not by themselves establish unchanged callee implementations or whole-image semantic equivalence.

## Stage-21 evidence class

Stage 21 adds **relocation-normalized body identity**. The current body must match the legacy complete-body hash after canonicalizing only direct Thumb branch immediates; branch offsets and kinds are checked independently, and literal-pool words are read from the current image rather than assumed unchanged. Current embedded strings are also verified for named Wi-Fi functions.

This is stronger than address-delta or similarity matching, but it remains explicit about dependencies: same-address ROM calls become current ABI boundaries, while unresolved ROM semantics remain unresolved.

## Stage-22 semantic-promotion rule

Stage 22 does not byte-compare ROM routine bodies because those bodies are absent from the current RAM/PatchRAM reference files. A current ROM role is promoted only when all three conditions hold: the current verified function reaches the same absolute ROM target at the same normalized call site, the legacy behavioral classification is strong, and the surrounding current function semantics/literals remain independently verified. All other boundaries remain generic or unresolved.

## Stage-23 evidence class

Stage 23 combines two conservative evidence classes:

1. **Current unresolved ABI shape** — stable ROM address plus current relocation-normalized caller and exact direct-branch offset/kind. This freezes argument/return shape without assigning semantics.
2. **Unique relocation-normalized table consumer** — a complete legacy PatchRAM function has exactly one normalized match in the current executable region after canonicalizing only direct Thumb branch immediates; current literal words and branch destinations are then independently checked.

No proprietary firmware bytes are committed. The Stage-23 tools carry hashes/normalized metadata only and extract current Orange Pi references outside the repository at runtime.

## Stage-24 evidence class

Stage 24 promotes wrapper/control-plane source only when a complete legacy function has exactly one relocation-normalized current match and all current literals/direct targets are independently re-read. Opaque ROM boundaries remain traits or structural addresses.

This adds no IDA-derived current claims and does not identify Wi-Fi `0x11d54`/`0x7616c` or Bluetooth `0x151bc` by vendor symbol.

## Stage-25 evidence class

Stage 25 extends relocation-normalized identity with current control-plane composition. Each promoted wrapper/dispatcher must have one complete normalized match in the current image, and all current literal words, direct targets, and relevant embedded strings are independently re-read.

Source promotion stops at unresolved ROM exits. Wi-Fi `0x12d10`/`0x6fdac` and Bluetooth `0x8d34c`/`0x6e4b4`/`0x11ea8` remain unnamed boundaries even though their call shapes and order are now frozen by current callers.

## Stage-26 evidence class

Stage 26 promotes only complete functions with exactly one relocation-normalized match in the current PatchRAM image. Every current literal word and direct branch target used by the source model is re-read from the current bytes. The mode-machine and init-wrapper ROM callees remain unnamed traits; only their current addresses, argument values, ordering, and return/control effects visible in the caller are frozen. The post-init MMIO function contains no external calls, so its four register operations are lifted directly from the uniquely matched current body.
