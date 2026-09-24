# BCM43752A2 Wi-Fi reconstruction

StableKite `main` removes `ap6275p/fw_bcm43752a2_pcie_ag.bin` and carries the Rust reconstruction in `../bcm43752-fw/`.

## Stage-18 correction

Stages 6–17 were recovered from legacy image SHA-256 `bfcdc3ecb5274745f3c3551abd0d9b11ede89b305837241364a055fefbf09de7` (857142 bytes). Its firmware string identifies version `18.35.387.23.57`, build `2021-08-03T09:39:42Z`, FWID `01-ea656a70`.

The current Orange Pi image is SHA-256 `6a2dbe01e72221defba91a52e158768d973a3c85ca2d881c924379e35ad36b23` (936074 bytes), identifying version `18.35.387.23.146`, build `2022-07-12T10:55:29Z`, FWID `01-93c53be6`.

Therefore the Stage-17 closure audit (5219 observed call-site weight; 3032 unresolved across 21 targets, top legacy target `0xF030`) remains valid only for the legacy evidence image until current-image disassembly relocates or re-identifies those routines.

Stage 18 records both identities in Rust. Its attempted batch IDA/Hex-Rays pass did not produce usable current-image evidence, so no old function address is promoted merely because names or firmware family match.

## Stage-19 exact relocation anchors

Inspection of the Stage-18 result archive showed that its `ida.log` contains only `License not yet accepted, cannot run in batch mode`; therefore Stage 18 produced no usable current-image IDA map. Stage 19 treats that pass as invalid and independently compares legacy function fingerprints against the current Orange Pi bytes.

Among 3,045 legacy IDA functions of at least 8 bytes that fit inside the legacy RAM image, 609 have one exact full-byte match in the current image, 37 have multiple exact matches, and 2,399 have no exact match. This is relocation evidence, not whole-image equivalence.

High-value uniquely matched named anchors:

| Legacy | Current | Bytes | Evidence |
| --- | --- | ---: | --- |
| `dngl_getdev_by_ifidx` `0x184528` | `0x185920` | 32 | exact complete function body |
| `j_dngl_sendwl` `0x1847f4` | `0x185bec` | 4 | exact unique Thumb thunk |
| `j_hnd_free` `0x1a390e` | `0x1a5cc6` | 4 | exact unique Thumb thunk |
| `j_nullsub_56` `0x1a45a8` | `0x1a6960` | 4 | exact unique Thumb thunk |

Only the 32-byte `dngl_getdev_by_ifidx` body is promoted as a direct current semantic anchor at this stage. The tiny thunks are useful seeds for fresh IDA analysis but their downstream targets must be followed in the current image. Legacy ROM-boundary addresses and the Stage-17 closure are not transferred by this table.
