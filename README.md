# Sire
Sire (which is a WIP) intends to be an small symbolic evaluator for Rust's MIR.

## How does it work?

Sire takes the optimized MIR of your code and evaluates its functions into small [expressions](https://github.com/christianpoveda/sire/blob/8b8a9f94398ac68b3b2b2b902c7980b3f0d7e647/src/interpreter.rs#L10). It also allows to export such expressions to the [smt-lib](http://smtlib.cs.uiowa.edu/) language, then you can reason more about them using a theorem prover.
So for example if you have a file `some_code.rs` containing:
```rust
fn main() {

}

fn sum(n: u64, m: u64) -> u64 {
    if m > 0 {
        sum(n + 1, m - 1)
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
(declare-fun sum ((_ BitVec 64) (_ BitVec 64)) (_ BitVec 64))
(assert (forall ((x1 (_ BitVec 64)) (x2 (_ BitVec 64))) (= (sum x1 x2) (ite (bvugt x2 (_ bv0 64)) (sum (bvadd x1 (_ bv1 64)) (bvsub x2 (_ bv1 64))) x1))))
```
you can use this code to reason about the `sum` function using [z3](https://rise4fun.com/Z3/sl8wn) for example.

## Coverage

Right now, just an small set of Rust functions can be evaluated with Sire (basically any recursive function without side effects nor loops) and I am working to expand this. To be more specific, the following are allowed:

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

Additionally, just the integer (both signed and unsigned) and boolean types are supported.

If you have any suggestions or questions feel free to open an issue/write me an email :)

## Installing

This project depends on nightly Rust, the preferred (only?) method is using
[`rustup`](https://rustup.rs/). Please check the `rustup` documentation on how
to get nightly. Now you will need to clone this repository:

```bash
$ git clone https://github.com/christianpoveda/sire 
```

Now to execute a `code.rs` file using `sire`, run the following inside the
repository folder

```bash
cargo run code.rs -O
```

This should throw a symbolic representation of every function in `code.rs` and
its `smt-lib` counterpart.
