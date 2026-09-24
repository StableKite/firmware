# AP6275P reconstruction documentation

- `provenance.md` — exact current and legacy reference identities and the Stage-18 correction.
- `bcm43752-wifi.md` — Wi-Fi status and current-image rebaseline boundary.
- `bcm4362a2-bluetooth.md` — HCD profile correction and current PatchRAM layout.
- `upstream-sync.md` — upstream tracking policy (unchanged from centralization).

Stage 18 deliberately prioritizes provenance correctness over new semantic claims.

Stage 19A records cryptographic exact-byte relocation anchors independently of IDA. Verified fresh IDA output is still required before any current-image call-graph or decompiler conclusions are accepted.

Stage 20 extends Stage 19A with IDA-independent current control-flow anchors and context-disambiguated exact structural relocations.

Stage 21 follows Stage-20 current destinations and verifies a relocation-normalized destination closure, including current literal-pool/string evidence and stable ROM call boundaries.
Stage 22 promotes only ROM-boundary roles supported by stable current call sites plus prior behavioral recovery, and adds libre current `dngl_finddev` / Bluetooth empty-slot semantics.

Stage 23 freezes the ABI shapes of the four still-unresolved current Wi-Fi ROM targets and reconstructs the current Bluetooth eight-record slot-table lifecycle from unique relocation-normalized consumers.

Stage 24 lifts verified current wrapper/reset semantics into Rust while retaining unresolved ROM calls as explicit traits.

Stage 25 lifts current deadman-control wrappers and Bluetooth pair/event/control dispatch while preserving opaque ROM exits as traits/routes rather than guessing vendor symbols.

Stage 26 lifts the verified current Bluetooth mode machine, init wrapper, and post-init MMIO program while keeping unresolved ROM contracts opaque.
