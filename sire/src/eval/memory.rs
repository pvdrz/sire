#![allow(rustc::default_hash_types)]
use std::collections::HashMap;

use rustc::mir::interpret::InterpResult;
use rustc::mir::*;
use rustc::{err_unsup, err_unsup_format};

use crate::sir::*;

#[derive(Default, Clone)]
pub struct Memory<'tcx> {
    map: HashMap<Place<'tcx>, Expr>,
}

impl<'tcx> Memory<'tcx> {
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get(&self, place: &Place<'tcx>) -> InterpResult<'tcx, &Expr> {
        self.map
            .get(place)
            .ok_or_else(|| err_unsup_format!("Cannot get from place {:?}", place).into())
    }

    pub fn get_mut(&mut self, place: &Place<'tcx>) -> InterpResult<'tcx, &mut Expr> {
        self.map
            .get_mut(place)
            .ok_or_else(|| err_unsup_format!("Cannot get from place {:?}", place).into())
    }

    pub fn insert(&mut self, place: Place<'tcx>, expr: Expr) {
        self.map.insert(place, expr);
    }

    pub fn insert_from_int(&mut self, int: usize, expr: Expr) {
        self.insert(Local::from_usize(int).into(), expr)
    }

    pub fn remove(&mut self, place: &Place<'tcx>) -> InterpResult<'tcx, Expr> {
        self.map
            .remove(place)
            .ok_or_else(|| err_unsup_format!("Cannot remove from place {:?}", place).into())
    }

    pub fn remove_from_int(&mut self, int: usize) -> InterpResult<'tcx, Expr> {
        self.remove(&Local::from_usize(int).into())
    }
}
