# BCM43752A2 Wi-Fi reconstruction

The original Orange Pi AP6275P package carries `fw_bcm43752a2_pcie_ag.bin` as the executable FullMAC runtime firmware. On StableKite `main` that binary is removed and the Stage-17 Rust reconstruction lives in `../bcm43752-fw/`.

## Stage-17 status

The reconstruction models recovered ROM/runtime boundaries, memory primitives, TLV/NVRAM handling, ROM-call classification, and dependency closure. The frozen Stage-7 top-target audit has total observed call-site weight 5219: 505 classified as libre source, 1014 as hardware-trait boundaries, 668 as terminal sinks, and 3032 still unresolved across 21 targets. The highest unresolved target is `0xF030` with observed weight 628. These weights are static call-site evidence, not runtime frequency estimates.

CLM and NVRAM/config files are not removed by this track because they are distinct data/configuration inputs rather than the executable firmware image being reconstructed.
