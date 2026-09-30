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

## Stage 66 — current bounded tail-copy wrapper

Stage 66 reconstructs current `0x1724E4`, an exact 46-byte wrapper. It contains one ordinary `BL` at `0x1724F2` and a final wide branch at `0x17250E`. Masking only those two four-byte control-transfer encodings leaves 38 fixed bytes; that normalized pattern has exactly one current hit at `0x1724E4` and one public-legacy structural hit at `0x16E3D4`. The exact current body SHA-256 is `5eeac25cb5add61ed45a10a4b71e7080b760cf01080ce01ca4251616d0bd2c46`; the public-legacy exact body SHA-256 is `dbe4a95fdfbd12166822ebe7a2b8c4e9c64e65dfcdd436d87f23fde2af03eb48`; normalized structural SHA-256 is `56d894ed1cf9c515b746b6df797e76f6bbafe453303a734d642a0907dbae1392`. Public legacy remains structural cross-check material only.

The current literal at `0x172514` contains buffer base `0x22300E`; the structurally corresponding public-legacy literal at `0x16E404` contains older relocated base `0x222E7A`. Firmware saves incoming R0 as an unsigned length-like value and incoming R1 as an opaque source token. It then calls current opaque boundary `0x3D24(0x22300E, 0, 59)`. The boundary return is ignored locally, while both saved incoming values survive the call.

After the first boundary returns, firmware restores R0 to the buffer base, logically shifts the saved incoming R0 right by one, stores the low byte of that result at `0x22300E`, and post-increments R0 to `0x22300F`. It then compares the original 32-bit input with literal `0x39` using unsigned `LS/HI` conditions. Inputs at or below 57 are forwarded unchanged as the final R2 length; inputs above 57 use literal 58. Thus the tail length is exactly `min(input, 58)` under unsigned comparison semantics.

Firmware restores its frame and tail-transfers to stable opaque boundary `0x3DB4(0x22300F, original_R1, clamped_length)`. The public-legacy structural body reaches the same two runtime boundaries, `0x3D24` and `0x3DB4`. The tail boundary return is the wrapper's final return. The source model preserves call/store ordering, the logical-shift/truncation behavior, unsigned clamp boundary, original-R1 forwarding, ignored first-boundary return, and final tail-result forwarding without assigning broader semantics to either runtime boundary.

## Stage 67 — current five-way state dispatch wrapper

Stage 67 reconstructs current `0x172518`, an exact 98-byte routine containing a `TBB` state dispatch and four direct calls. Masking only the four four-byte call encodings leaves 82 fixed bytes; the normalized pattern yields exactly one current hit at `0x172518` and one public-legacy structural hit at `0x16E408`. The exact current body SHA-256 is `53320368829851feb328f0312d0d5ef6acc92b94ec9bfe4db7294c307c8ac4cf`; the exact public-legacy body SHA-256 is `6e575295da77f7423d6f7dad2e7470243f74183d40b0b9be8518634d7922e889`; normalized structural SHA-256 is `f35f85dd3773b12fa14ed187d8c86996d4564f00f3a297a0cf9a637e3d9d20ab`. Public legacy remains structural cross-check material only.

The current post-body literal pool resolves state byte `0x223064`, source byte `0x222084`, context-word cell `0x22208C`, destination byte `0x223065`, raw Thumb callback pointer `0x171FF9`, and block address `0x22304C`. The structural legacy pool uses older relocated values `0x222ED0`, `0x221FD8`, `0x221FE0`, `0x222ED1`, `0x16DF49`, and `0x222EB8`. The four current direct targets are stable runtime boundaries `0x72B24`, `0x151FE`, `0x151BC`, and `0x15180`; the structural legacy body reaches those same targets.

Firmware reads the state byte once for dispatch, rejects values above four from the `TBB`, and uses exact table entries `[3, 0x15, 0x22, 0x22, 0x22]`. State zero calls opaque `0x72B24` with R0 forced to zero while incoming R1 and R2 remain live and R3 contains state zero. It then writes state one, copies byte `*0x222084` to `*0x223065`, reads `*0x22208C`, and calls opaque `0x151FE(0x22304C, 0x171FF9, 0, context)`. After that call it re-reads `*0x22208C`, loads block `0x22304C`, and calls opaque `0x15180(block, reread_context)`. Boundary return values are ignored locally.

State one copies the same source byte to `0x223065`, then calls opaque `0x151BC` with R0=`0x22304C`, R1 equal to original incoming R1, R2 equal to the copied byte, and R3 still holding destination address `0x223065`. It then reads `*0x22208C` and calls `0x15180(0x22304C, context)`. States two, three, and four do not call any boundary; all three write literal five to the state byte. States above four skip all dispatch work.

After every path, firmware re-reads `*0x223064`; this is a post-call/post-write read, not the initial dispatch snapshot. The final `CMP #2` plus conditional moves returns literal three when the re-read unsigned state is at least two, otherwise literal zero. The source model therefore allows opaque boundaries to mutate ambient state/context and preserves the two context reads on state zero, the final state re-read, exact state-table routing, original-R1 forwarding on state one, byte-copy ordering, and ignored boundary returns without assigning broader semantics to the opaque runtime functions.

## Stage 68 — current conditional context-registration leaf

Stage 68 reconstructs current `0x1726D0`, an exact 40-byte leaf in the post-command-dispatch region. This stage is intentionally current-HCD-first: masking the two four-byte direct-call encodings leaves 32 fixed bytes and yields exactly one current executable hit at `0x1726D0`, while the available public 73136-byte legacy HCD does not yield a valid normalized structural counterpart. No legacy address is promoted for this stage.

The exact current body SHA-256 is `7094b3b63524809b2ee62396f0b3f6c50d7abe909d7775480e0dc2ab4e0f0945`; normalized SHA-256 after zeroing the two direct-call encodings is `af8301b9362182feb73b2ce7e4e13c3910b72659ab407fae63b44628ff71c901`.

Three current literals follow the body: `0x1726F8 -> 0x223088`, `0x1726FC -> 0x172661`, and `0x172700 -> 0x223080`. The first is a context base, the second is a raw Thumb callback pointer into the immediately preceding current routine, and the third is a reset-byte address. The body first reads dword `[0x223088 + 8]` and tests bit 2 (`0x4`). If that bit is already set, it performs no runtime calls or reset write and returns the incoming R0 unchanged.

When bit 2 is clear, firmware preserves incoming R0 as the argument and calls current `0x151FE(0x223088, 0x172661, 0, argument)`. Its return is ignored. It then calls current `0x15180(0x223088, argument)`; this second return remains live. Only after that call does firmware write zero to byte `0x223080`, and the byte store does not replace R0, so the `0x15180` return is the function's final return. The source model preserves this early-return split, exact call order and arguments, ignored first return, post-finalize reset write, and final-return preservation while keeping both runtime contracts opaque.

## Stage 69 — current bounded callback/counter leaf

Stage 69 reconstructs current `0x172660`, the callback whose raw Thumb pointer `0x172661` is registered by Stage 68. The exact current body is 92 bytes. Masking its five four-byte direct control-transfer encodings leaves 72 fixed bytes and yields exactly one current executable hit at `0x172660`. The public 73136-byte legacy HCD does not provide a valid normalized structural counterpart, so this stage is current-HCD-first and does not promote a legacy address.

The exact current body SHA-256 is `8e8ea5a8218a32e27683a8f421cc120eb925951dee9d45366416b932e2f8efbc`; normalized current SHA-256 is `a4bbf8ff90fd2ebbdb6689c836f2ed37a9c927a356cc9765d0798b16fe24dac7`. Current literals resolve to source byte `0x202FD4`, counter byte `0x223080`, limit byte `0x223084`, control base `0x2067FD` (the body reads byte `+3`, current `0x206800`), and context base `0x223088`.

The routine reads the source byte and calls opaque current `0xBAC7C(source)`. Only the low byte of that return is retained. It snapshots the counter and limit, computes `next = counter + 1` with 8-bit wrap semantics, and writes `next` back before the branch. If `limit >= next`, the function converts the probe byte to the boolean `probe == 0` and tail-forwards that boolean to current `0x72D00`; the boundary return is final.

When `limit < next`, firmware writes zero to the counter, reads control byte `0x206800`, and extracts bit 1. If that bit equals the probe byte, it calls `0x72D00` with the inverted bit-1 value; that optional return is ignored. It then calls current `0x151BC(0x223088)`, ignores that return, and tail-forwards `(0x223088, 0, 24)` to the already identified clear/memset boundary `0x3D24`. The clear-boundary return is final. The source model preserves the pre-branch counter write, 8-bit wrap, low-byte probe truncation, optional-notify equality condition, exact ordering, and the two distinct tail-return paths.

## Stage 70 — current masked-state publish wrapper

Stage 70 reconstructs current `0x16F4AA`, an exact 88-byte routine. The public 73136-byte BCM4362A2 structural counterpart at `0x16C4DE` differs at only eight relocation bytes. Masking those bytes leaves 80 fixed bytes and yields exactly one current hit and exactly one public-legacy structural hit. Exact current SHA-256 is `b97da13be8c90afa3eed57b210195c02f0855470767aa60898514d98f35da885`; public-legacy raw SHA-256 is `46fdea73802eb008e080cf4bce05f99f2670c9991ae1c0d874680b8d68d0e0cb`; normalized structural SHA-256 is `19044ed2db7c79b9064b640dddc6c6680e435af9efaae0da086fc803f1006a96`. Public legacy remains structural cross-check material only.

The function receives an object token in R0, a mode in R1, and a pointer to an 8-byte local/state pair in R3. Before any boundary call, firmware replaces state word0 with its low 22 bits and clears bits 8 through 13 of state word4. When mode is exactly three, current `0x45624(object, &mode)` is called with a pointer to the saved stack copy of mode; its return is ignored, and the mode is re-read afterward. This mutation-visible re-read is material: if the post-call mode equals 25, the already recovered Stage-56 function at `0x16F438(object, 0)` is invoked and its return is ignored. Mode 25 supplied directly takes the same Stage-56 path without the normalizer call.

Firmware then always calls opaque current `0x44C00(object, post-normalize-mode)`. Its R0 return is kept live while the two masked state dwords are copied, in order, to `0x650160` and `0x650164`. Current literal `0x16F508` resolves to `0x650160`. A second current literal at `0x16F50C` resolves to base `0x206F78`; firmware reads byte `+0x17`, current address `0x206F8F`.

When that status byte is zero, the function returns the preserved `0x44C00` result. With a nonzero status byte, current `0x46A08` is called while R0 still contains that result. A zero predicate return becomes the function's final zero return. A nonzero predicate result instead causes literal four to be passed to current `0x46EB8(4)`, whose return becomes final. The source model preserves the pre-call masks, stack-mode mutation visibility, Stage-56 trigger, publish ordering, known R0 forwarding to the predicate boundary, and all three final-return shapes without assigning wider semantics to opaque runtime calls.

## Stage 71 — current published callback closure

Stage 71 reconstructs current `0x16F510`, the callback whose raw Thumb pointer `0x16F511` Stage 59 publishes into the current global callback table. The exact function body is 118 bytes. After masking the nine relocation bytes that differ across the direct-call encodings, the normalized body has exactly one current hit at `0x16F510` and one public-legacy structural hit at `0x16C544`. Public legacy remains structural cross-check material only.

The current literal pool resolves to guard word `0x200890`, halfword table base `0x202A90`, gate byte `0x222708`, and ambient dword `0x209644`; the legacy structural body uses the same guard/table/ambient addresses and older relocated gate byte `0x222584`. Direct current calls are `0x51800`, already recovered Stage 56 `0x16F438`, `0x45624`, selector-21 predicate `0x8917C`, and the established stack-guard failure sink `0x94C0`.

A material ordering edge occurs before dispatch: firmware unconditionally reads halfword `*(0x202A90 + selector * 2)` before checking whether incoming class R2 equals two. The selector is therefore not locally bounds-checked. The source model preserves this by exposing the indexed read through the backend instead of inventing a safe host array bound. Non-class-two calls then return zero.

For class two, selector six calls opaque `0x51800` with the entry register shape. Exact zero causes Stage 56 to run as `0x16F438(object, 0)`; a nonzero result skips Stage 56. Both paths then call `0x45624(object, &mut local_halfword)` and return literal one. Selector 20 requires gate byte `0x222708 == 1` and object byte `+0x2D == 0`; on success it ORs only bit 22 into dword `0x209644` and returns zero. Selector 21 first requires nonzero `0x8917C` return, then enters the same gate/object-byte/bit-22 update path. All other selectors return zero.

The binary snapshots `*0x200890` on entry and compares it again before return, calling `0x94C0` on mismatch. That compiler hardening shape is recorded in provenance constants consistently with earlier reconstructed guarded functions; it is not promoted into additional callback business semantics.

## Stage 72 — published callback-B multi-entry closure

Stage 72 closes raw callback-B Thumb pointer `0x16F4A9` that Stage 59 publishes into object slot `+0x14`. The callable entry begins at even address `0x16F4A8`. Its first and only wrapper-specific instruction is a PC-relative `LDR R3` whose literal resolves to stable ambient state-pair address `0x209644`; execution then falls directly into the already recovered Stage-70 shared entry at `0x16F4AA`.

The full callable region from `0x16F4A8` through the Stage-70 epilogue is 90 bytes. After masking the eight relocation bytes inherited from Stage 70, it has exactly one current hit at `0x16F4A8` and one public-legacy structural hit at `0x16C4DC`. Both entry literals resolve to the same state address `0x209644`. This proves a deliberate multi-entry shape rather than a separate duplicate implementation.

The source model therefore specializes Stage 70 with the ambient two-dword state beginning at `0x209644` and delegates directly to `bt_stage70_masked_state_publish`. It adds no new gate, no hidden mutation, and no return transformation. All Stage-70 ordering, mode normalization, Stage-56 trigger, publication, post-publish predicate, and final-return behavior remain authoritative.

## Stage 73 — published callback-C periodic maintenance closure

Stage 73 closes the third raw Thumb callback published by Stage 59: current pointer `0x16F33D`, callable even entry `0x16F33C`. The exact current body is 198 bytes and ends at the `POP {r3,r4,r5,pc}` at `0x16F400`; its 13-dword literal pool begins at `0x16F404`. Masking only the nine direct `BL` encodings leaves 162 fixed bytes, with exactly one current hit at `0x16F33C` and one public-legacy structural hit at `0x16C370`. The literal pool is stable across the two images except for the already-known relocated published byte `0x222709` versus legacy `0x222585`.

The function increments dword `0x209694` with 32-bit wrap and stores the new value before every later gate. Byte `0x20964C` controls the first periodic path: only nonzero divisor, counter above 40, and exact zero remainder reach `0x45DD4(0,3)` followed by `0x45F54`, with the first call's R0 forwarded directly into the second. A separate trigger byte at `0x209596` is active only when nonzero and the new counter equals three: firmware clears the trigger first, calls `0x451A0`, and, when byte `0x202A0B` is nonzero, calls `0x469C0`, converts its return to the exact-one boolean, and forwards that `0/1` to `0x46C48`.

Byte `0x20963B` is decremented when nonzero. The next periodic divisor is byte `0x202A83`; unlike the earlier divisor, firmware executes UDIV/MLS without a local zero check. The source model therefore exposes that architecture/runtime edge through an explicit `unchecked_remainder` backend method rather than inventing zero-divisor behavior. Exact zero remainder calls `0x458CC`.

When the new counter low nibble is zero, nonzero bytes `0x209711` and `0x209639` gate `0x45918`. Independently, firmware loads the pointer from `0x203381`, reads its dword at `+0x1C`, and calls `0x4599C` when bit four is set. Nonzero byte `0x202A0B` also gates `0x46C70`.

The final low-nibble-zero publication snapshots dword `0x650064` into `0x20959C`, extracts bits 12..15, and writes byte `0x20A22A`. Nibble values 1 through 14 publish literal one; edge values 0 and 15 instead copy byte `0x222709`. All opaque-call returns are discarded locally except the explicitly forwarded first-periodic R0 and the exact-one predicate conversion. The callback's final return is always literal one.

## Stage 74 — current-only signed-threshold selector

Stage 74 reconstructs current `0x16F820`, an exact 76-byte leaf with no direct runtime calls and no PC-relative literal loads. The exact body has one hit in the current executable range. No byte-identical public-legacy body counterpart was found, so no legacy address is promoted; semantics are derived only from the canonical current 91900-byte Orange Pi HCD.

The routine forms a pointer-cell address as `incoming_R3 + incoming_R1*4 + 0x94`, loads a record pointer from that cell, and begins scanning at record byte `+1`. Its index starts at one. Bytes `+1` through `+7` are loaded with signed `LDRSB` and compared against the full incoming R0 interpreted as signed 32-bit. A scan byte greater than or equal to that target advances the index; exhausting all seven candidates returns literal seven without ever reading record byte zero.

On the first signed scan byte strictly below the full target, firmware then truncates the target to its low byte and reads record byte zero. The base delta is computed as wrapping `u8(target-base)`, interpreted as `i8`, and conditionally negated when negative; the `-128` wrapping edge is therefore preserved. The scan delta is wrapping `u8(target-scan_byte)` interpreted as `i8` without an absolute-value transform. If the resulting base distance is signed-greater than the scan distance, the current index is returned; otherwise firmware returns the previous index. The source model preserves the unusual split between full signed target comparison and low-byte signed-distance arithmetic, along with selective byte-zero reads and exact pointer arithmetic.

## Stage 75 — current conditional publish selector

Stage 75 reconstructs current `0x16F874`, an exact 76-byte leaf immediately preceding the already recovered Stage-60 function at `0x16F8D0`. The complete body is unique in the current Orange Pi HCD. Masking the single four-byte direct-call encoding leaves 72 fixed bytes and still yields exactly one current hit at `0x16F874`. The public legacy HCD does not yield a corresponding normalized body, so no legacy address is promoted; semantics are current-HCD-first.

The routine's first observable memory read is a candidate byte from `incoming_R1 + incoming_R2 + 4` using 32-bit wrapping address arithmetic. Only after that byte has been fetched does firmware load the context pointer from dword `0x206EA8` and inspect context byte `+0x10`. This ordering is preserved even on special paths that later discard the initial candidate.

For context type values other than `0x13`, the initial candidate byte is zero-extended and written as a dword to primary output `0x60019C`. Firmware then reads gate byte `0x20B265`; when nonzero it writes the same dword to secondary output `0x600164`. This path never calls the opaque boundary and returns the original incoming R0 unchanged.

For context type `0x13`, firmware calls opaque current boundary `0x88414(context + 0x28)` and masks its R0 return with `0xF0`. Masked value `0x80` is a strict early return: neither output address nor the secondary gate is touched. Other masked values continue to a sign-bit-controlled byte selection. Firmware reads context byte `+0x27A`; when bit 7 is clear the selected byte comes from `incoming_R0 + incoming_R2 + 0x14`. When bit 7 is set, firmware reads context halfword `+0x258` and selects from `incoming_R0 + halfword + 0x24`. The selected byte is zero-extended and published primary-first, followed by the same gate-controlled secondary publication. The final return on this non-early special path is the masked `0x88414` result, not the selected byte or incoming R0.

Current PC-relative literals resolve to `0x206EA0` (with the actual context-pointer dword at `+8`), `0x60019C`, `0x20B265`, and `0x600164`. The source model keeps `0x88414` opaque and freezes only the exact local ordering, offsets, address arithmetic, output gates, and return shapes visible in the current HCD.

## Stage 76 — current indirect callback orchestrator

Stage 76 reconstructs current `0x16F970`, an exact 160-byte function ending at `0x16FA0E`. The exact body occurs once in the current Orange Pi HCD and contains no direct `BL`/`B.W` runtime call: all runtime transfers are `BLX`/tail `BX` through current-memory function-pointer tables. No public-legacy exact body is promoted, so semantics are current-HCD-first.

The first gate reads byte `0x201AF4` and requires bit zero. Firmware then reads callback-table root dword `0x203488`; a null root returns immediately. When mode byte `0x20DAA5` is nonzero, firmware additionally requires all three dwords at `0x20DA8C`, `+4`, and `+8` to be nonzero. All these local gate exits preserve incoming R0.

On the active path firmware loads provider root dword `0x20375C`, stores the low byte of incoming R0 to `0x20D9A2`, then loads the function pointer at provider `+8` and executes `BLX` without a local null check. A zero provider-call return exits with zero. A nonzero return becomes the live R0 token for the remaining callback chain.

Every optional callback access re-reads dword `0x203488` before loading its slot, preserving callback-table replacement or mutation visibility. Nonzero slot pointers are called with the currently-live R0; null slots leave R0 unchanged. Slot `+0x00` runs first, firmware writes literal two to status byte `0x20D9A4`, then slots `+0x04` and `+0x08` run. Mode byte `0x20DAA5` is then read again, so those earlier callbacks may change the later branch.

When the second mode read is nonzero, optional slots `+0x0C` and `+0x10` run. When it is zero, optional slots `+0x14`, `+0x18`, `+0x1C`, `+0x20`, and `+0x24` run instead. After either branch firmware re-reads the callback-table root one final time and loads slot `+0x28`. A null final slot returns the current R0 token; a nonnull final slot is reached by frame restore plus `BX`, so its return is the function's final return.

The PC-relative literal pool immediately after the body resolves to `0x201AF4`, `0x203488`, `0x20DAA5`, `0x20DA8C`, `0x20375C`, and `0x20D9A2`. The source model preserves local gate returns, the unguarded provider call, callback-root re-reads, two separate mode reads, R0 chaining, status-write placement, branch-specific slot order, and final tail-call behavior without assigning wider semantics to the indirect targets.

## Stage 77 — fixed-byte post-boundary initializer

Stage 77 reconstructs current `0x170F48`, an exact 30-byte wrapper consisting of one direct call followed by five byte stores. The current raw body has exactly one hit. Masking the single four-byte call encoding leaves 26 fixed bytes and yields exactly one current hit plus one public-legacy structural counterpart at `0x16D1A0`. The two literal bases are stable across both images: `0x2032F2` and `0x2032DC`.

No incoming argument register is modified before the call, so opaque boundary `0xE558` receives the incoming R0, R1, R2, and R3 values exactly. The boundary's R0 return remains live through the rest of the wrapper because the post-call code uses only R2/R3 and memory stores; that boundary return is therefore the wrapper's final return.

After the call returns, firmware writes the following bytes in order: `1` to `0x2032F3`; `5` to `0x2032DE`; `5` to `0x2032DF`; `0x82` to `0x2032E7`; and `0xB4` to `0x2032F0`. There are no local gates, conditional branches, or return transformations. The source model keeps `0xE558` opaque while freezing the exact argument forwarding, write order, values, addresses, and return preservation.

## Stage 78 — 16-bit-progress table fold

Stage 78 reconstructs current `0x1719B8`, an exact 34-byte leaf with no runtime calls. The body is byte-identical to the public-legacy structural counterpart at `0x16DB00` and appears exactly once in each image. The PC-relative table literal relocates from legacy `0x2220A8` to current `0x222154`.

The function receives an accumulator in R0, a byte pointer in R1, and an unsigned length in R2. R4 is initialized to the start pointer. At the top of every loop iteration firmware computes `R4 - R1`, truncates that difference with `UXTH`, and performs an unsigned `R2 <= progress` exit test. This is not equivalent to comparing the full pointer delta. In particular, lengths greater than `0xFFFF` never satisfy the local exit condition because the observed progress is always a 16-bit value; the source model preserves that exact edge rather than silently widening the counter.

When the loop continues, firmware reads one byte from the current pointer and post-increments the pointer. It XORs that byte with the current accumulator, keeps only the low eight bits as a table index, loads a dword from `0x222154 + index*4`, and computes the next accumulator as `table_word ^ (accumulator >> 8)`. The loop then repeats. Length zero returns immediately without reading input or table memory.

The source model exposes only byte and table-word reads and preserves 32-bit wrapping pointer arithmetic, 16-bit progress truncation, low-byte index formation, accumulator chaining, and the exact return value.

## Stage 79 — current snapshot/table-fold publisher

Stage 79 closes current `0x1719E0`, an exact 112-byte wrapper with public-legacy structural counterpart `0x16DB28`. The current body SHA-256 is `391f49f43ec468505bebb0a5c6670b4866ff49e8bdac1434c367d1a938cb0d4c`; legacy raw SHA-256 is `b3f07511f63248cc22e54be20b342b6a6b91dbc2b2c5ee7d78d160341b27c133`. Only seven bytes differ, all inside relocation encodings for the three external direct calls. Masking those seven bytes leaves 105 fixed bytes, normalized SHA-256 `13d25e6a39c44c2cebd9b9539107411f9a45c82569392e8e76962aa549d888e0`, with exactly one current hit and one legacy hit.

The wrapper first calls opaque current `0x19754` with the incoming R0-R3 unchanged. Its return is ignored, but the ordering is observable because all snapshot loads occur afterwards. Firmware then copies seven dwords into fixed current buffer `0x222E04`: sources `0x318088`, `0x32A004`, `0x3186A0`, `0x410434`, `0x41079C`, `0x4100AC`, and `0x410548` map in order to buffer offsets `+0x00..+0x18`.

Current dword `0x222DF4` selects the span. If nonzero, previous fold dword `0x222DFC` is written to buffer `+0x1C` and the fold length is 32 bytes. If zero, firmware reads pointer dword `0x200748`, subtracts four with wrapping arithmetic, and calls the already-proven `0x3DB4` memcpy primitive to copy four bytes into buffer `+0x1C`; the fold length is then 44 bytes. The bytes at buffer `+0x20..+0x2B` are not initialized locally before the 44-byte path and therefore remain ambient state.

Both paths call already-recovered Stage 78 at `0x1719B8` as `(0xFFFFFFFF, 0x222E04, length)`. Stage 78 preserves R1/R2 and returns with R3 equal to terminating 16-bit progress; for local lengths 32 and 44 this is exactly `length`. The immediately following opaque `0x19318` therefore receives `(fold_result, 0x222E04, length, length)`. Its return is ignored. Finally firmware writes dword one to `0x222DF4`, stores the Stage-78 fold result to `0x222DFC`, and returns that fold result.

Public-legacy literals retain the same external snapshot addresses and pointer cell but relocate the buffer/state block from current `0x222DF4/0x222DFC/0x222E04` to legacy `0x222C60/0x222C68/0x222C70`. The source model keeps `0x19754` and `0x19318` opaque and composes the known memcpy/Stage-78 behavior without assigning broader protocol meaning.

## Stage 80 — current fixed global initializer

Stage 80 reconstructs current `0x171B5C`, an exact 24-byte leaf with SHA-256 `8d7e0bc7b879564e498bf8d2d93e8717936a8149f7abfb214fe216586a9fc60b`. The exact body has one current hit. A public-legacy counterpart is not promoted: even after masking only the four PC-relative `LDR literal` imm8 bytes, the remaining 20-byte structural pattern has no legacy hit.

The body contains no runtime calls and no conditional control flow. Four current literals resolve to `0x204B18`, `0x3D090`, `0x204B10`, and `0x352600`. Firmware first stores the same dword `0x3D090` into pointer slots `0x204B18` and `0x204B10`, then writes zero to dword `0x352600`, then writes literal `0x1FFF` to dword `0x352614`.

R0 is never modified, so the function returns the incoming R0 value exactly. The source model exposes only the four ordered dword writes and preserves that return shape; no broader role is assigned to the pointed-to regions.

## Stage 81 — current critical-state repair wrapper

Stage 81 reconstructs current `0x171B84`, a 62-byte wrapper whose final wide branch restores the critical-state token rather than returning through a local `POP {pc}`. Masking the three four-byte control-transfer encodings leaves 50 fixed bytes and exactly one current structural hit. No public-legacy structural counterpart is promoted; this stage is current-HCD-first.

The wrapper preserves incoming R0, then calls the already identified critical-state swap entry `0x780(1)` and retains its returned token. It reads dword `0x352614` and compares it with literal `0x1FFF`. Only on mismatch does it write zero to `0x352600` and `0x1FFF` to `0x352614`, in that order.

After the repair gate it reads byte `0x2170EF`. A nonzero byte skips the optional call. A zero byte is first changed to one, then firmware calls opaque `0x15180` with the exact register shape `(0x217174, original incoming R0, 1)`. That call's return is ignored.

Finally the saved critical token is restored into R0, the local frame is popped without PC, and firmware tail-branches to `0x780(token)`. The return from that restore boundary is therefore the function's final return. The source model preserves the critical enter/repair/flag/finalize/restore order and keeps the `0x15180` contract opaque.

## Stage 82 — reset/gate/dispatch sequence

Stage 82 reconstructs current `0x171BD4`, an exact 112-byte function with a public-legacy structural counterpart at `0x16D98C`. Masking the seven four-byte direct-call encodings leaves 84 fixed bytes and exactly one current plus one legacy structural hit. Current direct targets are opaque setup entries `0x151E0` and `0x15214`, recovered Stage 81 at `0x171B84`, opaque `0xBAA08`, and opaque `0xBA988` called three times.

The function begins with `0x151E0(0x217174, 0x171D05, 0)` followed by `0x15214(0x217174, 1)`. The first return is ignored; the second return remains live in R0 unless a later Stage-81 or chain call replaces it. It then performs four fixed stores in binary order: byte `0x2170EF = 0`, halfword `0x2170EC = 0`, halfword `0x217170 = 0`, and halfword `0x204B14 = 0x50`.

Firmware reads dword `0x352604` and compares only its low 20 bits with `0xFFFFF`. A match writes byte `0x2170EE = 1`. A mismatch instead loads dword `0x204B18`, calls recovered Stage 81 with that value, and keeps the Stage-81 return in R0. The status dword at `0x352604` is then re-read after that optional call, preserving mutation visibility.

The second gate compares the full re-read status with exact literal `0x200FFFFF`. On mismatch firmware writes dword zero to `0x222E00` and returns the current R0. On exact match it loads dword `0x352608`, calls `0xBAA08`, then threads R0 through three consecutive calls to `0xBA988`, writes dword one to `0x222E00`, and returns the third `0xBA988` result. The source model preserves the reset order, two distinct status tests, post-Stage81 reread, live-R0 return shape, and four-call chain while keeping unresolved boundaries opaque.

## Stage 83 — low-20/full-word gate with Stage-81 fallback

Stage 83 reconstructs current `0x171D08`, an exact 62-byte wrapper with public-legacy structural counterpart `0x16D92C`. Masking the four-byte `0xBAA08` call and the final wide tail transfer leaves 54 fixed bytes. The current raw body SHA-256 is `aa982abca3cfe957b67570e4e7f4d92fc68d388dc318395e097bbe0a7827f2ac`, the public-legacy raw body SHA-256 is `09a0a9c26b3a718f1228e5762b317adace9e1ff8e706564550d4191236a742c3`, and normalized SHA-256 is `d17975c53c181fab99462dfd8013f73b50bd28cd9553469f0ca132add881b6b2`. The normalized signature has exactly one hit in each image. Public legacy remains structural evidence only.

Incoming R2 is a pointer to a dword and firmware intentionally loads that dword twice on the match path. The first load is reduced with `UBFX(..., 0, 20)` and compared against the full incoming R1. A mismatch clears byte `0x2170EF`, reads dword `0x204B10`, logical-shifts it right by one, and tail-transfers the result to already recovered Stage 81 at current `0x171B84`; that Stage-81 return is final.

On a low-20 match firmware rereads the full dword through incoming R2. If the reread is not exact `0x200FFFFF`, it writes one to byte `0x2170EE` and returns incoming R0 unchanged. Exact `0x200FFFFF` instead reads dword `0x352608`, calls opaque `0xBAA08(value)`, writes literal one to current dword `0x222E00`, then writes one to byte `0x2170EE`; the `0xBAA08` return remains final through both stores. The public-legacy structural literal corresponding to the publish word is relocated to `0x222C6C`, while the other fixed literals remain the same.

The source model preserves the two separate input-word reads, full-R1 versus low-20 comparison, mismatch flag clear, logical shift before Stage-81 tail, exact full-word special gate, publish-before-ready ordering, and all three final-return shapes without assigning broader meaning to opaque `0xBAA08`.

## Stage 84 — fixed callback-slot publisher

Stage 84 reconstructs current `0x1720D8`, an eight-byte leaf that loads two PC-relative literals, stores the second through the first, and returns. The raw eight-byte body `014b024a1a607047` is a common code shape and is therefore not treated as a unique provenance signature by itself.

The authoritative structural signature is the 16-byte body-plus-literal context. Current bytes are `014b024a1a607047d4662100d91f1700`; the public-legacy structural context at `0x16E028` is `014b024a1a607047d466210029df1600`. Both resolve the destination slot to `0x2166D4`; only the callback Thumb pointer relocates, current `0x171FD9` versus legacy `0x16DF29`. Masking those four callback-pointer bytes leaves 12 fixed bytes and exactly one current and one legacy structural hit.

The local semantics are exact and intentionally narrow: write raw current Thumb pointer `0x171FD9` as one dword to fixed slot `0x2166D4`, do not touch any other state, and return incoming R0 unchanged. No semantic role is assigned to the callback beyond the literal publication visible in the firmware.

## Stage 85 — two-halfword circular-distance helper

Stage 85 reconstructs current `0x17192C`, a 26-byte no-call leaf. Including the two PC-relative literal dwords produces a 34-byte context that is byte-identical to public-legacy structural `0x16DA74` and unique in both images. The executable-body SHA-256 is `aaba7d066a004a9249d95a66a76d359728a66bf0037a895adca855480b79f025`; the 34-byte body-plus-literals SHA-256 is `85cedc18833adb94022772de089afa8d4ac60577260cd7740f6c6dea6d319393`.

Firmware reads halfword `0x2170EC` first and halfword `0x217170` second. Both are zero-extended for the arithmetic. If the first value is lower, the return is `second-first`. Equality returns zero. If the first is greater, firmware adds literal 100 to the second value and then subtracts the first. These are ordinary 32-bit `ADDS`/`SUBS` operations; no local bounds check or explicit modulo normalization exists, so values outside the presumed 0..99 domain can produce a wrapping u32 result and the source model preserves that edge.

The helper has no runtime calls, no memory writes, and no dependency on incoming R0 because R0 is overwritten by the second halfword load before any use.

## Stage 86 — guarded table-step helper

Stage 86 reconstructs current `0x1718E8`, a 56-byte helper called by the larger control wrapper at `0x171C78`. Public legacy contains its relocation-normalized structural counterpart at `0x16DA30`. The two ordinary calls both target current critical-state boundary `0x780`; after masking those two call encodings the remaining body bytes are fixed and the normalized body is unique in both images.

Firmware preserves incoming R0 as an output pointer, calls `0x780(1)`, and keeps that return as the restore token. Only then does it read halfword `0x2170EC` followed by halfword `0x217170`. Equal values skip the table and counter update and produce local result zero. Unequal values use the original first halfword as an unchecked dword-table index into current base `0x222E34`, store the selected dword through the incoming R0 pointer, increment the first halfword with 16-bit wrap, replace any post-increment value above 99 with zero, and write it back to `0x2170EC`; this path produces local result one.

The helper finally calls `0x780(saved_token)` to restore the critical state. That restore return is ignored: the final R0 is explicitly replaced with the local zero/one result. The source model therefore preserves the critical-section ordering, unchecked original table index, 16-bit-before-range-check increment shape, and final-return override without assigning a broader semantic name to the table or critical-state boundary.

## Stage 87 — staged initialization/control wrapper

Stage 87 reconstructs current `0x171C78`, an exact 118-byte wrapper with public-legacy structural counterpart `0x16DBC4`. Twelve direct-call encodings relocate; masking those encodings leaves 70 fixed bytes and a unique normalized hit in both images. Current literals resolve to canary `0x200890`, initialized flag `0x222E00`, live threshold halfword `0x204B14`, fallback gate byte `0x222DF8`, and shift-source dword `0x204B10`; the two `0x222E00/0x222DF8` globals are relocated in the legacy image.

On entry firmware snapshots the canary and keeps incoming R0 both in R6 and in stack-local slot zero. When the initialized dword is zero, it calls recovered Stage 79 with register shape `(incoming_r0, incoming_r1, 0, 0x200890)`, threads that return through opaque `0xBAA08` and three consecutive `0xBA988` calls, then stores one to the initialized dword. Those chain returns do not directly become the function return.

If incoming R0 is zero, recovered Stage 85 is called and its distance is compared with a live read of halfword `0x204B14`; distance at or below threshold calls `0xBA988(distance)` and stores that return into stack-local slot zero. Otherwise, and unconditionally for nonzero incoming R0, recovered Stage 86 receives a pointer to that same stack-local slot. A Stage-86 result of one skips the fallback; a zero result reads byte `0x222DF8`, calling opaque `0xBDDBC` when nonzero or `0xBA988(0)` and replacing the local word when zero.

Recovered Stage 85 is then called a second time and the threshold halfword is reread. A second distance at or below that live threshold reads dword `0x204B10`, shifts it right one, and calls recovered Stage 81; that return is ignored. Finally the canary is reread. A mismatch calls hardening boundary `0x94C0` with the current stack-local result in R0. The wrapper's observable final return is the stack-local word: initially incoming R0, but it may have been replaced by Stage 86 through pointer aliasing or by the conditional `0xBA988` call.

## Stage 88 — current wait/reset wrapper

Stage 88 reconstructs current `0x171D68`, an exact 92-byte wrapper immediately after the already recovered Stage-83 region. The body contains five direct control transfers. Masking only those twenty relocation bytes leaves 72 fixed bytes and exactly one current hit. No trustworthy public-legacy structural counterpart is promoted: several strong internal anchors have no match in the public 73136-byte image, so this stage is current-HCD-first rather than forcing an address-delta correspondence.

The function first reads dword `0x352604`, extracts its low twenty bits, and compares them with literal `0x000FFFFF`. A mismatch clears byte `0x2170EF`, reads dword `0x204B10`, logical-shifts that value right by one, and tail-forwards it into the already recovered Stage-81 entry `0x171B84`. Incoming R0 is not part of that mismatch tail result.

A low-twenty match does not reuse the first status snapshot. Firmware rereads the full dword at `0x352604` and requires exact `0x200FFFFF`. While it differs, opaque `0xB0210` is called and the status word is reread again with no local retry limit. The wait call sees live R0, literal R1 `0x000FFFFF`, the initial low-twenty snapshot in R2, and the current failing full status in R3. Its return becomes the next live R0. Therefore, when the exact status is finally observed, the following opaque `0xBAAE4` sees either incoming R0 when no wait occurred or the return of the final wait call; this mutation-visibility edge is modeled explicitly.

After the exact status arrives, firmware writes dword `3` to `0x352600`, calls `0xBAAE4` with live R0 and register arguments `(R1=0x000FFFFF, R2=3, R3=0x352600)`, and ignores its return. It then calls critical-state boundary `0x780(1)` and preserves that returned token across four exact clears: dwords `0x352604` and `0x352614`, then bytes `0x2170EF` and `0x2170EE`. The final tail transfer is `0x780(saved_token)`, whose return is the function return. Runtime meanings of `0xB0210`, `0xBAAE4`, and `0x780` remain opaque beyond these observed register/order contracts.

## Stage 89 — fixed-global tail thunk

Stage 89 closes current entry `0x171DE0`. The exact function body is eight bytes: it loads R3 from the adjacent PC-relative literal, loads R0 from `[R3]`, and performs a wide tail branch to opaque boundary `0x13218`. The adjacent dword at `0x171DE8` is literal data `0x222078`, not executable body.

This tiny thunk has a useful ABI edge. Incoming R0 is replaced by `*0x222078`; incoming R1 and R2 are not touched; incoming R3 is replaced by literal address `0x222078` itself. Because the last instruction is a tail branch rather than a call/return pair, boundary `0x13218` receives exactly that register shape and its return is the thunk's final return with no local transformation.

The eight-byte body SHA-256 is `751b89e29eacb8def3bac515e947e9332028d7171646ed7f112fdae734dee068`. The 12-byte body-plus-literal context is unique in the current HCD. After masking the four branch-encoding bytes, normalized context SHA-256 is `817032c2cd1b973648c7edadc81d440f177fa280c45660f1444093766cafb020`. No public-legacy structural counterpart is promoted for this entry; semantics are current-HCD-first.

## Stage 90 — bounded signed status poll

Stage 90 reconstructs current `0x171DEC`, an exact 20-byte leaf. The adjacent PC-relative literal resolves to fixed dword address `0x650318`. The body is byte-identical to the public-legacy structural counterpart at `0x16DD3C`.

Firmware initializes R0 to decimal 100 and reloads `*0x650318` each iteration. The loaded dword is compared with zero using signed condition codes. Any signed-nonnegative value returns literal one immediately. A negative value decrements R0 and repeats while the counter is nonzero. Therefore an all-negative stream performs exactly 100 loads and returns zero. The source model preserves repeated live reads and does not invent a delay, snapshot, or side effect.

## Stage 91 — bounded bit-30 poll

Stage 91 reconstructs current `0x171E04`, an exact 20-byte leaf that is byte-identical to public-legacy structural counterpart `0x16DD54`. The adjacent literal resolves to fixed dword address `0x650310`.

Each iteration reloads `*0x650310`, executes `LSLS R3,R3,#1`, and branches on the N flag. The N flag after that shift reflects original bit 30, not bit 31. A set bit 30 returns literal one immediately. Otherwise the loop decrements an initial count of 100 and rereads the dword; after exactly 100 clear observations it returns zero. The model keeps this live-read behavior and does not invent any delay or snapshot.

## Stage 92 — guarded four-byte transfer sequence

Stage 92 reconstructs current `0x171E1C`, an exact 88-byte wrapper with public-legacy structural counterpart `0x16DD6C`. The current literal pool resolves to guard `0x200890`, source snapshot `0x222554`, mode/poll dword `0x650310`, byte publication dword `0x650328`, Stage-90 status dword `0x650318`, and control dword `0x650314`. The legacy pool is identical except the source snapshot is the older relocated address `0x2224A8`.

The wrapper snapshots the stack guard and source dword before testing bit 4 of `*0x650310`. When that bit is set, the local result is zero and the active transfer is skipped. When clear, firmware consumes the four bytes of the saved source dword in little-endian order. For each byte it writes the zero-extended byte to `0x650328`, writes `0x81000000` to `0x650318`, and calls the already recovered Stage-90 bounded signed poll; each Stage-90 return is ignored. After four bytes it calls Stage 91 once, ignores that return, then ORs only bit 3 into dword `0x650314` and sets local result one.

The stack guard is reread on both paths. A mismatch calls hardening boundary `0x94C0`. If that boundary returns, its R0 is the wrapper's final return because the epilogue does not restore the prior local result. The source model preserves that observable machine-code shape without assigning business meaning to the hardening boundary.

## Stage 93 — clear ambient control bit 3

Stage 93 closes current `0x171E8C`, an exact 12-byte leaf. The public 73136-byte legacy HCD contains the byte-identical structural body at `0x16DDDC`; both bodies have SHA-256 `8882b328cb6513904b88814cc2bc74155c538f6941d5e5815fe7f60be190369a` and each image has exactly one exact-body hit. The PC-relative literal immediately after the function resolves to stable dword `0x650314` in both images.

The leaf reads `*0x650314`, clears only bit 3 with `BIC`, writes the dword back, and returns through `BX LR`. It does not assign R0, so incoming R0 is the observable return value. There is no local gate and even an already-clear bit still performs the read and write. The source model preserves that exact one-read/one-write shape without assigning a wider semantic role to the ambient word.

## Stage 94 — ambient three-bit extractor

Stage 94 closes current `0x171E9C`. Its executable body is exactly 10 bytes and is byte-identical to public-legacy structural `0x16DDEC`, with SHA-256 `46b411d628c7bbce798d1ae39f81ebbbfab01286aa370d51c6c1abde4025a5d6` and one exact hit in each image. The `NOP` immediately after `BX LR` is alignment and is deliberately excluded from the function body. The following literal resolves to stable dword `0x65031C` in both images.

Firmware performs one dword load from `0x65031C` and returns `UBFX(word, 16, 3)`, i.e. bits 16 through 18 as a value from zero through seven. No extra masking, gate, or second read is introduced by the source model.

## Stage 95 — encode-and-poll tail wrapper

Stage 95 closes current `0x171EAC`. The executable body is exactly 22 bytes; the following `NOP` is alignment and is not part of the function. Public-legacy structural `0x16DDFC` is byte-identical, with SHA-256 `acab3c9b7c879c535a4d96d0100b8bed9b79f93db92626e4a3cd74a026b842e2` and one exact hit in each image. Its literal pool resolves identically in both images to `0x650328`, `0x0001FF00`, and `0x650318`.

Firmware stores incoming R0 to dword `0x650328`, computes `((incoming_R1 << 8) & 0x0001FF00) | 0x85000000`, and stores that dword to `0x650318`. The final wide branch is a tail transfer directly to the already recovered Stage-90 bounded signed-status poll at `0x171DEC`; therefore Stage 90's return is this wrapper's final return. The source model preserves the two-store order, 32-bit shift/mask behavior, and tail-return shape.

## Stage 96 — Stage-25 mode-one target closure

Stage 96 closes the raw current mode-one target `0x17218C` that Stage 25 had previously recorded without an implementation. The exact executable body is 80 bytes and is a relocation-normalized structural counterpart of public-legacy `0x16E0DC`. Masking the ten four-byte control-transfer encodings leaves 40 fixed bytes with exactly one hit in each image. The current raw body SHA-256 is `174f753c80a3e728fc05b7a28fb8ac9395264585d54119e5ce69c8e3d8ec5d07`; legacy raw SHA-256 is `96df136fd06c5e24628f1bbed2742602dc28aeecc85465a4f2c91f814ff41098`; normalized SHA-256 is `f41ecbfd317c3d109c45817edc5d90613ff03fc82bd88266deca2fec4d5c1085`.

The four current literals after the body resolve to callback slot `0x2166D4`, callback Thumb pointer `0x171FD9`, mode byte `0x223064`, and fallback gate base `0x20CEC4`. The legacy counterpart preserves the stable slot/gate base while relocating the callback pointer and mode byte. The routine first publishes the callback pointer and writes mode value two. It then executes opaque boundaries `0x86184`, `0x86370`, `0x89320`, `0x89398`, `0x860DC`, `0x959C0`, and `0x3386C`. Firmware explicitly resets R0 to zero before each of `0x89320`, `0x89398`, and `0x860DC`; thereafter the R0 return chain flows through `0x959C0` into `0x3386C`.

A nonzero `0x3386C` result skips the ambient byte read and reaches `0x2DE80`. An exact zero result reads byte `0x20CEDD`; a zero byte performs a tail transfer to the Stage-25 mode-two target `0x1720E8` with locally known R0 still zero. A nonzero byte instead calls `0x2DE80` with R0 zero. Both primary paths then replace R0/R1 with literals 22 and 19 and tail-transfer to opaque `0x8AEAC`; that tail result is final. The source model preserves these exact setup, zero-reset, short-circuit, R0-chain, and tail-return properties while leaving unspecified caller-volatile state of opaque boundaries unpromoted.
