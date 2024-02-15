//! We do something really dumb and brute force to manage liveness.
//! Basically, in between every statement, we have the option of making
//! initialized variables be considered moved. Note that this could cause
//! type check errors later on if that variable is used again. In "real life"
//! we would use a liveness check to detect that possibility, but to keep
//! these rules simple, we let the judgments just explore all possibilities.

use std::sync::Arc;

use formality_core::{cast_impl, set, Set, SetExt, Upcast};

use crate::grammar::{Block, Expr, Place, PlaceExpr, Statement, Var};

/// Tracks the set of live variables at a given point in execution.
/// The `Default` impl returns an empty set.
#[derive(Clone, Default, Debug, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub struct LivePlaces {
    vars: Set<Var>,
}

cast_impl!(LivePlaces);

impl LivePlaces {
    /// True if `v` is live -- i.e., may be accessed after this point.
    pub fn is_live(&self, v: impl Upcast<Var>) -> bool {
        self.vars.contains(&v.upcast())
    }

    /// Compute a new set of live-vars just before `term` has been evaluated.
    pub fn before(&self, term: &impl AdjustLiveVars) -> Self {
        let vars = term.adjust_live_vars(self.vars.clone());
        Self { vars }
    }

    /// Compute a new set of live-vars just before `terms` have been evaluated.
    pub fn before_all(&self, terms: impl IntoIterator<Item = impl AdjustLiveVars>) -> Self {
        let vars = terms.into_iter().fold(set![], |vars, term| {
            vars.union_with(term.adjust_live_vars(self.vars.clone()))
        });
        Self { vars }
    }

    /// Compute a new set of live-vars that doesn't include var
    pub fn without(self, var: impl Upcast<Var>) -> Self {
        let vars = self.vars.without_element(&var.upcast());
        Self { vars }
    }

    pub fn vars(&self) -> &Set<Var> {
        &self.vars
    }
}
pub trait AdjustLiveVars: std::fmt::Debug {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var>;
}

impl<T: AdjustLiveVars> AdjustLiveVars for &T {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        T::adjust_live_vars(self, vars)
    }
}

impl<T: AdjustLiveVars> AdjustLiveVars for Arc<T> {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        T::adjust_live_vars(self, vars)
    }
}

impl AdjustLiveVars for Vec<Statement> {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        self.iter()
            .rev()
            .fold(vars, |vars, stmt| stmt.adjust_live_vars(vars))
    }
}

impl AdjustLiveVars for Statement {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        match self {
            Statement::Expr(expr) => expr.adjust_live_vars(vars),
            Statement::Let(var, _ty, expr) => {
                let vars = expr.adjust_live_vars(vars);
                vars.without_element(var)
            }
            Statement::Reassign(place, expr) if place.projections.is_empty() => {
                let vars = expr.adjust_live_vars(vars);
                vars.without_element(&place.var)
            }
            Statement::Reassign(place, expr) => {
                let vars = expr.adjust_live_vars(vars);
                place.adjust_live_vars(vars)
            }
            Statement::Loop(expr) => expr.adjust_live_vars(vars),
            Statement::Break => vars,
            Statement::Return(expr) => expr.adjust_live_vars(vars),
        }
    }
}

impl AdjustLiveVars for Vec<Expr> {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        self.iter()
            .rev()
            .fold(vars, |vars, expr| expr.adjust_live_vars(vars))
    }
}

impl AdjustLiveVars for Expr {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        match self {
            Expr::Block(block) => block.adjust_live_vars(vars),
            Expr::Integer(_) => vars,
            Expr::Add(lhs, rhs) => {
                let vars = rhs.adjust_live_vars(vars);
                lhs.adjust_live_vars(vars)
            }
            Expr::Place(place) => place.adjust_live_vars(vars),
            Expr::Tuple(exprs) => exprs.adjust_live_vars(vars),
            Expr::Call(func, _method_name, _parameters, args) => {
                let vars = args.adjust_live_vars(vars);
                func.adjust_live_vars(vars)
            }
            Expr::New(_ty, _parameters, args) => args.adjust_live_vars(vars),
            Expr::Clear(_) => vars,
            Expr::If(cond, if_true, if_false) => {
                let if_true_vars = if_true.adjust_live_vars(vars.clone());
                let if_false_vars = if_false.adjust_live_vars(vars);
                cond.adjust_live_vars(if_true_vars.union_with(if_false_vars))
            }
        }
    }
}

impl AdjustLiveVars for Block {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        let Block { statements } = self;
        statements.adjust_live_vars(vars)
    }
}

impl AdjustLiveVars for PlaceExpr {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        self.place.adjust_live_vars(vars)
    }
}

impl AdjustLiveVars for Set<Place> {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        self.iter()
            .fold(vars, |vars, place| place.adjust_live_vars(vars))
    }
}

impl AdjustLiveVars for Place {
    fn adjust_live_vars(&self, vars: Set<Var>) -> Set<Var> {
        vars.with_element(&self.var)
    }
}
