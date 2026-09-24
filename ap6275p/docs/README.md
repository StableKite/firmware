# AP6275P reconstruction documentation

- `provenance.md` — exact current and legacy reference identities and the Stage-18 correction.
- `bcm43752-wifi.md` — Wi-Fi status and current-image rebaseline boundary.
- `bcm4362a2-bluetooth.md` — HCD profile correction and current PatchRAM layout.
- `upstream-sync.md` — upstream tracking policy (unchanged from centralization).

Stage 18 deliberately prioritizes provenance correctness over new semantic claims.

Stage 19A records cryptographic exact-byte relocation anchors independently of IDA. Verified fresh IDA output is still required before any current-image call-graph or decompiler conclusions are accepted.
