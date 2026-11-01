# AVR Delay Generator

A Rust library and planned command-line tool for generating **cycle-counted AVR assembly delay routines** from a target clock frequency and requested duration.

> **Status:** Early development / design. The CLI, assembly generation, and timing verification described below are planned features, not yet implemented.

## Goal

Generate a busy-wait delay routine that:

- Targets a requested delay for a specified CPU frequency.
- Supports rounding **up** (never undershoot the requested duration) or to the **nearest** cycle.
- Chooses between short instruction sequences, 8-bit counters, 16-bit counters, and nested counters.
- Preserves any general-purpose registers it modifies and restores the AVR status register (`SREG`).
- Reports the **actual** instruction-cycle count, achieved delay, and timing error.

Initial target: **classic AVR instruction timing**. Other AVR families may have different instruction availability or cycle counts.

## Proposed CLI

```sh
avr-delay --clock 16000000 --delay 20ms --round up
avr-delay --clock 16000000 --delay 63ns --round nearest
```

The executable will use the same core calculation and planning code exposed by the Rust library.

## Timing mathematics

### 1. Requested clock cycles

For clock frequency `f` in hertz and requested time `t` in seconds:

$$
C_{\text{ideal}} = f\,t
$$

If the requested time is in nanoseconds:

$$
C_{\text{ideal}} = \frac{f_{\text{Hz}}\,t_{\text{ns}}}{10^9}
$$

Rounding up:

$$
C_{\text{target,up}} = \left\lceil\frac{f_{\text{Hz}}\,t_{\text{ns}}}{10^9}\right\rceil
$$

Rounding to the nearest cycle (halfway cases round upward):

$$
C_{\text{target,nearest}} = \left\lfloor\frac{f_{\text{Hz}}\,t_{\text{ns}}}{10^9}+\frac12\right\rfloor
$$

Use integer arithmetic (`u128` intermediates in Rust) to avoid floating-point precision loss. Check that the final result fits the selected output integer type.

**Example:** At 16 MHz, a 20 ms delay requires:

$$
C = 16{,}000{,}000\times0.020 = 320{,}000\text{ cycles}
$$

One cycle lasts:

$$
T_{\text{cycle}}=\frac{1}{f_{\text{Hz}}}
$$

At 16 MHz, that is 62.5 ns.

### 2. An 8-bit countdown loop

```asm
    ldi  r18, 100
loop:
    dec  r18
    brne loop
```

On classic AVR:

| Instruction | Cycles |
| --- | ---: |
| `LDI` | 1 |
| `DEC` | 1 |
| `BRNE` taken | 2 |
| `BRNE` not taken | 1 |
| `NOP` | 1 |
| `RJMP` | 2 |
| `PUSH` / `POP` | 2 each |
| `IN` / `OUT` | 1 each |
| `RET` | 4 |

For `n` iterations (`1 <= n <= 256`), the body has `n` decrements, `n - 1` taken branches, and one untaken branch:

$$
C_{8,\mathrm{body}}(n)=n+2(n-1)+1=3n-1
$$

With the `LDI` initialization:

$$
\boxed{C_8(n)=3n}
$$

Thus the example requires **300 cycles**, excluding any surrounding register preservation, status restoration, `RET`, and the caller's call instruction. For 256 iterations, initialize the counter to zero and use wraparound.

### 3. A 16-bit countdown loop

```asm
    ldi  r24, lo8(50000)
    ldi  r25, hi8(50000)
loop:
    sbiw r24, 1
    brne loop
```

For classic AVR, `SBIW` takes 2 cycles. Therefore:

$$
C_{16,\mathrm{body}}(n)=2n+2(n-1)+1=4n-1
$$

With two `LDI` instructions:

$$
\boxed{C_{16}(n)=4n+1}
$$

For `n = 50000`, total initialization plus loop cost is **200,001 cycles**. A full 65,536-iteration countdown can be represented by loading the pair with zero and relying on wraparound.

**Constraint:** `SBIW` works only on register pairs `r25:r24`, `r27:r26`, `r29:r28`, or `r31:r30` on supported classic AVR devices.

### 4. Nested counters

When one counter is insufficient, repeat an inner countdown loop using an outer counter. The inner counter must be **reinitialized on each outer iteration**.

Let:

- `O` = number of outer iterations.
- `I` = number of inner iterations.
- `C_inner(I)` = initialization **plus** body cycles of the inner loop.
- `C_outer_step` = cycles required to decrement the outer counter.
- `C_outer_init` = one-time cost to initialize the outer counter.
- `C_outer_branch_taken` / `C_outer_branch_final` = outer branch costs.

Then:

$$
\boxed{C_{\text{nested}}=C_{\text{outer init}}+O\bigl(C_{\text{inner}}(I)+C_{\text{outer step}}\bigr)+(O-1)C_{\text{outer branch taken}}+C_{\text{outer branch final}}}
$$

This is a **template-dependent** formula, not a universal fixed cost: the precise instruction sequence determines each term. Additional padding and state-preservation instructions must be counted separately.

### 5. Register preservation and subroutine overhead

Generated code must account for any modified general-purpose registers **and** `SREG`, since counter instructions modify condition flags.

Illustrative classic-AVR save/restore sequence for one working register:

```asm
    push r18
    in   r18, 0x3F   ; SREG I/O address on classic AVR
    push r18

    ; Delay body; save any additional working registers as needed.

    pop  r18
    out  0x3F, r18
    pop  r18
    ret
```

This example incurs **14 cycles** outside the delay body: `PUSH` × 2 (4), `IN` (1), `POP` × 2 (4), `OUT` (1), and `RET` (4) = **14 cycles**.

> **Timing boundary:** By default, planned timing should cover the generated subroutine from its first instruction through `RET`, but not the caller's `RCALL`/`CALL`. A different timing boundary must be explicitly documented if supported.

For very short requested delays, the fixed overhead may exceed the requested duration; the generator should report that constraint instead of producing misleading output. `SREG` I/O location, availability of instructions, call overhead, and interrupt behavior are target-dependent.

### 6. Achieved delay and error

After emitting an instruction sequence, compute its **actual** executed cycle count, not just the requested target:

$$
T_{\text{actual,ns}}=\frac{C_{\text{actual}}\times10^9}{f_{\text{Hz}}}
$$

$$
\Delta T_{\text{ns}}=T_{\text{actual,ns}}-T_{\text{requested,ns}}
$$

For **up** mode, require `T_actual >= T_requested`. For **nearest** mode, minimize absolute timing error among sequences the generator can actually emit, using a documented tie-breaking rule.

## Proposed architecture

```text
src/
├── lib.rs          # Public API and configuration
├── timing.rs       # Target cycles, instruction timing, achieved delay
├── instruction.rs  # AVR instruction representation
├── planner.rs      # Select counters, nesting, and padding
├── generator.rs    # Assembly emission
└── main.rs         # CLI (future)
```

Pipeline:

1. **Parse** clock, time, rounding mode, and constraints.
2. **Calculate** requested target cycles.
3. **Plan** a feasible delay using instruction templates and permitted registers.
4. **Generate** AVR assembly with register/SREG preservation.
5. **Verify** exact path-dependent cycle totals and report timing error.

## Initial scope

- [ ] Configuration data types and rounding modes
- [ ] Integer-only cycle conversion with checked results
- [ ] Instruction timing metadata for classic AVR
- [ ] 8-bit loop template
- [ ] 16-bit loop template
- [ ] Nested loop template
- [ ] Register and `SREG` preservation
- [ ] Padding for residual cycles
- [ ] Independent cycle-count verification
- [ ] CLI argument parsing and output
- [ ] Boundary and overflow tests

## Assumptions and limitations

- The CPU runs at the specified effective frequency, without prescaler or clock changes during the routine.
- The delay is a **busy-wait**; it occupies CPU execution time.
- Timing assumes **no interrupts or other asynchronous interruptions** while counting. Saving/restoring `SREG` does not itself prevent interrupts.
- A valid stack is required for `PUSH`, `POP`, and `RET`.
- Assembly instruction timing must be checked for the particular target AVR core.
- For precise timing over long durations in deployed firmware, a hardware timer is often preferable.

## Reference

- [Microchip AVR Instruction Set Manual](https://onlinedocs.microchip.com/oxy/GUID-06C36746-0136-4691-8940-69BFD41FCC56-en-US-1/index.html)

## License

TBD.
