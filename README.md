# Sire
Sire (which is a WIP) intends to be an small symbolic evaluator for Rust's MIR.

## How?

Sire takes the optimized MIR of your code and evaluates its functions into small [expressions](https://github.com/christianpoveda/sire/blob/8b8a9f94398ac68b3b2b2b902c7980b3f0d7e647/src/interpreter.rs#L10). It also allows to export such expressions to the [smt-lib](http://smtlib.cs.uiowa.edu/) language, then you can reason more about them using a theorem prover.
So for example if you have a file `some_code.rs` containing:
```rust
fn fact(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        n * fact(n - 1)
    }
}
```
you can evaluate it cloning this repo and running
```bash
$ cargo run some_code.rs -C opt-level=3
```
then Sire should print something like 
```
fact = (ite (= x1 0) 1 (* x1 (fact (- x1 1))))
```
Such expressions can be used to reason about the `fact` function using [z3](https://github.com/Z3Prover/z3) for example.

## Coverage

Right now, just an small set of Rust functions can be evaluated with Sire (basically any recursive function without side effects) and I am working to expand this. To be more specific, the following are allowed:

- Statements:
    - `Assign`
    - `StorageLive`
    - `StorageDead`

- Terminators:
    - `Return`
    - `Goto`
    - `Call` (only if the function returns)
    - `SwitchInt`

- Rvalues:
    - `BinaryOp`
    - `Ref` (only shared references)
    - `Use`

- Operands:
    - `Move` and `Copy`
    - `Constant` (only scalars)

Additionaly, just the `i64`, `u64` and `bool` types are supported.

If you have any suggestions or questions feel free to open an issue/write me an email :)

