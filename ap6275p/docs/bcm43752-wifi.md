# BCM43752A2 Wi-Fi reconstruction

StableKite `main` removes `ap6275p/fw_bcm43752a2_pcie_ag.bin` and carries the Rust reconstruction in `../bcm43752-fw/`.

## Stage-18 correction

Stages 6–17 were recovered from legacy image SHA-256 `bfcdc3ecb5274745f3c3551abd0d9b11ede89b305837241364a055fefbf09de7` (857142 bytes). Its firmware string identifies version `18.35.387.23.57`, build `2021-08-03T09:39:42Z`, FWID `01-ea656a70`.

The current Orange Pi image is SHA-256 `6a2dbe01e72221defba91a52e158768d973a3c85ca2d881c924379e35ad36b23` (936074 bytes), identifying version `18.35.387.23.146`, build `2022-07-12T10:55:29Z`, FWID `01-93c53be6`.

Therefore the Stage-17 closure audit (5219 observed call-site weight; 3032 unresolved across 21 targets, top legacy target `0xF030`) remains valid only for the legacy evidence image until current-image disassembly relocates or re-identifies those routines.

Stage 18 records both identities in Rust and generates fresh IDA/Hex-Rays evidence from the current Orange Pi reference. No old function address is promoted to the current image merely because names or firmware family match.
