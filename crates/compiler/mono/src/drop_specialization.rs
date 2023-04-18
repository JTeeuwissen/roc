// This program was written by Jelle Teeuwissen within a final
// thesis project of the Computing Science master program at Utrecht
// University under supervision of Wouter Swierstra (w.s.swierstra@uu.nl).

// Implementation based of Drop Specialization from Perceus: Garbage Free Reference Counting with Reuse
// https://www.microsoft.com/en-us/research/uploads/prod/2021/06/perceus-pldi21.pdf

#![allow(clippy::too_many_arguments)]

use bumpalo::collections::vec::Vec;
use roc_module::low_level::{LowLevel, LowLevel::*};
use roc_module::symbol::{IdentIds, ModuleId, Symbol};
use roc_target::PtrWidth;

use crate::borrow::Ownership;
use crate::ir::{
    BranchInfo, Call, CallType, Expr, JoinPointId, Literal, ModifyRc, Param, Proc, ProcLayout,
    Stmt, UpdateModeId, UpdateModeIds,
};
use crate::layout::{
    Builtin, InLayout, Layout, LayoutInterner, STLayoutInterner, TagIdIntType, UnionLayout,
};

use bumpalo::Bump;

use roc_can::module;
use roc_collections::{MutMap, MutSet};

/**
Try to find increments of symbols followed by decrements of the symbol they were indexed out of (their parent).
Then inline the decrement operation of the parent and removing matching pairs of increments and decrements.
*/
pub fn specialize_drops<'a, 'i>(
    arena: &'a Bump,
    layout_interner: &'i STLayoutInterner<'a>,
    home: ModuleId,
    ident_ids: &'i mut IdentIds,
    update_mode_ids: &'i mut UpdateModeIds,
    procs: &mut MutMap<(Symbol, ProcLayout<'a>), Proc<'a>>,
) {
    for ((symbol, _layout), proc) in procs.iter_mut() {
        // Clone the symbol_rc_types_env and insert the symbol
        specialize_drops_proc(arena, proc);
    }
}

pub fn specialize_drops_proc<'a>(arena: &'a Bump, proc: &mut Proc<'a>) {
    let new_body = specialize_drops_stmt(
        arena,
        &mut DropSpecializationEnvironment {
            arena,
            indexed_child_of: MutMap::default(),
            incremented_children: MutMap::default(),
        },
        &proc.body,
    );

    proc.body = new_body.clone();
}

pub fn specialize_drops_stmt<'a>(
    arena: &'a Bump,
    environment: &mut DropSpecializationEnvironment,
    stmt: &'a Stmt<'a>,
) -> &'a Stmt<'a> {
    match stmt {
        Stmt::Let(binding, expr, layout, continuation) => match expr {
            Expr::Call(_) => todo!(),
            Expr::Tag {
                tag_layout,
                tag_id,
                arguments,
            } => todo!(),
            Expr::Struct(_) => todo!(),
            Expr::StructAtIndex {
                index,
                field_layouts,
                structure,
            } => {
                environment.add_index(*structure, *binding, *index);
                todo!()
            }
            Expr::GetTagId {
                structure,
                union_layout,
            } => todo!(),
            Expr::UnionAtIndex {
                structure,
                tag_id,
                union_layout,
                index,
            } => todo!(),
            Expr::Array { elem_layout, elems } => todo!(),
            Expr::EmptyArray => todo!(),
            Expr::ExprBox { symbol } => todo!(),
            Expr::ExprUnbox { symbol } => todo!(),
            Expr::Reuse {
                symbol,
                update_tag_id,
                update_mode,
                tag_layout,
                tag_id,
                arguments,
            } => todo!(),
            Expr::Reset {
                symbol,
                update_mode,
            } => todo!(),
            Expr::ResetRef {
                symbol,
                update_mode,
            } => todo!(),
            Expr::RuntimeErrorFunction(_) => todo!(),
            Expr::NullPointer | Expr::Literal(_) => todo!("Not relevant"),
        },
        Stmt::Switch {
            cond_symbol,
            cond_layout,
            branches,
            default_branch,
            ret_layout,
        } => todo!(),
        Stmt::Ret(_) => todo!(),
        Stmt::Refcounting(rc, continuation) => match rc {
            ModifyRc::Inc(_, _) => todo!("Look for previous decrements of children"),
            ModifyRc::Dec(_) => todo!("Insert info to use at increment."),
            ModifyRc::DecRef(_) => {
                todo!("Inlining has no point, since it doesn't decrement it's children")
            }
        },
        Stmt::Expect {
            condition,
            region,
            lookups,
            variables,
            remainder,
        } => todo!(),
        Stmt::ExpectFx {
            condition,
            region,
            lookups,
            variables,
            remainder,
        } => todo!(),
        Stmt::Dbg {
            symbol,
            variable,
            remainder,
        } => todo!(),
        Stmt::Join {
            id,
            parameters,
            body,
            remainder,
        } => todo!(),
        Stmt::Jump(_, _) => todo!(),
        Stmt::Crash(_, _) => todo!(),
    }
}

type Index = u64;

#[derive(Clone)]
struct DropSpecializationEnvironment<'a> {
    arena: &'a Bump,

    // Keeps track of which symbol is an index of which symbol.
    indexed_child_of: MutMap<Symbol, (Symbol, Index)>,

    // Keeps track of the incremented indexed children of a symbol.
    incremented_children: MutMap<Symbol, Vec<'a, (Symbol, Index)>>,
}

impl<'a> DropSpecializationEnvironment<'a> {
    fn add_index(&mut self, parent: Symbol, child: Symbol, index: Index) {
        let old_value = self.indexed_child_of.insert(child, (parent, index));
        debug_assert!(
            old_value.is_none(),
            "The same child should only be bound once."
        );
    }

    fn add_incremented(&mut self, symbol: Symbol) {
        match self.indexed_child_of.get(&symbol) {
            Some((parent, index)) => self
                .incremented_children
                .entry(*parent)
                .or_insert(Vec::new_in(self.arena))
                .push((symbol, *index)),
            None => {
                // Value not indexed, so we can ignore it.
            }
        }
    }

    // TODO assert that a parent is only inlined once / assert max single dec per parent.
}

/**
Free the memory of a symbol
*/
fn free<'a>(arena: &'a Bump, symbol: Symbol, continuation: &'a Stmt<'a>) -> &'a Stmt<'a> {
    // Currently using decref, but this checks the uniqueness of the symbol.
    // This function should only be called if it is known to be unique.
    // So instead this can be replaced with a free instruction that does not check the uniqueness.
    arena.alloc(Stmt::Refcounting(ModifyRc::DecRef(symbol), continuation))
}

fn branch_drop_unique<'a>(
    arena: &'a Bump,
    dropped: Symbol,
    continuation: &'a Stmt<'a>,
) -> &'a Stmt<'a> {
    /**
     * if unique xs
     *    then drop x; drop xs; free xs
     *    else decref xs
     */
    let unique_symbol = todo!("unique_symbol");

    let joinpoint_id = todo!("joinpoint_id");
    let jump = arena.alloc(Stmt::Jump(
        joinpoint_id,
        Vec::with_capacity_in(0, arena).into_bump_slice(),
    ));

    let unique_branch = arena.alloc(free(arena, dropped, jump));
    let non_unique_branch = arena.alloc(Stmt::Refcounting(ModifyRc::DecRef(dropped), jump));

    let branching = arena.alloc(Stmt::Switch {
        cond_symbol: unique_symbol,
        cond_layout: Layout::BOOL,
        branches: todo!(),
        default_branch: todo!(),
        ret_layout: todo!(),
    });

    let condition = arena.alloc(Stmt::Let(
        unique_symbol,
        Expr::Call(Call {
            call_type: CallType::LowLevel {
                op: Eq,
                update_mode: UpdateModeId::BACKEND_DUMMY,
            },
            arguments: arena.alloc_slice_copy(arguments),
        }),
        Layout::BOOL,
        branching,
    ));

    arena.alloc(Stmt::Join {
        id: joinpoint_id,
        parameters: Vec::with_capacity_in(0, arena).into_bump_slice(),
        body: continuation,
        remainder: condition,
    })
}
fn branch_drop_reuse_unique<'a>(arena: &'a Bump) {}

// TODO Figure out when unionindexes are inserted (just after a pattern match?)
// TODO Always index out all children (perhaps move all dup to after all indexing)
// TODO Remove duplicate indexes
// TODO Lowlevel is unqiue check
// TODO joinpoint split on isunique
// TODO insert decref, reuse token reference (raw pointer), free (decref until then.).
