# Sire
Sire (which is a WIP) intends to be an small symbolic evaluator for Rust's MIR.

## How?

Sire takes the optimized MIR of your code and evaluates its functions into small [expressions](https://github.com/christianpoveda/sire/blob/8b8a9f94398ac68b3b2b2b902c7980b3f0d7e647/src/interpreter.rs#L10). It also allows to export such expressions to the [smt-lib](http://smtlib.cs.uiowa.edu/) language, then you can reason more about them using a theorem prover.
So for example if you have a file `some_code.rs` containing:
```rust
fn main() {

}

fn sum(n: i64, m: i64) -> i64 {
    if m > 0 {
        sum(n + 1, m - 1)
    } else if m < 0 {
        sum(n - 1, m + 1)
    } else {
        n
    } 
}
```
you can evaluate it cloning this repo and running
```bash
$ cargo run some_code.rs -C opt-level=3
```
then Sire should print something like 
```
(declare-fun sum (Int Int) Int)
(assert (forall ((x1 Int) (x2 Int)) (= (sum x1 x2) (ite (> x2 0) (sum (+ x1 1) (- x2 1)) (ite (< x2 0) (sum (- x1 1) (+ x2 1)) x1)))))
```
Then you can use this code to reason about the `sum` function using [z3](https://rise4fun.com/Z3/F0Qk) for example.

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

