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
