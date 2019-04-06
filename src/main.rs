#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate syntax;

mod interpreter;
mod smt;
mod analysis;

use crate::interpreter::{Interpreter, Ty, Value};
use crate::smt::ToSmt;

use std::collections::HashMap;

use rustc::hir::def_id::LOCAL_CRATE;
use rustc::hir::ItemKind;
use rustc::ty::TyKind;
use rustc_driver::{report_ices_to_stderr_if_any, run_compiler, Callbacks};
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
    fn after_parsing(&mut self, _compiler: &interface::Compiler) -> bool {
        true
    }

    fn after_analysis(&mut self, compiler: &interface::Compiler) -> bool {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let hir = tcx.hir();
            let mut mirs = HashMap::new();
            let mut funcs = HashMap::new();
            let mut def_ids = Vec::new();

            let (entry_def_id, _) = tcx.entry_fn(LOCAL_CRATE).expect("no main function found!");

            for (node_id, item) in &hir.krate().items {
                if let ItemKind::Fn(_, _, _, _) = item.node {
                    let def_id = hir.local_def_id(*node_id);
                    if def_id != entry_def_id {
                        let name = tcx.def_path(def_id).to_filename_friendly_no_crate();
                        let mir = tcx.optimized_mir(def_id);
                        let args_ty = mir
                            .local_decls
                            .iter()
                            .take(mir.arg_count + 1)
                            .map(|local_decl| match local_decl.ty.sty {
                                TyKind::Bool => Ty::Bool,
                                TyKind::Int(int_ty) => Ty::Int(int_ty.bit_width().unwrap_or(64)),
                                TyKind::Uint(uint_ty) => {
                                    Ty::Uint(uint_ty.bit_width().unwrap_or(64))
                                }
                                _ => unimplemented!(),
                            })
                            .collect::<Vec<Ty>>();

                        mirs.insert(def_id, mir);
                        funcs.insert(def_id, Value::Function(name, Ty::Func(args_ty)));
                        def_ids.push(def_id);
                    }
                }
            }

            let mut interpreter = Interpreter::new(funcs, mirs);
            for def_id in def_ids {
                let result = interpreter.eval_mir(def_id).unwrap();
                println!("{}", result.to_smt());
            }
        });

        compiler.session().abort_if_errors();
        false
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
