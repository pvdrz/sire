#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate syntax;

mod analysis;
mod interpreter;
mod lang;
mod smt;

use crate::interpreter::Interpreter;
use crate::smt::ToSmt;

use rustc_driver::{report_ices_to_stderr_if_any, run_compiler, Callbacks, Compilation};
use rustc_interface::interface;

fn find_sysroot() -> String {
    let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
    let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));

    match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
        _ => option_env!("RUST_SYSROOT")
            .expect("could not find sysroot")
            .to_owned(),
    }
}

struct SireCompilerCalls;

impl Callbacks for SireCompilerCalls {
    fn after_parsing(&mut self, _compiler: &interface::Compiler) -> Compilation {
        Compilation::Continue
    }

    fn after_analysis(&mut self, compiler: &interface::Compiler) -> Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let functions = Interpreter::from_tcx(tcx).unwrap().eval_all().unwrap();
            for func in functions {
                println!("{}", func);
                println!("{}", func.to_smt());
            }
        });

        compiler.session().abort_if_errors();
        Compilation::Stop
    }
}

fn main() {
    let mut rustc_args = std::env::args().collect::<Vec<_>>();
    let sysroot_flag = String::from("--sysroot");

    if !rustc_args.contains(&sysroot_flag) {
        rustc_args.push(sysroot_flag);
        rustc_args.push(find_sysroot());
    }

    let result = report_ices_to_stderr_if_any(move || {
        run_compiler(&rustc_args, &mut SireCompilerCalls, None, None)
    })
    .and_then(|result| result);

    std::process::exit(result.is_err() as i32);
}
