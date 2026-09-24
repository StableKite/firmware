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
