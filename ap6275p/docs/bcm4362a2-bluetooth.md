# BCM4362A2 Bluetooth PatchRAM reconstruction

The original AP6275P package carries `BCM4362A2.hcd`, a Broadcom HCD/PatchRAM command stream. StableKite `main` removes that HCD and carries the Stage-17 Rust reconstruction in `../bcm4362a2-patch/`.

## Stage-17 exact profile

All 462 recovered `WRITE_RAM` records form three contiguous destination regions:

- code: 414 writes, 58,280 bytes;
- data: 8 writes, 1,840 bytes;
- configuration: 40 writes, 9,775 bytes.

Total classified payload: 69,895 bytes. The recovered profile has one sentinel launch and no explicit launch record.
