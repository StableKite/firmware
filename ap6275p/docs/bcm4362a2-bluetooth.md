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
