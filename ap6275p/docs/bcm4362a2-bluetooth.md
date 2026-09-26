# BCM4362A2 Bluetooth PatchRAM reconstruction

StableKite `main` removes `ap6275p/BCM4362A2.hcd` and carries source in `../bcm4362a2-patch/`.

## Stage-18 correction

Stages 6–17 used legacy HCD SHA-256 `3e4a1eddaf80f3e45f99e9c77b3cd84c85f605540da5f4f92300b80bca6d67ec` (73136 bytes). The current Orange Pi HCD is SHA-256 `f7adf14413063f14b0204684fb67ddcd2ae6bca3120343cb9a5cab86a1a545c3` (91900 bytes). They are distinct PatchRAM programs.

The HCD WRITE_RAM records are not ordered by destination address. Earlier Stage-17 code incorrectly tested contiguity in stream order. Stage 18 fixes this by sorting recovered WRITE_RAM extents before checking gap/overlap-free coverage.

| Profile | WRITE_RAM | Payload bytes | Code | Data | Config | Launch |
| --- | ---: | ---: | --- | --- | --- | --- |
| Legacy | 462 | 69895 | 414 writes, `[0x160400,0x16e7a8)` | 8 writes, `[0x221d9c,0x2224cc)` | 40 writes, `[0x240000,0x24262f)` | one sentinel |
| Current Orange Pi | 575 | 87868 | 518 writes, `[0x160800,0x17298c)` | 9 writes, `[0x221d9c,0x222578)` | 48 writes, `[0x240000,0x242dd4)` | one sentinel |

Stage 18 freezes the current structural profile but does not assert that legacy patched-function addresses or meanings survived unchanged. Stage 19A permits only unique complete-body byte-identity anchors; current-image call-graph/decompiler claims remain pending verified IDA evidence.

## Stage-19 exact relocation anchors

Stage-18 batch IDA evidence is invalid: its log contains only the unaccepted-license diagnostic. Stage 19 therefore begins with exact full-function byte identity against the current reconstructed PatchRAM code region.

Among 266 legacy IDA functions of at least 8 bytes, 44 have one exact full-byte match in the current executable region, 9 have multiple exact matches, and 213 have no exact match. The following named/structural anchors are unique:

| Legacy | Current | Bytes | Meaning retained from exact body |
| --- | --- | ---: | --- |
| `0x160800` | `0x160800` | 368 | unnamed structural patch anchor |
| `bt_index_stride20` `0x162f4c` | `0x163668` | 12 | returns 20 × an 8-bit global index |
| `bt_set_global_60` `0x169104` | `0x16b7a4` | 10 | stores constant 60 to a PC-relative current-image global and returns zero |
| `bt_u32_gt_2` `0x16cc80` | `0x170178` | 10 | unsigned argument > 2 helper |
| `bt_find_first_slot_state1` `0x16e010` | `0x1720c0` | 24 | loop/selection helper; callees still require current-image traversal |

The legacy `bt_object_state_is_2` at `0x1604d4` lies before the current PatchRAM code start (`0x160800`) and is not claimed to exist in the current HCD. Other legacy names are likewise not transferred unless current evidence supports them.

## Stage-20 current control-flow anchors

Stage 20 verifies direct Thumb branch instructions on the **current** PatchRAM code. It does not infer targets from a global address delta.

The exact `bt_find_first_slot_state1` body at `0x1720c0` contains a verified `BL` at `0x1720c6`; decoding the current instruction gives target `0x172044`. This proves the current helper destination for that call site, but does not claim that the helper body is unchanged from legacy `sub_16DF94`.

The 368-byte exact structural anchor at `0x160800` remains at the same current address. Stage 20 verifies 19 direct `BL` instructions inside it. They reach eight distinct ROM-side destinations:

`0x202c0`, `0x202e8`, `0x21f0c`, `0x222a4`, `0x224a8`, `0x43d2c`, `0x443fc`, and `0x45dd4`.

These addresses are current direct-call boundaries proven from current instruction bytes. No semantic label is assigned to the ROM routines until separately recovered.

For repeated exact bodies, all 44 globally unique >=8-byte BT matches form a monotonic spine. Three of the nine globally repeated exact bodies have exactly one occurrence between their nearest monotonic neighbors and are recorded as context-disambiguated structural relocations.

## Stage-21 helper-body verification

Following the Stage-20 `bt_find_first_slot_state1` call target to current `0x172044`, Stage 21 verifies all 62 helper code bytes against legacy `sub_16DF94` after canonicalizing only the three direct `BL` immediates. The normalized hashes are identical and all three calls still reach the same ROM addresses: `0x3d24`, `0xf8cac`, and `0x94c0`.

The helper's literal pool is checked separately. The context/guard word remains `0x200890`; the 7-byte per-slot record base moves from legacy `0x222e42` to current `0x222fd6`. The instruction sequence still computes `7 * index`, so the eight indices scanned by `bt_find_first_slot_state1` correspond to current record starts `0x222fd6 .. 0x223007` in steps of seven bytes.

This establishes the current helper's record-addressing/control-flow shape. The exact vendor identities of ROM routines `0x3d24`, `0xf8cac`, and `0x94c0` remain unresolved; Stage 21 therefore does not rename those ROM boundaries.

## Stage-22 slot-helper semantics

Stage 21 proves that current helper `0x172044` has the same complete instruction skeleton as legacy `sub_16DF94`, while independently verifying the relocated record base `0x222fd6`.

The helper's three direct ROM calls remain at the same absolute addresses and call-site offsets:

- `0x3d24` — memset-like: zeroes a 7-byte local comparison buffer;
- `0xf8cac` — memcmp-like: compares `record_base + 7 * index` with that zero buffer;
- `0x94c0` — stack-guard terminal sink, reached only if the canary changes.

Therefore the helper returns `1` exactly when the selected 7-byte slot record is all zero (apart from the terminal stack-canary failure path). Combined with current `bt_find_first_slot_state1` at `0x1720c0`, the higher-level behavior is now reconstructed as: **return the first empty slot in indices 0..7, or 8 if none is empty**.

The Rust source adds `bt_slot_record_is_empty` and `bt_find_first_empty_slot` as allocation-free libre equivalents.

## Stage-23 slot-table lifecycle

Stage 23 expands the Stage-22 empty-slot semantics into the current table lifecycle. Five legacy functions have exactly one relocation-normalized complete-body match in the current PatchRAM code, with current literal words and direct branch targets independently re-read:

| Role | Legacy | Current | Current evidence |
| --- | --- | --- | --- |
| find matching key+payload | `0x16dfdc` | `0x17208c` | payload base `0x222fd7`, `memcmp-like 0xf8cac` |
| clear all eight records | `0x16e2b0` | `0x1723c0` | table base `0x222fd6`, `memset-like 0x3d24` |
| insert if absent | `0x16e2c0` | `0x1723d0` | lookup `0x17208c`, empty-slot scan `0x1720c0`, `memcpy-like 0x3db4` |
| remove matching record | `0x16e348` | `0x172458` | lookup `0x17208c`, `memset-like 0x3d24` |
| reset-table tail | `0x16e370` | `0x172480` | ends by clearing 56-byte table; other reset side effects remain outside the libre model |

The recovered record is exactly seven bytes: one tag byte followed by six payload bytes. The reconstructed source preserves firmware return behavior: insertion returns `0` both for an existing duplicate and a successful new insert, returns `17` when all eight slots are occupied; removal returns `1` when a record was cleared and `0` when absent.

A deliberate edge case is preserved: key `0` with a six-byte zero payload matches an already-empty all-zero record.

## Stage-24 adjacent control-plane semantics

Stage 24 extends the verified slot-table lifecycle with three source-level operations.

Current `0x172480` is the unique relocation-normalized match of legacy `sub_16E370`. Current literals prove mode byte `0x223064`, auxiliary flag `0x222fd0`, opaque reset contexts `0x22304c`/`0x223068`, and table base `0x222fd6`. When mode is nonzero the firmware calls stable boundary `0x151bc` with context A; when mode is 4 it also calls context B and clears the auxiliary flag. It then clears mode and always clears all 56 table bytes. The exact role of `0x151bc` remains opaque and is a Rust trait.

Current `0x1724c8` removes a six-byte payload under tag 0, retrying tag 1 only when tag 0 was absent.

Current `0x1724e4` clears a 59-byte scratch buffer at `0x22300e`, stores the low byte of `input_len >> 1` in byte 0, and copies `min(input_len,58)` source bytes into bytes 1.. using the stable memset/memcpy boundaries.

Two adjacent functions are retained as structural seeds only: current `0x172408` (legacy `sub_16E2F8`) calls stable `0x8d34c` then current insert `0x1723d0`; current `0x172430` dispatches by mode through `0x172408`, `0x6e4b4`, and `0x11ea8`. Their external ROM contracts are not yet promoted.

## Stage-25 pair configuration and event/control dispatch

Stage 25 follows the Stage-24 structural seeds and verifies a larger current control-plane cluster by unique relocation-normalized complete-body identity.

Current pair configuration is now source-level:

- `0x1721ec` (legacy `sub_16E13C`) accepts selector 0 or 1 plus a nonzero value byte, writes the selected two-byte `{key,value}` pair at `0x222fd2`, marks shared dirty/aux byte `0x222fd0`, and returns 0; invalid selector/value returns 18.
- `0x172220` (legacy `sub_16E170`) searches those two pairs when the dirty byte is set and otherwise returns fallback byte `0x22207c`.

The event path is also lifted conservatively. Current `0x172408` checks event byte `+8 == 8` and byte `+13 != 0`, calls stable-but-unnamed `0x8d34c` with the event u16 at `+11`, then feeds object tag `+130` and six-byte payload `+124` to current slot insertion `0x1723d0`. A failed guard returns the original event pointer-shaped value; a lookup miss returns zero. Current `0x172430` routes modes 2/3 directly to opaque `0x11ea8`; all other modes run the adapter first and then tail to opaque `0x6e4b4` with the original event pointer restored.

Current `0x172594` implements command opcodes 1..4: capped payload preparation, byte store at `0x222fd1`, pair-config update, and current mode-machine invocation `0x172518`. Unknown opcode returns 18. Current `0x1725dc` routes mode 1 to current `0x17218c`, mode 2 to `0x1720e8`, and otherwise returns.

Current `0x172518` and init wrapper `0x1725f8` are verified normalized matches but remain structural seeds because several external ROM contracts are still unresolved.

## Stage-26 mode/init/MMIO lifting

Stage 26 closes the two Stage-25 structural seeds without assigning vendor names to their ROM dependencies. Current `0x172518` is the unique relocation-normalized match of legacy `sub_16E408`; all four current direct targets remain at `0x72b24`, `0x151fe`, `0x151bc`, and `0x15180`, while current literals independently resolve mode `0x223064`, mirrored byte `0x223065`, source byte `0x222084`, interval word `0x22208c`, context `0x22304c`, and callback Thumb address `0x171ff9`.

The source model preserves the exact switch: mode 0 performs the `0x72b24(0)` prelude, changes mode to 1, mirrors the source byte, invokes the four-argument `0x151fe` boundary and then the common `0x15180` boundary; mode 1 mirrors the byte, invokes `0x151bc`, then the same common boundary; modes 2..4 become 5. The function returns 0 only when the resulting mode is below 2, otherwise 3. All four ROM contracts remain opaque traits.

Current `0x1725f8` uniquely matches legacy `sub_16E4E8`. It zeroes 268 bytes at `0x217c8c`, calls `0xbfad0(0)`, conditionally feeds that result to `0xbf9f4` when bit 2 of word `0x320180` is clear, then invokes stable boundaries with contexts `0x217d48` and `0x217d6c`. The final seven-argument boundary `0x144bc` receives workspace `0x217c8c`, config `0x222570`, kind 23, callback Thumb address `0xbfc11`, two zero arguments, and the current u16 value from `0x204bc8`. The Rust model preserves this sequence while keeping the five unresolved ROM calls behind a trait.

Stage 26 also recovers current `0x17294c`, the sole relocation-normalized match of legacy `sub_16E768`. It has no external calls and directly programs four 32-bit MMIO operations: write 10 to `0x423758`; OR `0x800` into `0x64085c`; replace bits `0xF80` with `0x100` at `0x640834`; and replace bits `0xF8` with `0xE8` at `0x420be0`. These operations are exposed through a hardware-register trait rather than raw host memory access.

## Stage 28 — current state/MMIO primitives

Stage 28 promotes a set of small BCM4362A2 helpers only where complete current function bodies are globally unique relocation-normalized matches and their literal targets were re-read in the current PatchRAM image.

Verified current addresses include index-stride helper `0x163668` (global `0x203160`), object field reset `0x165252`, byte-18 limit check `0x1653b4` (limit `0x203034`), entry-span helper `0x16aa04` (table base `0x20d770`, stride 78), global-60 setter `0x16b7a4` (global `0x202c6d`), unsigned `>2` helper `0x170178`, field-5 setter `0x1714d4`, and four MMIO helpers at `0x171dec`, `0x171e04`, `0x171e8c`, and `0x171e9c`.

The two polling routines preserve their exact bounded behavior: at most 100 reads, returning 1 on the first signed-nonnegative value at `0x650318` or first bit-30-set value at `0x650310`, otherwise 0 after the 100th failing read. The register helpers clear bit 3 at `0x650314` by read-modify-write and return bits 16..18 of `0x65031c`.

The object reset writes byte `+129 = 0`, byte `+19 = 2`, and the caller value at `+28`. The entry-span helper returns `entry[11] + entry[12] + 13` for 78-byte records. The large legacy routine `sub_16E550` has no current normalized complete-body match and is explicitly not promoted by Stage 28.

## Stage-29 low-level programming/countdown closure

Stage 29 extends the Stage-28 MMIO helpers with five complete current functions. Each has exactly one relocation-normalized match in the current PatchRAM code, and every literal/direct branch used below is re-read from the current bytes rather than inferred from a global delta.

- Current `0x171e1c` (legacy `sub_16DD6C`) first rejects when bit 4 of `0x650310` is set. Otherwise it writes the four little-endian bytes of current source word `0x222554` to `0x650328`, writes `0x81000000` to `0x650318` for each byte, invokes the existing bounded signed-status poll after each write, invokes the existing bit-30 poll once after all four bytes, then sets bit 3 of `0x650314`. The poll return values are ignored exactly as in the firmware.
- Current `0x171eac` (legacy `sub_16DDFC`) writes a program value to `0x650328`, encodes `(index << 8) & 0x1ff00 | 0x85000000` into `0x650318`, and tail-dispatches the existing signed-status bounded poll.
- Current `0x171ed0` (legacy `sub_16DE20`) computes a stable pending mask as `requested & ~MMIO[0x651000+offset]`, returns success immediately when it is zero, and otherwise retries the program-word helper at most 32 times. The pending mask is intentionally not recomputed; only the status word is re-read after each attempt.
- Current `0x171fdc` (legacy `sub_16DF2C`) toggles current command byte `0x222fd1` to boolean 0/1 and tail-dispatches the independent byte from `0x202fd4` to still-opaque boundary `0xbac58`.
- Current `0x171ff8` (legacy `sub_16DF48`) decrements byte `0x223065`, interprets the new byte as signed, and when it is `<= 0` and current mode `0x223064` is exactly 1 or 2, passes a pointer to local u16 value `58` to still-opaque boundary `0x2ce90`. Otherwise the incoming R0 value is preserved.

The larger legacy `sub_16DE54` remains deliberately unpromoted: although adjacent helpers are now verified, its wider copy/program contract needs additional current evidence. The ROM boundaries `0xbac58` and `0x2ce90` remain unnamed traits. Compiler stack-canary plumbing to stable terminal boundary `0x94c0` is not reproduced by safe Rust.
## Stage 30 — current byte-program transaction

Stage 30 closes the low-level programming path around the Stage-29 primitives. Legacy `sub_16DE54` has exactly one relocation-normalized complete-body match in the current PatchRAM code: current `0x171f04`, 206 bytes. All nine direct branch sites retain the same instruction kind and offset. RAM-side calls relocate coherently to current `0x171e9c`, `0x171e1c`, `0x171ed0`, and `0x171e8c`; ROM `memcpy` (`0x3db4`) and stack-guard (`0x94c0`) remain at the same absolute addresses. The guard/context literal remains `0x200890`.

An independent whole-current-code scan finds no second normalized match. The larger current caller corresponding to legacy `sub_169548` is likewise unique at `0x16be1c`; its direct calls at function offsets `+0xb4` and `+0x10a` both target current `0x171f04`. The legacy/current-normalized caller supplies the two observed shapes: a 46-byte record update at offset `46*index+128`, and a 1-byte update at `46*index+129`.

The Rust reconstruction now models the complete transaction while composing the already recovered Stage-28/29 MMIO primitives. It clears the output byte count, gates on mode bits equal to 2, begins source-word programming, handles a leading unaligned fragment by shifting bytes into the correct 32-bit lanes, programs subsequent chunks in at most four bytes, preserves the firmware's 8-bit wrapping written count, and clears control bit 3 on both success and the first programming failure. Safe Rust intentionally omits compiler stack-canary plumbing. No vendor name is assigned to the routine.

Two Wi-Fi allocation wrappers (`0x1a6408` and `0x1a645c`) were also reverified as globally unique relocation-normalized current bodies during Stage 30, but remain structural-only because their `0x703c0`/`0x705a4` contracts are still opaque.

## Stage 31 — current 8×46-byte record command handler

Stage 31 promotes the larger current caller at `0x16be1c` (legacy `sub_169548`), whose unique relocation-normalized identity was already established during Stage 30. Current bytes were re-read again before promotion: its nine direct call sites target `0x780`, `0xf620`, `0xf614`, `0x3db4`, current Stage-30 transaction `0x171f04` twice, `0x3db4`, `0x780`, and stack-guard terminal `0x94c0`. The current literal pool still supplies guard/context `0x200890`, status MMIO `0x650310`, and active-bit register `0x6408d8`.

The handler operates on a 368-byte window containing eight 46-byte records. Record byte 1 is the state byte and bytes 4..45 are a 42-byte payload. Command 0 reads one indexed record. Command 1 accepts only state 0, builds `[index,1,0,0,payload42]`, programs all 46 bytes through current `0x171f04`, and changes response status to 3 if the firmware-reported written count is not 46. Command 2 accepts only state 1 and programs byte `0x0f` at record offset +1. Command 3 ORs bits for state-0 records into the caller-provided bitmap; the firmware does not clear pre-existing bitmap bits first. Invalid command values above 3 return status 18 before entering the critical/read path; invalid record indices take the normal cleanup path.

The still-unidentified `0x780`, `0xf620`, and `0xf614` boundaries remain explicit traits. Stage 31 records only their observed call shapes: enter/leave token, conditional prepare when MMIO status bit 12 is clear, and the offset-128 / length-368 record-window read. No vendor symbol is assigned to them.

## Stage 32 — active-record replay and command-class-5 adapter

Stage 32 continues outward from the Stage-31 8x46-byte record command handler using the same IDA-independent evidence rule: a complete legacy function body must have exactly one relocation-normalized match in the current PatchRAM image, and current literals/direct targets are re-read independently.

Legacy `sub_1696AC` has one current normalized match at `0x16bf80` (104 bytes). Its six direct calls remain `0x780`, `0xf620`, `0xf614`, `0x780`, `0xa5d24`, and stack-guard terminal `0x94c0`; current literal values remain guard/context `0x200890`, status MMIO `0x650310`, and active register `0x6408d8`. The routine enters the same critical/read path as Stage 31, conditionally prepares when status bit 12 is clear, reads offset 128 / 368 bytes, clears active bit 0, restores the critical token, and then scans all eight 46-byte records. For every record whose state byte `+1` equals 1 it calls the still-opaque boundary `0xa5d24` with selector 0 and pointer `record+4`, i.e. the 42-byte payload tail. No vendor name is assigned to `0xa5d24`.

Legacy `sub_169758` has one current normalized match at `0x16c02c` (34 bytes). Its sole current `BL` targets the already recovered Stage-25 command dispatcher at `0x172594`. Request class byte `+12` must equal 5; on that path request byte `+11` is decremented with 8-bit wrap semantics and forwarded as the dispatcher frame length, with frame bytes beginning at request `+13`. The returned dispatcher status is stored at response byte `+5`. For other request classes the firmware writes status 1 and preserves the incoming pointer-shaped return value.

Safe Rust omits compiler stack-canary plumbing but preserves ordering, status handling, wraparound, and the opaque runtime boundary as a trait.

## Stage 33 — current small control-plane helpers

Stage 33 promotes four additional BCM4362A2 functions only after whole-current-code scanning proves a single relocation-normalized complete-body match for each and current literal/direct-target values are re-read independently.

Current `0x16bff4` is the unique 52-byte match of legacy `sub_169720`. Its opaque direct targets remain unchanged at `0x89314`, `0x8957c`, and tail boundary `0x89240`; its mirrored-byte literal relocates to current `0x222708`. The source model preserves the odd control flow: response status is cleared first; request class other than 1 writes status 18 and returns the incoming pointer-shaped value; class 1 runs phase A, runs phase B only when phase A returns 1, mirrors request byte +13 in either accepted case, and tail-dispatches only when the original phase-A result was 1. No vendor role is assigned to any of the three opaque targets.

Current `0x16c228` is byte-identical to legacy `sub_169850` across all 24 bytes and has no calls or literals. It returns 239 when bit 2 of the object word at offset +564 is set, otherwise 245, then subtracts one when bit 6 is set. This pure helper is reconstructed directly.

Current `0x16c400` is the unique 22-byte normalized match of legacy `sub_169A28`. Its sole call remains opaque `0x9d3dc`; current literals are context `0x20cef8` and mark byte `0x222700`. It forwards request payload beginning at +12 to the opaque parser-like boundary, copies the low-byte result to response status +5, sets the current mark byte to one, and returns the boundary result. The boundary name remains deliberately generic.

Current `0x16c828` is byte-identical to legacy `sub_169CE8` across all 60 bytes. Current literals independently resolve count byte `0x203160`, metadata-base pointer `0x20cf10`, and 20-byte-per-index flag-table pointer `0x22257c`. The routine accepts an index only when it is below count and metadata record byte +166 has bit 0 set; valid requests write byte +2 of the indexed 20-byte record to one exactly when request byte +14 is zero, otherwise zero. Invalid index/metadata writes response status 66. The Rust backend keeps metadata and table storage abstract rather than exposing raw firmware pointers.

## Stage 34 — slot-lifecycle command wrappers and configuration setter

Stage 34 closes three dispatch-facing wrappers around the already recovered current seven-byte slot lifecycle. Whole-current-code scanning gives one relocation-normalized match for each legacy wrapper: `sub_169FE8 -> 0x16cb28`, `sub_169FFE -> 0x16cb3e`, and `sub_16A01A -> 0x16cb5a`. Their first calls remain three distinct opaque boundaries at `0x9b03c`, `0x9b070`, and `0x9b110`; Stage 34 deliberately does not assign vendor names to them. Their second branch targets relocate coherently to recovered current clear `0x1723c0`, insert `0x1723d0`, and remove `0x172458` respectively.

Current `0x16cb28` runs the opaque clear precheck and clears the eight-record table only when the existing caller-owned response status byte is zero. Current `0x16cb3e` runs its insert precheck and, only when response status is zero, inserts request tag byte +12 plus the six-byte payload beginning at +13; the insert result is copied to response status +5. Current `0x16cb5a` runs its remove precheck and, only on zero response status, tail-dispatches request tag +12 plus payload +13 into the already reconstructed remove operation; this wrapper does not itself copy the remove return into response status.

Stage 34 also promotes current `0x16c978`, which is byte-identical to legacy `sub_169E38` across all 38 bytes and has no calls. Current literals independently resolve the two-word destination at `0x222790` and the second context at `0x20cec4`. Request bytes +13/+14 and +15/+16 become two little-endian u16 values, while request bytes +12 and +17 become context bytes +31 and +32. Rust exposes this as a compact value struct rather than raw firmware pointers.

## Stage 35 — current eligibility and mask helpers

Stage 35 promotes four functions in the current `0x16d3b8..0x16d4d4` cluster only after a whole-current-code relocation-normalized scan finds exactly one match for each legacy body. Current `0x16d450` and `0x16d490` are additionally byte-identical to their legacy counterparts. Current code SHA-256 remains `7ed8c96bdbe5110a66f31ccd5b3d851064194e3a560940db655e75535141d502`.

Current `0x16d3b8` (legacy `sub_16A63C`) is a 26-byte wrapper. It always calls still-opaque boundary `0x32dd4` first. Only when object word `+36` has mask `0x110` does it use that mapped index to load a byte from the current table rooted at `0x221ebc`; otherwise it returns zero. The unconditional mapper call is preserved in Rust.

Current `0x16d3d8` (legacy `sub_16A65C`) is a unique 104-byte normalized body. Its direct call chain is current helper `0x16d3b8` followed by still-opaque boundaries `0x21f42`, `0x2a428`, and `0x2a2b8`. Current literals independently resolve to `0x208338`, `0x2091fc`, `0x209454`, and `0x208194`. The source model preserves each early return, the low-24-bit object-word gate, the reset-latch zero store before the final post-zero boundary, and the final special predicate for code `0x080b` or either of two caller-supplied object identities. The external contracts remain traits and receive no vendor names.

Current `0x16d450` (legacy `sub_16A6D4`) is byte-identical across all 56 bytes and has no calls. Current literals are enabled-mask word `0x221ec4` and indexed-mask table `0x221ec6`. The model preserves the Thumb register-shift behavior used to form the candidate bit, the conditional shifted-word/index-table preservation rules, and the final clear in object word `+36`. In particular, register shift amounts use the low eight bits and shifts at or above 32 yield zero for the positive values used here; this avoids silently substituting Rust's modulo-width `wrapping_shl` semantics.

Current `0x16d490` (legacy `sub_16A714`) is byte-identical across all 68 bytes and has no calls. It scans exactly three records at 400-byte stride from current table address `0x209d68`. A record matches only when byte `+209` has bit `0x10`, `((byte[229] + 29) & 31) <= 3`, and byte `+215` equals the full 32-bit input key. Rust exposes only those observed fields, not a guessed vendor structure name.

## Stage 36 — current 1028-byte window transaction

Stage 36 promotes current `0x16c240`, the sole whole-current-code relocation-normalized match of legacy `sub_169868` (440 bytes). The legacy and current raw hashes differ, but after canonicalizing the eleven direct branch immediates the complete normalized bodies are identical. The current branch map is: `0x8e12c`, `0x886e0`, `0x9d7c4`, recovered Stage-33 helper `0x16c228`, critical-state boundary `0x780`, `0x8f160`, `0x79aae`, two calls to the known memcpy-like `0x3db4`, and a final tail back to `0x780`. Current literal words independently resolve the window-base pointer at `0x20ce98` and the group-base pointer at `0x20be7c`.

The routine selects one window from a 16-entry family using `(context.byte555 >> 3) & 0xf`; the window stride is exactly 1028 bytes, consisting of 1024 payload bytes followed by a four-byte header. A second context bit, `(byte556 >> 2) & 1`, selects the special noncritical exit. Selector byte `request[12]` must be at most 239. A still-opaque validator consumes `request+9`, the current context, the selected window, and the special bit. Any nonzero validator result is copied to response status byte `+5` and returned unchanged.

When context flag word `+564` has bit `0x10` clear, the already recovered Stage-33 pure helper supplies the current limit code (239/245, optionally decremented by flag bit 6). For context byte `+554` mode `0x10`, operation `request[13] & 0xfd == 1` rejects when the limit is below request length byte `+15`; operation mask zero rejects when existing window low-11-bit length plus the request length exceeds the limit. Restricted flag patterns `(flags & 0x12)==2` or `(flags & 0x14)==0x14` allow only original operation 3 with zero length. All these rejection paths store status 18 and return the current result value rather than synthesizing a new return.

Only `special == 0` enters the current critical-state boundary. Original operation 4 transforms the low 12 bits of context word `+552` through still-opaque `0x8f160`, preserves the high nibble, then restores the critical token. Operations 1 and 3 (`operation & 0xfd == 1`) replace header bits 0..10 with request length and bits 11..21 with the special value, optionally notify the 676-byte-stride group selected by the same index, optionally transform context low 12 bits, copy the request payload to window offset zero, set header bit 22, and set bit 23 only for original operation 3. Other operations append at the pre-copy low-11-bit window offset, re-read the header after the copy before adding length, and original operation 2 sets header bit 23. Exact halfword/full-word/byte header write ordering is retained behind the Rust backend trait.

The external contracts at `0x8e12c`, `0x886e0`, `0x9d7c4`, `0x780`, `0x8f160`, and `0x79aae` remain deliberately unnamed traits. The source does not invent host-side payload bounds: the known `0x3db4` copy sites are represented by a backend copy operation carrying the exact selected window, destination offset, and firmware `u8` length.

## Stage 37 — current request-to-config update cluster

Stage 37 promotes three adjacent current functions after whole-current-code relocation-normalized scans find exactly one match for each legacy complete body: `sub_169D30 -> 0x16c870` (106 bytes), `sub_169DA4 -> 0x16c8e4` (94 bytes), and `sub_169E02 -> 0x16c942` (54 bytes). Their current direct targets independently resolve to the same small cluster: opaque object lookup `0x8d34c`, opaque current-context lookup `0x886e0` where applicable, current config-object boundary `0x163724`, and current commit/tail boundary `0x161268`. No vendor symbol is assigned to those unresolved boundaries.

Current `0x16c870` first interprets request bytes `+16/+17` as a little-endian index. The current global count remains at `0x203160`; metadata remains rooted through current pointer `0x20cf10` with 264-byte stride and enable bit 0 at metadata byte `+166`. An out-of-range or disabled index writes response status 66 and returns the incoming request-shaped value. The function then looks up request key word `+12`; a null lookup writes status 2, while an object whose byte `+223` lacks bit 1 writes status 26 and returns that object result. Success obtains the current config object, stores request word `+16` at config `+4`, request word `+14` at config `+2`, stores one at config byte `+11`, and tail-dispatches the current commit boundary.

Current `0x16c8e4` uses request byte `+16` as a selector and rejects values above 239 with status 18 while returning the selector value. It then requires a non-null current context from `0x886e0`; null writes status 66, while a context whose byte `+592` lacks bit 0 writes status 18 and returns the context-shaped value. The same object lookup and byte-`+223` bit-1 gate follow. Success stores selector byte `+16` at config `+10`, request word `+14` at config `+2`, clears config byte `+11`, and tail-dispatches the same commit boundary.

Current `0x16c942` is the simplest variant. It performs only the request-key lookup; null writes response status 2. On success it obtains the current config object, writes little-endian request bytes `+15/+16` at config `+6`, bytes `+17/+18` at config `+8`, request byte `+14` at config `+12`, and request byte `+19` at config `+13`. It returns the config-object result directly and does not invoke the commit boundary. The Rust reconstruction preserves these distinct return shapes and leaves successful caller-owned response status untouched.

## Stage 38 — current compare/update and flag-control routines

Stage 38 promotes two additional BCM4362A2 routines only after whole-current-code scanning proves a single relocation-normalized complete-body match for each. Legacy `sub_169E68` maps uniquely to current `0x16c9a8` (164 bytes); its current direct targets are `0x8d34c`, `0x9d3dc`, `0x86984`, `0x6e774`, and tail `0x84458`. Legacy `sub_169F0C` maps uniquely to current `0x16ca4c` (152 bytes); its current targets are `0x8d34c`, `0x86984`, `0x6e774`, and tail `0x84256`. None of these unresolved runtime entries receives a vendor name.

Current `0x16c9a8` looks up an object from request u16 `+12`. A missing object finalizes with status 2. Either object word at `+68` or `+72` carrying bit `0x2000` finalizes with status 35. Otherwise the opaque parser receives request bytes starting at `+14` and object storage beginning at `+440`. Request byte `+15 == 4` or `+16 == 4` causes the little-endian request word `+17/+18` to be stored at object `+442` even when the parser later returns nonzero. On parser success, unequal object u16 values at `+440` and `+444` set object byte `+460 |= 2`, byte `+62 |= 8`, and word `+72 |= 0x2000`, invoke the opaque update boundary, and finalize with status zero. Equal words take the separate `0x84458` tail after normal finalization.

Current `0x16ca4c` begins with the same opaque object lookup. Object byte `+61` bit `0x20` skips the update path. Current literal-backed global `0x2030bc` with bits `0x1010` both set forces object byte `+61 |= 4`. Otherwise current control byte `0x20807d` selects either byte `0x20809a` when control bit 3 is set or fallback byte `0x20ce9c` when it is clear. If the selected byte lacks bit 3 while object byte `+253` is nonzero, firmware takes a direct jump-out to current `0x6e774`; that transient register ABI remains deliberately opaque. The normal update path sets object byte `+61 |= 4` and word `+72 |= 8`, then invokes current `0x86984` with the control and selected bytes shifted into bits 28..31. Normal finalization uses status zero; if byte `+61` bit `0x20` is set after finalization, the result tail-dispatches through current `0x84256`.

The Rust reconstruction exposes both routines through traits so runtime mutation of the object/global state remains observable. In particular, current literal-backed values at `0x2030bc`, `0x20807d`, `0x20809a`, and `0x20ce9c` are not frozen as compile-time constants.

## Stage 39 — current indexed-entry helper and gated dispatch

Stage 39 promotes two more globally unique relocation-normalized current bodies. Legacy `sub_169FB0` maps uniquely to current `0x16caf0` (52 bytes), with current calls `0x9d58c`, `0x8e450`, and current RAM target `0x164444`; its current entry-base pointer literal is `0x20be7c`. Legacy `sub_16A038` maps uniquely to current `0x16cb78` (124 bytes). Its current targets are `0x33730`, `0x4f99c`, already recovered Stage-35 predicate `0x16d490`, `0x6595c`, `0x6e774`, `0x32ea4`, and stack-guard terminal `0x94c0`. Current literals independently resolve stack-guard/context `0x200890`, gate global `0x20b0f0`, and callback Thumb address `0x16d05d`.

Current `0x16caf0` always runs the opaque `0x9d58c` precheck. Only when the caller-owned response status byte is zero does it map request selector byte `+12`, compute `entry_base + 676*index`, call the opaque current entry updater with `entry+40` and request byte `+31` interpreted as signed `i8`, then copy entry byte `+83` to response byte `+6`. The Rust source preserves that status gate, 676-byte stride, signed byte behavior, and update/copy order without assigning vendor names to the runtime boundaries.

Current `0x16cb78` looks up request u16 `+12` with kind 3. A missing object uses status 2. For a present object, status 12 is selected when the current global/runtime conjunction rooted at `0x20b0f0` rejects, object byte `+28 & 0xf8` equals `0x68`, or the already recovered current Stage-35 three-record predicate at `0x16d490` matches. Otherwise opaque current `0x6595c` receives the original request pointer beginning at `+9` plus an in/out local handle slot and supplies the status. Firmware then always calls current finalizer `0x6e774` with request u16 `+9` and status. Only zero status invokes current `0x32ea4` with the resulting handle, original request pointer `+9`, and exact callback Thumb value `0x16d05d`. Safe Rust omits compiler stack-canary plumbing while retaining `0x94c0` as provenance.

## Stage 40 — current 442-byte request-selection transaction

Stage 40 promotes legacy `sub_16A0C0` only after a whole-current-code scan proves a single relocation-normalized complete-body match at current `0x16cc00`, size 442 bytes. Current branch targets independently resolve to `0x33730`, `0xbddbc`, recovered Stage-35 predicate `0x16d490`, `0x4f99c`, `0x33b8c`, `0xb0630`, `0x33d28`, `0x335ac`, and stack-guard terminal `0x94c0`; current literals retain stack-guard/context `0x200890` and gate global `0x20b0f0`. All unresolved runtime calls remain traits.

The source model preserves the firmware's exact local decision structure. The kind-3 lookup uses request u16 `+3`; missing lookup returns 2. Present objects return 12 when the current global/runtime conjunction rejects, the recovered Stage-35 three-record predicate matches, or unsigned `(object.word0 - 24) > 2` together with object byte `+256` bit 2. In non-override mode, selector u16 `+13 <= 3` or wrapped `(byte+17 - 3) <= 0xfb` returns 18. Request word `+18 == 0xffff` is first replaced by 63, then XORed with `0x03c0`, and the transformed word is written back.

Opaque current mode selector `0x33b8c` controls the remainder. Mode 1 transforms against the primary object's `+52` context through current `0xb0630`, writes the output word back to request `+18`, and returns 14 if current `0x33d28` rejects, object byte `+167` lacks bit `0x10`, or output bits `0x03f8` are nonzero; otherwise the primary object is selected and the transform's byte result is returned. Mode 0 returns 12 in normal mode but enters the secondary path in override mode. Modes other than 0/1/2 return zero.

The secondary path looks up a second object using primary byte `+215`. When found, current `0xb0630` transforms against secondary `+52`; when absent, the binary still copies the uninitialized local output word to request `+18`. Rust models this unusual ambient-stack dependency explicitly through `uninitialized_word_seed()` instead of silently inventing zero. A secondary object satisfying current `0x33d28`, byte `+167` bit `0x10`, and zero `0x03f8` output mask returns 14. Otherwise normal mode compares primary `+208 & 0x0fff` with original request u16 `+15` and applies the exact wildcard/equality rules across original request dwords `+5/+9` and primary dwords `+324/+328/+340/+344`; mismatch returns 18. Matching/override paths upgrade zero transform status to 35 when primary byte `+230` has bit `0x40`. The output object receives the secondary handle, including null.

The safe Rust API requires a non-null output slot. The original null-output branch calls current `0xbddbc` and then dereferences the pointer, so Stage 40 records that boundary but does not fabricate recoverable null semantics. Compiler stack-canary plumbing is likewise omitted from safe Rust while `0x94c0` remains provenance.

## Stage 41 — current object/request update transaction

Stage 41 promotes legacy `sub_16A4B4` after whole-current-code scanning proves a single relocation-normalized complete-body match at current `0x16d05c`, size 294 bytes. Current direct targets are `0x33730`, `0x6f438`, `0x33b8c`, `0x33d28`, `0x4ff46`, `0x6f340`, `0x32f08`, `0x54be4`, and stack-guard terminal `0x94c0`; current literal context remains `0x200890`. Runtime contracts remain unnamed traits.

The routine first performs the kind-3 lookup from request u16 `+3`; missing lookup calls the opaque missing-object boundary with status 2. For a present object it records the opaque mode result and opaque object-predicate result. When that predicate is nonzero and object byte `+167` has bit `0x10`, request word `+59` is masked with `0xfff8` before the mode-specific path. Unsupported modes other than 0/1/2 return the predicate result unchanged.

Mode 1 either uses object byte `+166` directly when object dword `+28 & 0xf8 == 0x68`, or invokes current resolver `0x4ff46`, which may return an output object. A nonzero resolver/code value calls current `0x6f340` with an exact `opcode == 1031` boolean and then current `0x32f08(primary,1,3)`. Zero code requires an output object; the binary immediately dereferences it, so Rust exposes a deliberate null-candidate fault trait instead of fabricating recovery. For a valid candidate, byte `+230` bit 7 becomes exactly `(opcode == 1031)` while preserving low seven bits, and byte `+231` bit 7 becomes exactly `(opcode == 1085)`, again preserving low bits. The candidate then enters current `0x54be4` with kind 0.

Modes 0 and 2 update the primary object. Existing dwords `+324/+328/+332/+336` are first copied to history dwords `+356/+360/+364/+368`, then request dwords `+5/+9` replace object `+324/+328`. Request bytes `+57/+58` become little-endian object word `+332`, request byte `+61` becomes object byte `+336`, and request bytes `+59/+60` become object word `+334`. The primary object then enters current `0x54be4` with kind 6. Safe Rust omits compiler stack-canary plumbing while retaining `0x94c0` as provenance.

## Stage 42 — current request/mode transaction

Stage 42 promotes legacy `sub_16A284` after whole-current-code scanning proves a single relocation-normalized complete-body match at current `0x16cdc4`, size 248 bytes. Legacy and current normalized SHA-256 are both `7061ff3accf792b6c79f9c34b0169ec51d455c1f3edcbc71bbcde88e30bc0f27`. All ten direct branch-site offsets/kinds are preserved. Stable runtime targets remain `0x33730`, `0x33b8c`, `0x33ac8`, `0x6e774`, `0x65244`, `0x32ea4`, and stack-guard `0x94c0`; the only RAM-side relocation is legacy `0x16aa10` to current `0x16d90c`. Current literals independently remain `0x200890`, `0x208338`, and callback Thumb value `0x71c61`.

The source model preserves the raw request layout because the firmware reads an unaligned completion/status id at byte `+9`, lookup key at `+12`, and control word at `+14`. After current kind-3 lookup, a missing object reports status 2. For a present object the firmware computes `control ^ 0x3306`; when all bits except bit 0 are clear it returns the opaque object handle immediately without a status call.

Mode 1 validates `(control ^ 0x3306) & 0xff1e` through still-opaque `0x33ac8`. Validation failure reports status 18; object byte `+31` bit 3 reports status 12. Otherwise the mask is stored at object word `+236`, status zero is reported, and execution continues through current relocated `0x16d90c`. Stage 42 records this current target but does not assign semantics to it yet.

Mode 0 builds the exact constant-shaped local message observed in the binary: completion id, kind 17, lookup key, fixed bytes 64/31/0/0/64/31/0/0, a template word loaded through current context `0x208338` at byte offset 77, and `(low_byte(control) >> 5)`. Current `0x65244` produces a status plus opaque output handle; that status is always reported through `0x6e774`, and only status zero continues to current `0x32ea4(out_handle,message,0x71c61)`. Modes other than 0/1 preserve the opaque mode result. Compiler stack-canary plumbing is omitted in safe Rust while retaining `0x94c0` as provenance.

## Stage 43 — current mode-1 post continuation

Stage 43 closes the current continuation reached by the Stage-42 mode-1 request path. Legacy `sub_16AA10` has one relocation-normalized complete-body match in the current PatchRAM image at `0x16d90c` (142 bytes / 47 instructions). All five direct branch sites retain their offsets and instruction kinds; current targets are `0x3ccdc`, `0x3cc9e`, `0x4bc44`, `0x6f246`, and `0x414a0`.

The source model preserves the exact local object transitions. It tests object word `+236` against mask `0x3306` and object byte `+167` against mask `0xe0`. The zero/zero branch sets byte `+31` bit 3 and either returns the object unchanged when byte `+235 & 0x30` is zero or tail-dispatches current `0x3cc9e` with argument 1 equal to zero. Commit paths copy word `+236` to word `+104`, invoke current `0x4bc44`, notify current `0x6f246` with zero, word `+100`, and `word236 ^ 0x3306`, then tail current `0x414a0` with `(object,1,0)`.

A deliberately unusual register edge is kept explicit. When both masked fields are nonzero, current `0x3ccdc` is called with the object in R0 and word `+236` in R1. If that call returns zero in R0, the binary branches directly to the commit block without restoring R1, so current `0x4bc44` receives the caller-volatile post-call R1 value. Rust represents the opaque probe result as `{r0, r1_after}` rather than silently replacing `r1_after` with a guessed stable value.

When the probe returns nonzero, byte `+31` bit 3 is set. Byte `+235 & 0x30 == 0x10` returns the probe result unchanged; other values tail current `0x3cc9e` with argument 1 equal to one. All five runtime contracts remain unnamed traits and compiler-generated call/return mechanics are not strengthened beyond the observed register-level behavior.

## Stage 44 — adjacent post-mode continuations

Stage 44 continues from the already recovered current `0x16D90C` mode-1 continuation. No external runtime entry is assigned a vendor name unless independently proven.

Legacy `sub_16AAA0` has exactly one relocation-normalized whole-current-code match at current `0x16D99C` (72 bytes). The current body preserves external targets `0x3CCDC`, `0x3CC9E`, `0x3C3B0`, and `0x2EC18`, plus shared-flags literal base `0x208830`. The source model preserves the probe gate on object byte `+29` bit 7, forwards the probe result in argument register R1 to the follow-up only when the result is `1`, and selects the flagged tail only when dword `+56` bit 3 is set, byte `+29` bit 7 is clear, and runtime dword `0x208834` bit 11 is set. Otherwise it takes the default tail.

Legacy `sub_16AAEC` has exactly one relocation-normalized whole-current-code match at current `0x16D9E8` (138 bytes). The source model preserves the initial probe/follow-up gate, the `(1,0)` mode-write call, object byte `+28` rewrite to `(old & 7) | 0x40`, global `0x2090CC == 1` conditional notification, the primary-handle map/final-predicate path, and all return-shape distinctions.

Two binary-specific ABI edges are deliberately explicit. First, dword `+52` is shifted left 21 into R2 before the bit-10 test. When bit 10 is set, opaque `0x2EB58` is called and the caller forwards *post-call caller-volatile R2* directly to `0x6E9E0`; safe Rust therefore models `0x2EB58` as producing the R2 value observed after the call instead of assuming the pre-call shift survives. Second, after opaque `0x58488` returns nonzero, the binary executes `MOVS R0,#0` before the tail branch to `0x5833C`; the tail receives zero rather than the predicate result.

Compiler stack-canary mechanics are omitted from safe Rust. All external runtime contracts remain traits.

## Stage 45 — current record-window maintenance

Stage 45 promotes legacy `sub_16AB80` only after a fresh whole-current-code scan proves one relocation-normalized complete-body match at current `0x16DB4C` (136 bytes). The current raw SHA-256 is `aa4bb94358cc26103017357c495b264f56810a935b9921a7f90b836f44bbeda8`; masking only the two direct branch encodings produces normalized SHA-256 `df101fd7b60c00fc47cabefe0462b5a7e32090fb06c7473fd05eedf25b44e034`, identical to the legacy body, with the sole current hit at `0x16DB4C`. Current direct targets remain opaque `0x3B04A` and already proven memset-like `0x3D24`. The two literal-pool constants are independently unchanged at `0x30078` and `0x30018`.

When object byte `+149` equals 2, `(dword+144 & 0x30078) == 0x30018` increments state byte `+22` with eight-bit wraparound and sets state byte `+20` bit `0x10`. Otherwise, if `(byte+144 >> 3) & 0x0f` is above 2, byte `+146 & 3` is 1 or 2, and byte `+146` bit 2 is clear, state byte `+20` gains bit `0x04`.

State byte `+15` is then tested. When nonzero, firmware clears it before calling opaque current `0x3B04A(state)` and retains that boundary's return. The source trait deliberately permits the boundary to mutate the state because firmware re-reads byte `+14` after the call. A nonzero byte `+14` is cleared, exactly 116 bytes at state `+16..+131` are zeroed through the already established memset-like semantics, state byte `+1` is cleared, and the return becomes the pointer-shaped value `state+16`, overriding any prior boundary result.

Without that clear path, state byte `+20` bit 0 suppresses index movement. When bit 0 is clear, byte `+1` increments only while it is below `byte+20 >> 5`. If neither opaque boundary nor clear path runs, the binary returns the original object pointer-shaped value. Safe Rust preserves all three return shapes using explicit 32-bit handles while keeping host pointers out of the model.

## Stage 46 — current indexed-record/state transition

Stage 46 promotes legacy `sub_16AD56` only after a repeated local whole-current-code scan confirms exactly one relocation-normalized complete-body match at current `0x16DD22` (136 bytes). Current direct calls remain `0x17E2C` at body offset `+0x0C`, `0x3AF86` at `+0x3A`, and `0x3C7C2` at `+0x46`. These runtime contracts remain deliberately unnamed.

The exact entry gate is `((object.byte152 >> 3) & 0x0f) > 2` together with `(object.byte154 & 3)` equal to 1 or 2. Before any second-argument branch, firmware stores `2 * result_17E2C` with u32 wrapping at `state + 53 + 25 * state.byte1`. No bound check on `state.byte1` is visible. Safe Rust therefore delegates this operation to an unchecked-offset backend method instead of inventing a record count or clamp.

Register-level inspection corrects one misleading Hex-Rays rendering: current `0x3AF86` is called with `R0 = result_17E2C` and `R1 = state_ptr`. The Rust boundary receives both values and a mutable state object because firmware subsequently reads state fields after that call. If the boundary returns zero, firmware sets state byte `+14` to one and returns zero. On nonzero return, state byte `+18` receives object byte `+15`, current `0x3C7C2(object)` is reduced to boolean `(return == 0)`, state word `+16` receives the post-boundary value of state word `+4`, state byte `+20` adds `0x20` with u8 wrap, byte `+19` receives the boolean, byte `+15` becomes one, and byte `+14` becomes one.

When the function's second argument is zero, no `0x3AF86`/`0x3C7C2` call occurs. Instead state byte `+20` bits 5..7 increment modulo eight while low five bits are preserved; if the new high-three-bit value exceeds three, bit 0 is forced to one. This path and both entry-gate rejection paths return the original `0x17E2C` result unchanged.

## Stage 47 — current lookup/slot transfer and packed-state update

Stage 47 targets current `0x16DE9C`, the globally unique relocation-normalized 116-byte match of legacy `sub_16AED0`. Its only direct runtime calls remain opaque current boundaries `0x1EE18` and `0xB0460`.

The routine looks up an object using input byte `+20`, unconditionally calls the release boundary on the caller-owned old slot, then replaces that slot with lookup-object dword `+16`. The release return is preserved as the function's final return throughout all later local writes. State byte `+29` becomes 2.

Packing into state halfword `+26` depends on lookup-object byte `+11 & 0xC0`. Mode `0x40` inserts `(replacement.byte2 >> 3)` into bits 3..12. Mode `0x80` inserts `(replacement.u16_at_2 >> 3)` into the same ten-bit field. Other modes leave bits 3..12 unchanged. In every mode, bits 0..2 are then replaced from replacement byte `+2 & 7`. No local null check is observed after lookup; safe Rust therefore delegates handle validity/dereference behavior to the backend rather than inventing recovery.

Finally input byte `+9` gets bit 0 set, lookup-object byte `+11` becomes `(old & 0xC0) | 0x3C`, and lookup-object dword `+16` is cleared to zero. The backend API keeps the reads/writes explicit so ownership-transfer ordering remains observable.

## Stage 48 — current lookup mode router and three-record transfer

Stage 48 targets current `0x16DF10`, the globally unique relocation-normalized 450-byte match of legacy `sub_16AF44`. The current raw body SHA-256 is `c0e223d9224b45048947c3de595360ebc1e320252c8872d7fed72617f82168bc`; the prior normalized fingerprint is `fc9857892f2ff00ca3c88d4bd66e885bd1281653d0c66a5f6813dbf2fd64da46`. The embedded tail to legacy `sub_16AED0` relocates coherently to current Stage-47 `0x16DE9C`. Other direct runtime targets remain opaque current boundaries `0x335AC`, `0x1EE18`, `0x1F3BC`, `0xB0460`, and `0x1F3E0`. The body also performs two ordered reads from ambient word address `0x3189DC`.

The function first obtains a context from `0x335AC(input.byte20)` and a lookup object from `0x1EE18(input.byte20)`. If state byte `+28 == 2` and state byte `+29` is 2 or 4, lookup byte `+11` bits 2..5 take a short path: modes 0..2 are replaced with mode 3 before the `0x1F3E0(lookup, context)` tail; mode 3 calls `0x1F3BC(lookup)`, returning zero unchanged on zero or ORing a freshly re-read lookup byte `+11` with `0x3C` and returning the nonzero boundary result; modes 4..15 return the lookup handle directly.

Outside that gate, lookup byte `+12` receives input byte `+20`. Lookup byte `+11` bits 6..7 receive class 1 or 2 from context byte `+167` and input byte `+0`: when context `& 0xE0 == 0x20`, the input bits 3..6 nibble is class 1 for values <=9 and class 2 otherwise; on the alternate path, masked values `0x18` and `0x48` select class 1 and all others class 2. Firmware then reads ambient dword `0x3189DC`, stores its low byte through argument-1 dword `+4`, reloads the ambient dword, and stores its next byte through argument-1 dword `+0`; the Rust backend keeps these two reads/stores separate.

The 16-way dispatch uses lookup byte `+11` bits 2..5. Modes 0/1 delegate directly to reconstructed Stage 47 when lookup dword `+16` is nonzero; otherwise they advance to mode 1/2 and initialize that 12-byte record unless its pointer field is occupied, in which case current `0xB0460` is tail-called on argument-1 dword `+0`. Mode 2 also delegates to Stage 47 when lookup dword `+16` is nonzero; with no replacement it reads both record-1 and record-2 pointer fields, fills record 1 and rewinds to mode 1 when record 1 is empty, otherwise fills record 2 when it is empty, and otherwise releases the current argument-1 dword `+0`. Mode 3 requires nonzero `0x1F3BC(lookup)` before filling record 0 and clearing the mode; zero takes the release tail. Mode 15 clears the mode then fills record 0. Modes 4..14 return the lookup handle.

Each local record initialization preserves firmware write order: zero record halfword `+24`, write input bits 3..6 into record byte `+24` bits 3..6, copy input halfword `+2` to record halfword `+26`, then copy argument-1 dword `+0` into record dword `+20`. Successful local record paths tail to `0x1F3E0(lookup, context)`. No semantic names are assigned to opaque runtime contracts.

## Stage 49 — current state transition coordinator

Stage 49 targets current `0x16E0D8`, the previously established relocation-normalized 1410-byte match of legacy `sub_16B10C`. The exact current body SHA-256 is `20b80f21aa042f817b71455fc43e0d9f1eab0af5e070d947381ab7a2f2634714`; the prior normalized fingerprint is `beff71fcd38e1f4cd2adc1f8400db59447d0622acf36daf4696423a0c250530c`. The prior candidate map also established that all 40 masked external call sites relocate coherently. Stage 49 deliberately leaves those runtime contracts opaque except for already-observable argument/return and pointer-mutation behavior.

The entry conditionally clears bytes `+0x97`, `+0x115`, and `+0x116` when argument 1 is one, then passes the state and argument through current `0x3A604`. A nonzero result enters a byte-`+0x0E` chain through `0x17E2C`, `0x17820`, and `0x390E4`; a zero predicate returns the current `0x202E8` result. A second `0x25288` precheck has the same final return, with optional diagnostic `0x1D104(0x32, 0x18)`. The equal-argument path clears bytes `+0x94/+0x95`, calls `0x25320`, clears ambient bit 0 at `0x3186D0` when byte `+0x5E` is nonzero, and copies word `+0x78` into word `+0x0C` when dword `+0x68` is nonzero.

On the main path, current `0x2521C` and ambient byte `0x208338+19` control state byte `+0x98` bit 7; byte `0x209B98` can then override that bit. The routine calls `0x6301C(state)` and, when ambient bit 4 is clear, `0x4D57C(state+0x90)`. It snapshots state halfword `+0x98` into the low half of the pushed argument-1 scratch cell. The outer split is exact: zero dword `+0xA0`, clear byte-`+0x90` bit 7, or nonzero byte `+0x135` selects the common path. Otherwise `0x21FC2` can bypass the secondary gate only when byte `+0x0F` is one, ambient byte `0x208B78+59` is zero, ambient pointer `0x208B78` is *different* from the state pointer, and byte `+0x9E` is zero. All other cases use `0x6304C`; nonzero selects common and zero selects the alternate transition path.

The common path rewrites two packed fields in the saved argument scratch cell and updates byte `+0x11B`. Role one clears both fields and can, after a second `0x21FC2`, set byte `+0x124` and ambient pointer `0x208B7C` under the observed pointer/mask/mode gates. Other roles either clear the fields after the `0x29778`/word-`+0x104` test or set both fields to one. The resulting low halfword replaces only the low half of the pushed state-pointer scratch cell, is zero-extended into ambient dword `0x318ACC`, and drives byte `+0x9C`, optional `0x4D4DC`, and optional `0x1D104` notification before the shared tail.

The alternate path handles bytes `+0x9F`, `+0x9E`, `+0x134`, and `+0x129`, including the observed byte-`+0x99` bit-1 toggle and `0x1EFA8(byteA4)`. The mode table at `0x20289E` and threshold dword `0x202854` gate an optional `0x367CC(state)` early return after `0x21FC2`; byte-A4 values `0x18..0x1A` skip that early call. The path then applies optional `0xAF094`/`0x624E8`, sets bytes `+0x9E/+0x115/+0x125`, updates ambient mask `0x208BB8` from state dword `+0xF8` when `0x202FA8` bit 16 is set, and re-reads state words `+0x98/+0x9A` into scratch/ambient snapshots before `0x253B0(state+0x90)`.

Current `0x335AC(byteA4)` and `0x38918(context.word64)` feed a matrix update at `0x208C9C + class*40 + index*20`. For the observed mode predicates, byte `+18` is shifted to `+19` and byte `+18` becomes one or zero. The function then re-reads the mode, maps it through `0x20289E` into byte `+0x9C`, and stores `(byte9C - 1)` modulo 16 bits into word `+0x0C`. For role zero, another `0x21FC2` plus byte `+0x121` and `0x3C7C2` choose the exact ambient byte pairs written to `+0x11E/+0x11F`.

The later diagnostic/feature section preserves the firmware's re-read ordering around opaque calls: optional `0x1D104` notifications pack words `+0x98/+0x9A` or payload bytes behind dword `+0xA0`; ambient byte `0x207BA8` gates `0x2C79E`, optional `0x335AC`/`0x2C78C`, and a possible write of one to `0x207BA5`. The shared tail can copy `0x20285B` into byte `+0x131`, call `0x629D0`, call `0x1EBA0(byteA5, scratch_state)`, run `0x6329C(state)`, update ambient `0x3186D0` bit 0 from post-call word `+0x9A`, optionally call `0x2CAA8(state, 1)`, and finally return current `0x3A742(4, state, &scratch_arg1)`.

Two compiler stack scratch cells are modeled intentionally. Only the low halfword of the pushed state-pointer cell is overwritten, so the `0x1EBA0` argument retains the original pointer's high 16 bits. The pushed argument-1 cell likewise retains its original high 16 bits while its low half carries the local packed state and is finally passed by address to `0x3A742`. The stack-canary load/check and `0x94C0` failure call are compiler-generated hardening and are omitted from the semantic Rust model. No semantic names are assigned to unresolved runtime boundaries.

## Stage 50 — current lookup-slot maintenance and promotion

Stage 50 targets current `0x16DDAC`, the previously established relocation-normalized 236-byte match of legacy `sub_16ADE0`. The exact current body SHA-256 is `b86f7aff5e1ff3fc3197634715de77ccfdeb4e730cf48d2bd3d6d7cca708f63f`; the retained normalized fingerprint is `13bb761283b42e7a03ca34739eb2d7bca1ff02011f82ebef78585da77fe64d31`. The current body directly reaches opaque boundaries `0x1EE18`, `0x1F270`, `0x780`, and the already-known release boundary `0xB0460`. The literal immediately after the body at `0x16DE98` independently resolves to ambient address `0x208338`.

The firmware loads a lookup selector through the caller's pointer chain (`*(*(arg0 + 4))`) and passes it to `0x1EE18`. If lookup dword `+0x10` is already nonzero, the function immediately returns the lookup handle. Otherwise it dispatches on lookup byte `+0x0B` bits 2..5. Modes above three skip all mode bodies.

Mode 0 selects lookup dword `+0x20`; mode 1 selects dword `+0x14`. When the selected dword is nonzero, current `0x1F270(lookup)` runs and its result is retained as the local test value and live return value. A zero selected dword leaves the lookup handle as the live return value.

Mode 2 selects dword `+0x2C`. When nonzero, it calls `0x1F270(lookup)`, then brackets the following work with two opaque `0x780` calls. If the `0x1F270` result is not one, firmware calls `0xB0460` unconditionally on dword `+0x20` and dword `+0x2C`—including zero values—and clears both slots. The second `0x780` result becomes the live return value. This unconditional release behavior is preserved rather than normalized into null-checked cleanup.

Mode 3 brackets a full slot cleanup with `0x780`: dwords `+0x14`, `+0x20`, `+0x2C`, and `+0x10` are read in that order, each nonzero handle is released through `0xB0460`, and that slot is cleared. The second `0x780` result remains live after the cleanup. The four table targets therefore differ materially: mode 0 -> `+0x20`, mode 1 -> `+0x14`, mode 2 -> `+0x2C` plus conditional cleanup, and mode 3 -> full cleanup.

After dispatch, lookup dword `+0x14` is re-read. If nonzero and its pointed object's byte `+2` has low two bits clear, ambient byte `0x208338 + 0x13` bit 3 must be set to continue; otherwise the current live return value is returned. If the local `0x1F270` result is not one, the function also returns without promotion.

The promotion path is ordered. It calls `0x780(1)`, reads old dword `+0x20`, then re-reads dword `+0x14`, copies `+0x14` into primary dword `+0x10`, and clears `+0x14`. A nonzero old `+0x20` is then released and cleared; dword `+0x2C` is deliberately re-read only after that release, then conditionally released and cleared. The function tail-calls the second `0x780` with the token returned by the first call, and that boundary result is the final return. Current `0x1F270` and `0x780` remain opaque; only their observed arguments, return flow, and surrounding state ordering are modeled.

## Stage 51 — current record/state coordinator

Stage 51 reconstructs current `0x16DBDC`, the retained relocation-normalized match of legacy `sub_16AC10` (326 bytes). The exact current body SHA-256 is `5918418add251df8ca6ddf078bdf025632ccb634697829997a39a444c2838c1c`; the retained normalized candidate hash is `447cb38addf0223c087cb4f0cfa8949bc3296982cb623a4610175593655ff8de`. Direct current targets are `0x335AC`, `0x3A6CC`, `0x3B04A`, and `0x3D24`; their broader runtime meanings remain opaque.

The routine first calls `0x335AC(object.byteA4)` and exits immediately if it returns zero. When object byte `+0x94` equals 2, it ORs bit 1 into stats byte `+0x14` if object byte `+0x90` has bit 7 clear, and increments stats byte `+0x16` with u8 wrap when `(object.byte90 >> 3) & 0x0f <= 2`.

For nonzero event byte `+6`, record-side accounting is gated by event mode `((byte0 >> 3) & 0x0f) > 2` and `(byte2 & 3)` equal to 1 or 2. The record base is `stats + 25 * stats.byte1`, with no local bounds check. If object byte `+0x94` is 2 and object byte `+0x91` bit 0 is clear, record byte `+0x22` increments. If that bit is set instead, the low byte of zero-argument boundary `0x3A6CC()` goes to record byte `+0x28` and object byte `+0x113` goes to record byte `+0x29`. When object byte `+0x94` is not 2, record byte `+0x21` increments.

Still on the nonzero-event path, object byte `+0x0f == 0` together with stats byte `+0 != 0` adds event byte `+4` and object byte `+0x96` into record halfword `+0x26` with 16-bit wrap, then clears stats byte `+0`. On the zero-event path, object byte `+0x0f == 1` and object byte `+0x94 != 2` increment record byte `+0x23`; if stats byte `+0` is nonzero, record halfword `+0x26` additionally gains 2 and stats byte `+0` becomes zero.

The late path runs only when object mode `((byte90 >> 3) & 0x0f) <= 1`. A nonzero stats byte `+0x0f` is cleared before current `0x3B04A(stats)`, and later fields are re-read after that opaque call. If the post-call stats byte `+0x0e` is nonzero, firmware clears it, calls current `0x3D24(stats + 16, 0, 0x74)`, then forces stats byte `+1` to zero. Otherwise it reads stats byte `+0x14`: if bit 0 is clear and stats byte `+1` is below `byte20 >> 5`, the index increments by one. Because return-register contents are incidental and path-dependent, the Rust model exposes this routine as effect-only rather than assigning a semantic return value.

## Stage 52 — current lookup/range gate

Stage 52 reconstructs current `0x16EBA4`, a compact 64-byte function immediately following the already mapped Stage-49 region. A 73136-byte public legacy BCM4362A2 image has the same body shape at `0x16BBD8`; after masking only six relocation bytes belonging to the three direct calls, the 64-byte bodies are byte-identical, with normalized SHA-256 `2f7f8f883fc65db016ee47a53f644947900bc9d8fa8468c10d6f9e2f857e9c9e`. Scanning the current executable range with the remaining 58 fixed bytes yields exactly one hit at `0x16EBA4`. Because the available public legacy HCD is not the project's canonical legacy blob, it is used only as a structural relocation cross-check; current semantics are derived from the exact Orange Pi current HCD.

The exact current body SHA-256 is `eb6364d7f626b41ac2b70b26bb48e51022b9a2e559157f21e596e49bbc6081cd`. Its direct current boundaries are `0x338FC`, `0x4D552`, and `0x18540`; their broader runtime meanings remain opaque. Two post-body literals resolve to ambient dword addresses `0x221EDC` and `0x221EE0`.

Locally, the routine calls `0x338FC(input.byteA4)` and returns zero immediately if that result is zero. Otherwise it reads dword `+0` from the returned object. A zero dword skips the second and third boundaries and uses zero as the local candidate value. For a nonzero dword, the firmware re-reads input byte `+0xA4`, calls `0x4D552`, then loads dword `+0x0C` from the record, clears its top nibble with `& 0x0FFFFFFF`, and passes the second-boundary result plus that masked dword to `0x18540`.

The resulting value is compared as an unsigned integer against two ambient bounds. It must be strictly greater than `*0x221EDC`; otherwise the routine returns zero without reading the upper bound. Only after passing that test does it read `*0x221EE0`, returning one exactly when the value is strictly less than that upper bound. The source model therefore preserves an open interval `(lower, upper)`, the lower-bound short circuit, the second input-byte read, and the exact three-boundary call order without assigning semantic names to the opaque runtime contracts.

## Stage 53 — compact current post-gate object sequence

Stage 53 reconstructs current `0x16EEA4`, a 58-byte function in the adjacent post-Stage-49 region. A public 73136-byte BCM4362A2 legacy image has the same body shape at `0x16BED8`; masking only the eight relocation bytes belonging to the five direct calls leaves 50 fixed bytes, and those normalized bodies are byte-identical. The fixed-byte scan has exactly one current hit at `0x16EEA4`. The public legacy HCD is not the project’s canonical legacy blob, so it is used only as a structural relocation cross-check; the local semantics below are derived from the exact current Orange Pi HCD.

The exact current body SHA-256 is `9ce069266cda1ab403c9849c0cb256665640e6858376be3349620b243bf9be0a`; its normalized structural SHA-256 is `49203391a9e054c5d8e55b8e1bf11c46ded353dd43b6aebb9c3cd8985fa811bb`. Direct current calls are `0x21F20`, `0x24824`, `0x5180C`, internal current `0x16F5F4`, and `0x50B76`. Two PC-relative literal loads resolve to stable ambient byte addresses `0x20A234` and `0x20A223`; the same literal values occur at the structurally corresponding legacy sites.

The routine receives an object pointer in R0 and preserves it in R4. It first calls opaque `0x21F20(object)` and immediately returns zero when that result is zero. On the nonzero path, the object fields used by the next call are loaded only after the gate returns: R0 receives dword `+0x48`, R1 receives dword `+0x1C`, and R2 is literal one before opaque `0x24824`. This post-gate read ordering is preserved so gate-side object mutations remain observable.

Next, the routine clears byte `*0x20A234` to zero, then calls `0x5180C(object)`, current internal `0x16F5F4(object)`, and `0x50B76(object)` in that exact order. The return from `0x50B76` remains live in R0 through all remaining local stores and is the function’s final return on this path. The firmware then forces object byte `+0x5F` to one, re-reads object byte `+0x14` only after all three object-pointer calls, and writes that post-call value to ambient byte `*0x20A223`. The source model keeps all five runtime contracts opaque while preserving these local ordering, reread, global-store, and return-flow properties.

## Stage 54 — compact current two-boundary wrapper

Stage 54 reconstructs current `0x16F228`, an exact 20-byte wrapper. A public 73136-byte legacy BCM4362A2 image has the same body shape at `0x16C25C`. Masking the four relocation bytes belonging to its ordinary call and final wide tail branch leaves 16 fixed bytes; the normalized bodies are byte-identical, and the fixed-byte scan yields exactly one current hit at `0x16F228`. As in Stages 52–53, the public legacy HCD is structural cross-check material only; current semantics are derived from the exact Orange Pi current HCD.

The exact current body SHA-256 is `c9746dcab7a28858cebab9a5bb4ab08bd50bba18a2a573363015aa6829d076e1`; normalized structural SHA-256 is `b4d587e08025f4fd93ed6d654e7e5ffa741df97c1bdebe4563ef37b21a0dd8c7`. The ordinary current call target is `0x44468`; the final `B.W` relocates to current `0x265E8`. The structurally corresponding legacy body reaches the same two runtime addresses.

Locally, firmware preserves the incoming opaque object token, sets R1 to literal one, and calls `0x44468(object, 1)`. The call's return is ignored. It then restores the saved frame, restores the same object token to R0, and tail-branches to `0x265E8(object)`. There is no local null/range check and no other local state mutation. The tail boundary's return is therefore the wrapper's final return. The source model keeps both runtime contracts opaque and preserves exactly this two-call forwarding behavior.

## Stage 55 — compact current masked-record scan

Stage 55 reconstructs current `0x16F306`, an exact 40-byte leaf routine. A public 73136-byte BCM4362A2 legacy image contains a structurally corresponding body at `0x16C33A`, and the two 40-byte bodies are byte-identical with no relocation differences. Their exact SHA-256 is `f2bf19b9cea9f572655abc2c81fa124d4d76f5f06b92a3490cf4a15ba72ae932`; an exact-body scan yields exactly one current hit at `0x16F306`. The public legacy HCD remains structural cross-check material only; the semantics below are derived from the exact current Orange Pi HCD.

The routine has no direct `BL`/`B.W` runtime calls. Its initial PC-relative literal load at `0x16F308` resolves through literal `0x16F334` to stable table base `0x20A2D4`. Incoming R3 is used first as a pointer: the routine loads one dword from `[R3]`, then overwrites R3 with scan index zero. That single dword is therefore a snapshot used for the entire scan.

The scan index takes exactly the values zero, one, and two. For each index, firmware computes `1 << index` and tests that bit against the saved mask. A clear bit skips the record entirely. For a set bit, firmware computes `record = 0x20A2D4 + 0x84 * index`, reads the halfword at `record + 0x22`, and compares it with literal six. The first selected record whose status halfword equals six is returned immediately as the record pointer. If no selected record matches after index two, the routine returns zero. Bits above bit two are never inspected, and the source model preserves the single mask load, ascending scan order, selective memory reads, exact `0x84` stride, `+0x22` halfword offset, first-match return, and zero fallback without inventing broader semantics for the table.

## Stage 56 — current gated bit-22 update

Stage 56 reconstructs current `0x16F438`, an exact 98-byte routine used by a nearby later wrapper. The public 73136-byte BCM4362A2 legacy image has the same body shape at `0x16C46C`. The only body differences are two bytes inside the relocation encoding of the single direct call: masking those bytes leaves 96 fixed bytes, the normalized bodies are byte-identical, and the fixed-byte scan yields exactly one current hit at `0x16F438`. The exact current body SHA-256 is `2d0b25b41c913eb1f2c10076a5b82e6fe74e4c549ba5886bf47ae254c57024df`; normalized structural SHA-256 is `7d3da5cec31b73cc6e7dab2f7c2ece63df93c42f8d5f4ef88714cabca966de65`. Public legacy remains structural cross-check material only.

The routine saves incoming R0 as the object pointer and incoming R1 as a mode value, sets R1 to literal two, and calls opaque current `0x21DE8(object, 2)`. That call happens before every object-field read. Its R0 return remains live for the entire function: it is used later in a signed comparison and is also the function's final return on every path, including early exits. A null object returns immediately after the call. Otherwise the gate reads object byte `+0x0F`; when that byte is nonzero, mode must equal one. When byte `+0x0F` is zero, object byte `+0x34` is read: a nonzero value bypasses the mode check and becomes an additive seed, while a zero value again requires mode one and uses seed zero.

On the update path, a PC-relative literal at current `0x16F49C` points to ambient byte triplet base `0x221F1D`; the structurally corresponding public legacy literal points to its older relocated address `0x221EE9`. Firmware reads triplet bytes `+2` and `+1` and proceeds with the threshold path only when byte `+2` is strictly below byte `+1`. It then re-reads object byte `+0x34`, reads object byte `+0x2E`, sums them, reads triplet byte `+0`, and computes `threshold = triplet[0] * (byte52 + byte46) + seed`. The opaque probe result is compared with this threshold using signed `CMP`/`BGE`; only signed `probe < threshold` reaches the final predicate.

A second current literal at `0x16F4A0` points to ambient byte `0x221F1C`. If that byte is zero, the new predicate bit is one. If it is nonzero, firmware reads object halfword `+0x22` and sets the predicate bit to one exactly when that halfword is not equal to six. All failed update predicates produce bit zero. Finally current literal `0x16F4A4` points to dword `0x209644`: the routine reads that dword, clears only bit 22 (`0x00400000`), inserts the computed boolean into bit 22, and writes the dword back. Early null/mode exits occur before this read-modify-write, so they leave the ambient dword untouched. The source model preserves these exact ordering, signed-comparison, short-circuit, and bit-preservation properties while keeping the `0x21DE8` contract opaque.

## Stage 57 — current gated counter/copy wrapper

Stage 57 reconstructs current `0x16F59E`, an exact 64-byte routine whose public 73136-byte BCM4362A2 structural counterpart at `0x16C5D2` is byte-identical. The exact body SHA-256 is `62914519accb7aeef6ac538f926aa4ceb9a771694c87fefd18ad6045eb431c56`, and an exact-body scan yields exactly one current hit at `0x16F59E`. Public legacy remains structural cross-check material only. The routine has one direct call, current `0x16F5CE -> 0x16F438`, which is the Stage-56 function and therefore makes Stage 57 integration dependent on Stage 56 integration.

The first gate is incoming R2. When R2 is zero, the routine returns immediately with incoming R0 unchanged and performs no ambient access. Otherwise it tests incoming R3 bit zero. A clear bit forces current ambient byte `*(0x221F1D + 2)` to zero. With bit zero set, firmware loads object byte `+0x6E` using signed `LDRSB`, loads object byte `+0x6C` unsigned, and increments the old ambient byte only when signed byte110 is strictly greater than byte108. The increment is an 8-bit `ADDS` followed by `STRB`, so `0xFF` wraps to zero. Every other path stores zero. The structurally corresponding public-legacy literal points at the older relocated triplet base `0x221EE9`.

After the byte store, firmware uses current literal `0x209644` as a source block. It reads the halfword at source `+2`, masks it with `0x3F`, and proceeds only when the low six bits equal `0x19`. Failure returns incoming R0 unchanged. Success sets R1 to one and calls Stage-56 `0x16F438(object, 1)`. Its R0 return becomes the Stage-57 final return. Only after that call does firmware read dwords `*0x209644` and `*0x209648` and write them, in order, to `0x650160` and `0x650164`. This ordering is material because Stage 56 may modify bit 22 of the first source dword; Stage 57 therefore copies the post-Stage56 value, not a pre-call snapshot.

## Stage 58 — current ambient triplet-byte clear

Stage 58 reconstructs current `0x16F5F4`, the internal boundary that Stage 53 previously kept opaque. It is an exact 8-byte leaf body, `01 4B 00 22 9A 70 70 47`, with no runtime calls. The public 73136-byte BCM4362A2 structural counterpart at `0x16C628` has the identical 8-byte body, and an exact-body scan yields one current hit at `0x16F5F4`. Public legacy remains structural cross-check material only.

The function loads one PC-relative literal immediately following the body. Current literal `0x16F5FC` contains `0x221F1D`; the structurally corresponding public-legacy literal `0x16C630` contains older relocated address `0x221EE9`. It then places literal zero in R2, stores that byte to offset `+2` from the loaded base, and returns with `BX LR`. The source model therefore exposes only the exact local effect: one zero byte store to current ambient address `0x221F1F`, with no invented runtime semantics.

## Stage 59 — current ambient-byte copy and callback publication

Stage 59 reconstructs current `0x16F600`, an exact 30-byte leaf body. The public 73136-byte BCM4362A2 structural counterpart at `0x16C634` is byte-identical, and an exact-body scan yields exactly one current hit at `0x16F600`. Public legacy remains structural cross-check material only. The current body contains no direct `BL`/`B.W` calls; its behavior is driven entirely by seven PC-relative literals following the body.

Current literals resolve to source byte `0x20A22A`, destination byte `0x222709`, object-pointer cell `0x202A74`, global block `0x2167D4`, and raw Thumb pointers `0x16F511`, `0x16F4A9`, and `0x16F33D`. The public-legacy structural body uses the same source byte, object cell, and global block, but its destination byte is older relocated `0x222585` and its three raw Thumb pointers are older relocated `0x16C545`, `0x16C4DD`, and `0x16C371`.

Locally, the function first copies one byte from `*0x20A22A` to `*0x222709`. It then loads `*0x202A74`. A zero value returns immediately after the byte copy. For a nonzero object token, firmware stores raw Thumb pointer `0x16F511` to global-block offset `+0x54` (`0x216828`), stores raw Thumb pointer `0x16F4A9` to object offset `+0x14`, and stores raw Thumb pointer `0x16F33D` to global-block offset `+0x5C` (`0x216830`), in that order. R0 is not modified anywhere in the body; the source model preserves that incoming-register value while keeping the wider meanings of the object/global slots and callback targets opaque.

## Stage 60 — current gated ten-byte saturating update

Stage 60 reconstructs current `0x16F8D0`, an exact 72-byte leaf routine. Searching the public 73136-byte BCM4362A2 image without assuming the earlier relocation delta finds exactly one byte-identical structural counterpart, at legacy `0x16C670`. The current body itself also has exactly one hit in the current executable range. There are no direct runtime calls in the body. Public legacy remains structural cross-check material only.

Four current literals following the body resolve to gate byte `0x222747`, selector block `0x20DCD2`, source-byte base `0x22270B`, and target block base `0x20DCD8`. The corresponding public-legacy literals are `0x2225C2`, `0x20DCD2`, `0x222586`, and `0x20DCD8`: the two block bases are stable while the gate/source byte regions moved. Current firmware first reads `*0x222747` and returns immediately when it is zero. Otherwise it reads selector byte `*(0x20DCD2 + 3)`.

The selector is multiplied by ten. Firmware forms `source = 0x22270B + selector * 10`. Source byte zero selects the arithmetic direction: a nonzero byte means subtract; zero means add. The loop then runs exactly ten iterations with source offsets one through ten. This is intentionally an overlapping window shape: the source pointer advances by `selector * 10`, not by eleven, while the body consumes byte zero plus offsets one through ten. The source model preserves that arithmetic exactly rather than normalizing it to a non-overlapping record stride.

Targets are ten bytes beginning at `0x20DCD8 + 0x159 = 0x20DE31`, with exact stride `0x24`. In subtract mode each target becomes `max(0, old - delta)`, implemented by subtraction followed by sign-mask clearing. In add mode each target becomes `min(255, old + delta)`, implemented by addition and signed compare against `0xFF`. Each result is stored before advancing to the next target. R0 is untouched on the early-gate path; on the update path it is overwritten with the computed source-window pointer and remains that value at return. The source model preserves this observable return-register shape along with all ten writes and saturation boundaries.

## Stage 61 — current indirect dual-sample clamp

Stage 61 is a current-only reconstruction of exact 60-byte body `0x16F928`. The body has exactly one hit in the current executable range. No exact or 16/24-byte-prefix structural counterpart was found in the available public legacy HCD, so no legacy address is asserted. There are no direct `BL`/`B.W` calls, but the body performs two indirect `BLX` calls through a runtime function pointer.

The current literal pool contains gate byte `0x22274A`, dispatch-pointer cell `0x20375C`, and offset-byte base `0x20D9A2`; the body reads offset byte `+2`, i.e. `0x20D9A4`. A zero initial gate returns immediately. Otherwise firmware freshly loads `*0x20375C`, then the method dword at `+0x5C`, calls it with selector one, re-reads offset byte `0x20D9A4`, adds that byte to the returned R0, and reads a sample byte at resulting pointer `+0x15`.

The firmware then freshly reloads the same dispatch pointer/method and calls it with selector zero. It again re-reads `0x20D9A4` and forms the second sample pointer. Only after both calls does it re-read gate byte `0x22274A`, then read the second sample byte. It computes the unsigned absolute difference between the first and second bytes. The first sample is written over the second sample only when the re-read gate is strictly less than that difference; equality does not write. On the nonzero-gate path R0 remains the second computed base pointer (`selector-zero return + post-call offset`) at function return. The source model preserves both offset re-reads, the gate re-read, indirect call order, strict comparison, and return-register shape while keeping the indirect method contract opaque.

## Stage 62 — current three-selector record-byte initializer

Stage 62 reconstructs current `0x17022C`, an exact 70-byte leaf routine. Its body has exactly one hit in the current executable range. Searching the public 73136-byte BCM4362A2 image finds exactly one byte-identical structural counterpart at `0x16CD34`. There are no direct or indirect runtime calls in the body; public legacy remains structural cross-check material only.

The current post-body literals are selector-table address `0x222750` and record-base pointer cell `0x20918C`. The structural legacy body uses older relocated selector table `0x2225C3` and the same pointer cell `0x20918C`. Firmware processes exactly three selector bytes, at table offsets zero, one, and two. A selector equal to `0xFF` is skipped completely.

For every non-`0xFF` selector, firmware freshly reloads dword `*0x20918C`; this is not hoisted outside the three-selector sequence. The selector arithmetic is `7 * selector`, then doubled while adding the freshly loaded base, producing exact record stride 14. Firmware stores literal byte eight at `base + 14 * selector + 11`. The three selector positions are handled in table order, and each live selector can therefore observe a different base-pointer value if ambient state changes between loads. R0 is not modified anywhere in the body, so the source model preserves the incoming R0 token as its observable return shape.

## Stage 63 — current optional-boundary sequence

Stage 63 reconstructs current `0x1704D4`, an exact 34-byte wrapper. The body contains three direct `BL` instructions. Masking only those 12 relocation bytes leaves 22 fixed bytes; the normalized pattern has exactly one current hit at `0x1704D4` and one public-legacy structural hit at `0x16CFDC`. Current calls are `0x163668`, `0x3D24`, and `0x171A7C`; the legacy structural body calls older relocated `0x162F4C`, stable `0x3D24`, and older relocated `0x16DC50`. Public legacy remains structural cross-check material only.

The current literal following the body points to context-pointer cell `0x22257C`; the legacy structural literal points to older relocated `0x2224CC`. Firmware loads this cell once into R4 and saves incoming R0 separately in R5. If the loaded context is nonzero, it first calls opaque `0x163668` with the incoming R0, then calls opaque `0x3D24(context, 0, first_return)`. The second call's R0 return becomes the current R0 value.

The final call is gated by the saved original input, not by the context and not by either boundary return. When original input is nonzero, firmware calls opaque `0x171A7C` with whatever R0 is current at that point: the untouched original input when context was zero, or the `0x3D24` return when context was nonzero. The final call's return is ignored. Firmware then forces R0 to zero and returns. The source model preserves this exact argument/order/forwarding behavior without assigning broader semantics to the three boundaries.

## Stage 64 — current first-exact-one index scan

Stage 64 reconstructs current `0x1720C0`, an exact 24-byte wrapper. It contains one direct `BL` at `0x1720C6`. Masking only that four-byte relocation leaves 20 fixed bytes; the normalized pattern yields exactly one current structural hit at `0x1720C0` and exactly one public-legacy structural counterpart at `0x16E010`. The current call targets `0x172044`; the legacy structural body calls older relocated `0x16DF94`. Public legacy remains structural cross-check material only.

The function initializes its scan index to zero and overwrites incoming R0 with the zero-extended low byte of that index before every call. The opaque current boundary `0x172044(index)` is invoked for indices zero through seven in ascending order. The wrapper stops only when the boundary return is exactly literal one; other nonzero values do not match.

When a call returns one, firmware returns the current index immediately. Otherwise it increments the index and continues while the index is not eight. If none of the eight calls returns exactly one, the wrapper returns literal eight. The source model preserves the exact call order, equality-to-one predicate, early return, ignored incoming R0, and sentinel-eight fallback without assigning broader meaning to the opaque boundary.

## Stage 65 — current zero-fallback mode wrapper

Stage 65 reconstructs current `0x1724C8`, an exact 28-byte wrapper. It has two direct control transfers to the same current boundary `0x172458`: an ordinary `BL` at `0x1724D0` and a final `B.W` at `0x1724DE`. Masking only those two four-byte encodings leaves 20 fixed bytes; the normalized pattern yields exactly one current structural hit at `0x1724C8` and exactly one public-legacy structural counterpart at `0x16E3B8`, whose corresponding boundary is older relocated `0x16E348`. Public legacy remains structural cross-check material only.

Firmware saves the incoming R0 input, copies it into R1, sets R0 to literal zero, and calls `0x172458(0, input)`. A `CBNZ` tests that call return directly. Any nonzero value branches to the ordinary epilogue and is returned unchanged.

Only an exact zero return reaches the fallback path. Firmware restores the saved input to R1, sets R0 to literal one, restores the frame, and tail-branches to the same boundary as `0x172458(1, input)`. The tail boundary return is therefore the wrapper's final return, including zero. The source model preserves the exact nonzero gate, input forwarding, two mode values, and tail-result semantics while keeping the shared boundary opaque.
