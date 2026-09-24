# BCM4362A2 Bluetooth PatchRAM reconstruction

StableKite `main` removes `ap6275p/BCM4362A2.hcd` and carries source in `../bcm4362a2-patch/`.

## Stage-18 correction

Stages 6–17 used legacy HCD SHA-256 `3e4a1eddaf80f3e45f99e9c77b3cd84c85f605540da5f4f92300b80bca6d67ec` (73136 bytes). The current Orange Pi HCD is SHA-256 `f7adf14413063f14b0204684fb67ddcd2ae6bca3120343cb9a5cab86a1a545c3` (91900 bytes). They are distinct PatchRAM programs.

The HCD WRITE_RAM records are not ordered by destination address. Earlier Stage-17 code incorrectly tested contiguity in stream order. Stage 18 fixes this by sorting recovered WRITE_RAM extents before checking gap/overlap-free coverage.

| Profile | WRITE_RAM | Payload bytes | Code | Data | Config | Launch |
| --- | ---: | ---: | --- | --- | --- | --- |
| Legacy | 462 | 69895 | 414 writes, `[0x160400,0x16e7a8)` | 8 writes, `[0x221d9c,0x2224cc)` | 40 writes, `[0x240000,0x24262f)` | one sentinel |
| Current Orange Pi | 575 | 87868 | 518 writes, `[0x160800,0x17298c)` | 9 writes, `[0x221d9c,0x222578)` | 48 writes, `[0x240000,0x242dd4)` | one sentinel |

Stage 18 freezes the current structural profile but does not assert that legacy patched-function addresses or meanings survived unchanged. Fresh current-image IDA evidence is collected before semantic porting.
