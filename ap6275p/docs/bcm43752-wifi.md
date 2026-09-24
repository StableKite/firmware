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

## Stage-20 current control-flow anchors

Stage 20 follows the exact Stage-19 thunks on the **current** Orange Pi bytes rather than assuming their legacy destinations. Each 4-byte thunk instruction is SHA-256 checked against the legacy instruction, decoded again at its relocated current address, and required to produce the expected current target.

| Exact thunk | Current thunk | Current destination | Claim |
| --- | --- | --- | --- |
| `j_dngl_sendwl` | `0x185bec` | `0x185b00` | direct Thumb `B.W` destination |
| `j_hnd_free` | `0x1a5cc6` | `0x1a5fd4` | direct Thumb `B.W` destination |
| `j_nullsub_56` | `0x1a6960` | `0x1a63bc` | direct Thumb `B.W` destination |

The destination addresses are current control-flow anchors. Stage 20 does **not** claim that the destination function bodies are byte-identical to the legacy implementations.

For repeated exact bodies, Stage 20 also builds a strictly monotonic spine from the 608 globally unique >=8-byte Wi-Fi matches. Of the 37 globally repeated exact bodies, 29 have exactly one occurrence between their nearest monotonic exact neighbors and are therefore recorded as context-disambiguated structural relocations. They are not automatically promoted to semantic names.

## Stage-21 destination-body closure

Stage 21 starts only from Stage-20 **current** destinations and follows corresponding direct branch sites recursively. A function is admitted only when its complete current body has the same SHA-256 as the legacy body after canonicalizing direct Thumb `BL`/`B.W`/wide-conditional branch immediates. Branch-site offsets and branch kinds must also match.

This proves a 13-function Wi-Fi relocation-normalized closure. Named current anchors inside that closure are:

| Name | Legacy | Current | Bytes | Current corroboration |
| --- | --- | --- | ---: | --- |
| `dngl_getdev_by_ifidx` | `0x184528` | `0x185920` | 32 | Stage-19 exact full body |
| `dngl_finddev` | `0x184548` | `0x185940` | 50 | normalized body + current `"dngl_finddev"` and error string |
| `dngl_sendwl` | `0x184708` | `0x185b00` | 220 | normalized body + current `"dngl_sendwl"` and drop diagnostic |
| `hnd_free` | `0x1a3c1c` | `0x1a5fd4` | 414 | normalized body + current `"hnd_free"` and allocator diagnostics |

The Stage-20 `j_nullsub_56` destination `0x1a63bc` begins with current opcode `0x4770` (`BX LR`), directly confirming a return/no-op stub at that destination.

Thirteen direct-call/tail-call ROM-side boundaries reached by this verified closure remain at the exact same absolute current addresses: `0xa814`, `0x11d54`, `0x12d10`, `0x6fdac`, `0x70718`, `0x70814`, `0x70b80`, `0x70e10`, `0x710cc`, `0x710dc`, `0x711c8`, `0x71248`, `0x7616c`. In particular, current `hnd_free` still calls `0x70e10`, upgrading the previously legacy-only heap-block-size-like boundary to a current-image call boundary.

Literal pools are not assumed equal. Stage 21 reads them from the current image: the `dngl_*` identity/diagnostic strings moved into the `0x2002xx` range and the `hnd_free` strings into the `0x2024xx` range while low RAM pointers such as `0x170100`, `0x170220`, and `0x170224` remain unchanged.

## Stage-22 current ROM-boundary semantics

Stage 21 proves the complete current instruction skeleton for 13 functions. Stage 22 audits every direct ROM-boundary call from that verified closure: 28 current call sites reach 13 stable absolute ROM targets.

Two roles are now promoted for the **current** image because current call-site layout and the prior behavioral recovery agree:

- `0xA814`: diagnostic/printf-like sink — 6 verified current call sites, including the miss path of current `dngl_finddev` and the invalid/free diagnostics in current `hnd_free`.
- `0x70E10`: heap-block-size-like boundary — 7 verified calls, all from current `hnd_free`.

Seven additional addresses reached only from the verified `hnd_free` body (`0x70718`, `0x70814`, `0x70b80`, `0x710cc`, `0x710dc`, `0x711c8`, `0x71248`) are kept as **hnd_free-internal boundaries** without inventing vendor semantics. Four targets (`0x11d54`, `0x12d10`, `0x6fdac`, `0x7616c`) remain unresolved.

The source now includes a libre `dngl_finddev_with_diagnostics` wrapper: it reuses the already reconstructed interface-index mapping and exposes the optional miss diagnostic through a trait rather than depending on the proprietary ROM logger.

## Stage-23 unresolved-boundary ABI freeze

Stage 22 left five calls across four ROM targets intentionally unresolved. Stage 23 does not guess their vendor symbols. Instead it freezes the exact call shapes reached from the Stage-21 relocation-normalized current closure:

| ROM target | Current caller | Calls | Verified shape |
| --- | --- | ---: | --- |
| `0x11d54` | `sub_185538` → current `0x1869f0` | 1 | one-argument tail boundary |
| `0x12d10` | `sub_1A3718` → current `0x1a5ad0` | 1 | two-argument direct call |
| `0x6fdac` | `sub_1A3718` → current `0x1a5ad0` | 2 | zero-argument direct call |
| `0x7616c` | `sub_1A36D0` → current `0x1a5a88` | 1 | one-argument call whose return value is consumed |

These are ABI-shape facts only. They deliberately remain `Unresolved` in the source model until stronger current-image evidence establishes semantics.

## Stage-24 current wrapper semantics

Two Stage-21 relocation-normalized functions are now lifted to source while their ROM dependencies remain explicitly opaque.

- Current `0x1869f0` (legacy `sub_185538`) selects one of two optional callback shapes and then always tail-calls `0x11d54` with the second argument. `0x11d54` remains an unresolved one-argument tail boundary.
- Current `0x1a5a88` (legacy `sub_1A36D0`) increments an 8-bit wrapping counter. The callback path executes only when the incremented value is 0 or 1. If a callback is present it queries `0x7616c`; a nonzero result is replaced by a current literal fallback before the pair callback. `0x7616c` remains an unresolved one-argument return boundary.

Rust models both ROM entries through traits. No vendor symbol or stronger behavior is assigned.

## Stage-25 current deadman control

Four legacy functions around the deadman state machine have exactly one relocation-normalized current match:

| Legacy | Current | Role |
| --- | --- | --- |
| `sub_1A3718` `0x1a3718` | `0x1a5ad0` | terminal/fatal wrapper; structural only |
| `sub_1A375C` `0x1a375c` | `0x1a5b14` | elapsed-threshold rearm wrapper |
| `sub_1A378C` `0x1a378c` | `0x1a5b44` | mode-to-boundary argument wrapper |
| `sub_1A37A8` `0x1a37a8` | `0x1a5b60` | deadman state machine; structural/control-flow anchor |

The current image independently contains `deadman_state_machine` at `0x202344` and `%s: Unexpected event [%d] in state [%d]!!!\n` at `0x20235a`.

Current `0x1a5b14` has one threshold word: the same value is first tested for nonzero and then used in the strict unsigned test `threshold < now-last`; on success it stores `now` before calling stable `0x12d10(handle,configured_value)`. Current `0x1a5b44` calls the same boundary with the stored value only when mode is 1, otherwise with zero. Rust models `0x12d10` as an opaque two-argument trait. `0x6fdac`, reached twice by current `0x1a5ad0`, also remains opaque.

No vendor symbol is assigned to either ROM boundary.


## Stage-27 complete deadman runtime

Stage 27 corrects one Stage-25 over-parameterization and closes the current deadman runtime around the verified state-machine body. Legacy/current `sub_1A375C` / `0x1a5b14` has a single threshold word, not independent `enabled` and `threshold` values: the same word must be nonzero and must satisfy the strict unsigned comparison `threshold < wrapping(now-last)` before rearm.

An exhaustive scan of the whole current Wi-Fi image finds exactly one relocation-normalized match for each promoted body: state machine `0x1a5b60`, sample-change helper `0x1a5c08`, thresholded-sample helper `0x1a5c28`, opaque lookup/output wrapper `0x1a5ca6`, and their three RAM-side dependency callees `0x1a6898`, `0x1a67d4`, `0x1a6824`. Current literals and branch destinations are re-read independently.

The state-machine source model preserves the exact 32-bit wrapping transitions. State 0 accepts event 0 by applying the stored deadman value and entering state 1; other events use the verified diagnostic path. In state 1, event 2 increments object/global outstanding counters with wraparound. Event 3 marks object offset `+0xB4`, decrements both counters with wraparound, optionally obtains the current value through the verified event-3 gate and stable ROM `0x6fb24`, and applies the corrected threshold rearm. If outstanding becomes zero and `(event & ~2) == 1`, the source disables the boundary value and returns to state 0. Critical enter/leave and the still-opaque event-3 routines remain traits.

Current `0x1a5c08` performs wrapping sample subtraction, returns zero when unchanged, otherwise stores the new sample and tail-dispatches an opaque change handler. Current `0x1a5c28` additionally implements the ARM register-shift threshold `1 << (shift & 0xff)` with shifts >=32 producing zero; its one-byte flag is cleared on zero delta and made sticky when already set or when delta exceeds the shifted threshold. The final `(delta,flag)` evaluator remains opaque.

Current `0x1a5ca6` calls stable ROM `0x703c0` with its third and fourth register arguments forced to zero. Its two output words are updated only for a nonzero result. No vendor name is assigned to `0x703c0`.

## Stage 28 — current heap/control front-end

Stage 28 extends the current-image closure without IDA. Globally unique relocation-normalized complete-body matches plus current literal/string re-reading identify the heap/control front-end at `0x1a5ccc..0x1a5f1c`. The allocator core at `0x1a5dac` is recorded as a structural anchor only; its internal free-list policy is deliberately not promoted to source semantics.

Verified current functions are `heap_store_context` `0x1a5cd8`, flag setters `0x1a5ce4`/`0x1a5d10`, status-report wrapper `0x1a5cf8`, record-address helper `0x1a5d64`, handle selector `0x1a5d74`, allocator core `0x1a5dac`, and allocation front-end `0x1a5f1c`. The tiny context/list-head getters at `0x1a5ccc` and `0x1a5d5c` are accepted only because unique normalized callers branch to them and their current literals re-read as `0x20a584` and `0x209268`.

Current data evidence fixes context `0x20a584`, record base `0x20a6b0`, default-handle object `0x20a6a8`, status format `0x2024d2` (`"\nFWID 01-%x\nflags %x\n"`), and bad-handle diagnostic `0x2025ac` (`"Handle is greater than max supported value\n"`). The context prefix has flags at `+0`, a word at `+4`, diagnostic word at `+0x1c`, and the `0x200000` setter word at `+0x60`.

The handle selector preserves the terminal invalid-handle path: when record selection is enabled and `handle > 2`, the firmware emits the diagnostic then enters the Stage-25 deadman-fatal routine. It is therefore represented as `FatalInvalid`, not as a successful pointer result. Valid selected handles map to `0x20a6b0 + 24*handle + 16`; selection disabled returns the default object at `0x20a6a8`.

The allocation front-end preserves only proven routing: ARM register-shift `1 << class` (register-shift semantics) above 4 goes to opaque ROM `0x703c0`; size above `0xfffffc` is rejected; otherwise size is aligned upward to four bytes before entering structural allocator core `0x1a5dac`. Later list manipulation remains outside the Stage-28 source claim.
