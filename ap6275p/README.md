# AP6275P — brīwai atkurstan Wi-Fi be Bluetooth firmware

Šis katalogs turri dwāi StableKite Rust rekonstrukcijans per Orange Pi 5B **AP6275P**:

- `bcm43752-fw/` — BCM43752A2 Wi-Fi runtime firmware rekonstrukcija;
- `bcm4362a2-patch/` — BCM4362A2 Bluetooth PatchRAM rekonstrukcija.

Originalai `fw_bcm43752a2_pcie_ag.bin` be `BCM4362A2.hcd` ni ast en šisse `main` greiwā. CLM, NVRAM be kitāi upstream dātai palīkstan ni etwerptai.

**Stage 18 etwerpsnā:** senesnāi Stage 6–17 rekonstrukcijans twērai sen aldāks reference-bildans. Nāuns Orange Pi reference ast Wi-Fi `6a2dbe01…` be Bluetooth `f7adf144…`; šisse stage stawīdi tikkan provenance be strukturin profilin, en tēisan nāun IDA semantikan ni mazēi ast perkelta.

`upstream-master` ast tikka spīgelis `orangepi-xunlong/firmware:master`; `main` ast StableKite greiwa sen brīwai rekonstrukcijans.

Tehniskai dokumentāi ast en `docs/`.

> Prūsiskan teksts ast rekreaciōnin/atgīwinātas Prūsiskan; tehniskai nāmmai palīkstan en originalin formā.

**Stage 19A:** legacy→current exact-byte relocation anchors are recorded separately from heuristic/IDA semantics. The Stage-18 batch IDA claim was invalidated after inspection: both Stage-18 IDA logs only reported an unaccepted batch-mode license, so no Stage-18 current disassembly is treated as evidence.

**Stage 20:** current direct Thumb branch targets are derived only after re-checking exact current instruction bytes; repeated exact bodies may be structurally disambiguated by monotonic neighboring anchors without being promoted to semantics.

**Stage 21:** Stage-20 destinations are followed into relocation-normalized current bodies; current literal pools/identity strings are checked separately, and stable ROM call boundaries are recorded without inventing unresolved ROM semantics.
**Stage 22:** current verified call sites rebase selected ROM boundary roles (`0xA814`, `0x70E10`, BT memset/memcmp/stack-guard) and add source-level `dngl_finddev` / empty-slot semantics without requiring IDA.
