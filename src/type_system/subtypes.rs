use formality_core::{judgment_fn, Cons, Set};

use crate::{
    grammar::{NamedTy, Parameter, Perm, Place, Ty},
    type_system::{
        env::Env,
        lien_chains::{lien_chains, ty_chains, Lien, LienChain, My, Our, TyChain},
        lien_set::lien_set_from_chain,
        quantifiers::fold_zipped,
    },
};

judgment_fn! {
    /// Provable if `a <: b` in an owned (`my`) context.
    pub fn sub(
        env: Env,
        a: Parameter,
        b: Parameter,
    ) => Env {
        debug(a, b, env)

        (
            (sub_in_cx(env, My(), a, My(), b) => env)
            ------------------------------- ("sub")
            (sub(env, a, b) => env)
        )
    }
}

judgment_fn! {
    /// Provable if `a <: b` when appearing in the context of lien chains `chain_a` and `chain_b` respectively.
    fn sub_in_cx(
        env: Env,
        chain_a: LienChain,
        a: Parameter,
        chain_b: LienChain,
        b: Parameter,
    ) => Env {
        debug(chain_a, a, chain_b, b, env)

        (
            (ty_chains(&env, liens_a, a) => ty_liens_a)
            (ty_chains(&env, &liens_b, &b) => ty_liens_b)
            (sub_ty_chain_sets(&env, &ty_liens_a, ty_liens_b) => env)
            ------------------------------- ("sub")
            (sub_in_cx(env, liens_a, a: Ty, liens_b, b: Ty) => env)
        )

        (
            (lien_chains(&env, liens_a, a) => liens_a)
            (lien_chains(&env, &liens_b, &b) => liens_b)
            (sub_lien_chain_sets(&env, &liens_a, liens_b) => env)
            ------------------------------- ("sub")
            (sub_in_cx(env, liens_a, a: Perm, liens_b, b: Perm) => env)
        )
    }
}

judgment_fn! {
    fn sub_ty_chain_sets(
        env: Env,
        ty_liens_a: Set<TyChain>,
        ty_liens_b: Set<TyChain>,
    ) => Env {
        debug(ty_liens_a, ty_liens_b, env)

        (
            ------------------------------- ("nil")
            (sub_ty_chain_sets(env, (), _b_s) => env)
        )

        (
            (&b_s => b)
            (sub_ty_chains(&env, &a, &b) => env)
            (sub_ty_chain_sets(env, &a_s, &b_s) => env)
            ------------------------------- ("cons")
            (sub_ty_chain_sets(env, Cons(a, a_s), b_s) => env)
        )
    }
}

judgment_fn! {
    fn sub_ty_chains(
        env: Env,
        ty_chain_a: TyChain,
        ty_chain_b: TyChain,
    ) => Env {
        debug(ty_chain_a, ty_chain_b, env)

        (
            (if a == b)!
            (sub_lien_chains(env, chain_a, chain_b) => env)
            (let layout_a = ty_chain_a.lien_chain().layout())
            (let layout_b = ty_chain_b.lien_chain().layout())
            (if layout_a == layout_b)
            -------------------------------- ("var")
            (sub_ty_chains(env, TyChain::Var(chain_a, a), TyChain::Var(chain_b, b)) => env)
        )

        (
            (let NamedTy { name: name_a, parameters: parameters_a } = a)
            (let NamedTy { name: name_b, parameters: parameters_b } = b)
            (if name_a == name_b)! // FIXME: subtyping between classes
            (sub_lien_chains(env, &chain_a, &chain_b) => env)
            (fold_zipped(env, &parameters_a, &parameters_b, &|env, parameter_a, parameter_b| {
                sub_in_cx(env, &chain_a, parameter_a, &chain_b, parameter_b)
            }) => env)
            (let layout_a = ty_chain_a.lien_chain().layout())
            (let layout_b = ty_chain_b.lien_chain().layout())
            (if layout_a == layout_b) // FIXME: should consider if these are boxed classes
            -------------------------------- ("named ty")
            (sub_ty_chains(env, TyChain::NamedTy(chain_a, a), TyChain::NamedTy(chain_b, b)) => env)
        )
    }
}

judgment_fn! {
    /// Provable if every chain in `chains_a` is a subchain of some chain in `chains_b`.
    fn sub_lien_chain_sets(
        env: Env,
        chains_a: Set<LienChain>,
        chains_b: Set<LienChain>,
    ) => Env {
        debug(chains_a, chains_b, env)

        (
            ------------------------------- ("nil")
            (sub_lien_chain_sets(env, (), _chains_b) => env)
        )

        (
            (&chains_b => chain_b)
            (sub_lien_chains(&env, &chain_a, &chain_b) => env)
            (sub_lien_chain_sets(env, &chains_a, &chains_b) => env)
            ------------------------------- ("cons")
            (sub_lien_chain_sets(env, Cons(chain_a, chains_a), chains_b) => env)
        )
    }
}

judgment_fn! {
    fn sub_lien_chains(
        env: Env,
        a: LienChain,
        b: LienChain,
    ) => Env {
        debug(a, b, env)

        // Special cases for fully owned things

        (
            --------------------------- ("my-*")
            (sub_lien_chains(env, My(), _b) => env)
        )

        (
            --------------------------- ("our-sh")
            (sub_lien_chains(env, Our(), Cons(Lien::Shared(_), _)) => env)
        )

        (
            (lien_covered_by(lien_a, lien_b) => ())
            (sub_lien_chain_exts(&env, &chain_a, &chain_b) => env)
            --------------------------- ("matched starts")
            (sub_lien_chains(env, Cons(lien_a, chain_a), Cons(lien_b, chain_b)) => &env)
        )
    }
}
judgment_fn! {
    fn sub_lien_chain_exts(
        env: Env,
        a: LienChain,
        b: LienChain,
    ) => Env {
        debug(a, b, env)

        (
            (lien_set_from_chain(&env, &chain_a) => lien_set_a)
            (lien_set_from_chain(&env, &chain_b) => lien_set_b)
            (lien_set_covered_by(&lien_set_a, lien_set_b) => ())
            (lien_chain_covered_by(&chain_a, &chain_b) => ())
            --------------------------- ("chain-chain")
            (sub_lien_chain_exts(env, chain_a, chain_b) => &env)
        )
    }
}

judgment_fn! {
    fn lien_set_covered_by(
        a: Set<Lien>,
        b: Set<Lien>,
    ) => () {
        debug(a, b)

        (
            ------------------------------- ("nil")
            (lien_set_covered_by((), _b) => ())
        )

        (
            (&b_s => b)
            (lien_covered_by(&a, b) => ())
            (lien_set_covered_by(&a_s, &b_s) => ())
            ------------------------------- ("cons")
            (lien_set_covered_by(Cons(a, a_s), b_s) => ())
        )
    }
}

judgment_fn! {
    fn lien_chain_covered_by(
        a: LienChain,
        b: LienChain,
    ) => () {
        debug(a, b)

        (
            (lien_chain_covered_by(chain_a, chain_b) => ())
            ------------------------------- ("skip lease prefix")
            (lien_chain_covered_by(Cons(Lien::Leased(_), chain_a), chain_b) => ())
        )

        (
            (lien_chain_strictly_covered_by(chain_a, chain_b) => ())
            ------------------------------- ("strictly covered")
            (lien_chain_covered_by(chain_a, chain_b) => ())
        )
    }
}

judgment_fn! {
    fn lien_chain_strictly_covered_by(
        a: LienChain,
        b: LienChain,
    ) => () {
        debug(a, b)

        (
            ------------------------------- ("my-my")
            (lien_chain_strictly_covered_by(My(), My()) => ())
        )

        (
            (lien_covered_by(lien_a, lien_b) => ())
            (lien_chain_strictly_covered_by(&chain_a, &chain_b) => ())
            ------------------------------- ("lien-lien")
            (lien_chain_strictly_covered_by(Cons(lien_a, chain_a), Cons(lien_b, chain_b)) => ())
        )
    }
}

judgment_fn! {
    fn lien_covered_by(
        a: Lien,
        b: Lien,
    ) => () {
        debug(a, b)

        (
            ------------------------------- ("our-our")
            (lien_covered_by(Lien::Our, Lien::Our) => ())
        )

        (
            (if place_covered_by_place(&a, &b))
            ------------------------------- ("lease-lease")
            (lien_covered_by(Lien::Leased(a), Lien::Leased(b)) => ())
        )

        (
            (if place_covered_by_place(&a, &b))
            ------------------------------- ("shared-shared")
            (lien_covered_by(Lien::Shared(a), Lien::Shared(b)) => ())
        )

        (
            (if a == b)
            ------------------------------- ("var-var")
            (lien_covered_by(Lien::Var(a), Lien::Var(b)) => ())
        )
    }
}

/// A place P1 *covers* a place P2 if it is a prefix:
/// for example, `x.y` covers `x.y` and `x.y.z` but not `x.z` or `x1`.
#[tracing::instrument(level = "Debug", ret)]
fn place_covered_by_place(place: &Place, covering_place: &Place) -> bool {
    covering_place.is_prefix_of(place)
}

#[cfg(test)]
mod tests;
