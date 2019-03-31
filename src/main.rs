#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_mir;

use rustc::hir::def_id::LOCAL_CRATE;
use rustc::ty;
use rustc_interface::interface;

use rustc_mir::interpret::EvalContext;

struct SireCompilerCalls;

impl rustc_driver::Callbacks for SireCompilerCalls {
    fn after_parsing(&mut self, compiler: &interface::Compiler) -> bool {
        println!("We arrived after parsing");
        true
    }

    fn after_analysis(&mut self, compiler: &interface::Compiler) -> bool {
        println!("We arrived after analysis");
        compiler.session().abort_if_errors();

        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let (main_id, _) = tcx.entry_fn(LOCAL_CRATE).expect("no main function found!");

            let main_instance = ty::Instance::mono(tcx, main_id);
            println!("{:?}", main_instance);
        });

        compiler.session().abort_if_errors();
        false
    }
}

fn find_sysroot() -> String {
    if let Ok(sysroot) = std::env::var("SIRE_SYSROOT") {
        return sysroot;
    }

    // Taken from PR <https://github.com/Manishearth/rust-clippy/pull/911>.
    let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
    let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
    match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
        _ => option_env!("RUST_SYSROOT")
            .expect(
                "could not find sysroot. Either set `MIRI_SYSROOT` at run-time, or at \
                 build-time specify `RUST_SYSROOT` env var or use rustup or multirust",
            )
            .to_owned(),
    }
}

fn main() {
    let mut rustc_args = std::env::args().collect::<Vec<_>>();
    let sysroot_flag = String::from("--sysroot");

    if !rustc_args.contains(&sysroot_flag) {
        rustc_args.push(sysroot_flag);
        rustc_args.push(find_sysroot());
    }

    let result = rustc_driver::report_ices_to_stderr_if_any(move || {
        rustc_driver::run_compiler(&rustc_args, &mut SireCompilerCalls, None, None)
    })
    .and_then(|result| result);

    std::process::exit(result.is_err() as i32);
}
