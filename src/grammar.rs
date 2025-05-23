pub use crate::dada_lang::grammar::*;
use crate::dada_lang::FormalityLang;
use formality_core::{term, Downcast, Fallible, Set, Upcast, UpcastFrom};
use std::sync::Arc;

mod cast_impls;

#[cfg(test)]
mod test_parse;

#[term($*decls)]
pub struct Program {
    pub decls: Vec<Decl>,
}

impl Program {
    pub fn class_named(&self, name: &ValueId) -> Fallible<&ClassDecl> {
        self.decls
            .iter()
            .filter_map(|d| d.as_class_decl())
            .filter(|d| d.name == *name)
            .next()
            .ok_or_else(|| anyhow::anyhow!("no class named `{:?}`", name))
    }
}

#[term]
pub enum Decl {
    #[cast]
    ClassDecl(ClassDecl),
}

/// Class predicates categorize classes according to how they
/// can be used. The eventual hierarchy will be
///
/// * `tracked class` -- true linear type that must be moved (not yet fully designed)
/// * `guard class` -- affine type that must be dropped
/// * `class` -- the default, a class that whose fields can be mutated
/// * `our class` -- a value type that is always considered shared
///
/// In all cases class predicates exist modulo generics.
///
/// Ordering is significant here.
#[term]
#[derive(Copy, Default)]
pub enum ClassPredicate {
    /// `Guard` classes are permitted to have destructors (FIXME: we don't model those right now).
    /// A `Guard` class cannot be shared and, since they have a destructor, we cannot drop them
    /// from borrow chains (i.e., `mut[guard] mut[data]` cannot be converted to `mut[data]`
    /// even if `guard` is not live since, in fact, the variable *will* be used again by the dtor).
    Guard,

    /// `Share` classes are the default. They indicate classes that, while unique by default,
    /// can be shared with `.share` to create an `our Class` that is copyable around.
    #[default]
    Share,

    /// `Our` classes are called `struct` in surface syntax, they indicate classes
    /// (by default) are shared and hence can be copied freely. However, their fields
    /// cannot be individually mutated as a result.
    Our,
}

impl ClassPredicate {
    pub fn apply(self, parameter: impl Upcast<Parameter>) -> Predicate {
        Predicate::class(self, parameter)
    }
}

// ANCHOR: ClassDecl
#[term($?class_predicate class $name $binder)]
pub struct ClassDecl {
    pub name: ValueId,
    pub class_predicate: ClassPredicate,
    pub binder: Binder<ClassDeclBoundData>,
}

#[term($:where $,predicates { $*fields $*methods })]
pub struct ClassDeclBoundData {
    pub predicates: Vec<Predicate>,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<MethodDecl>,
}
// ANCHOR_END: ClassDecl

// ANCHOR: FieldDecl
#[term($?atomic $name : $ty ;)]
pub struct FieldDecl {
    pub atomic: Atomic,
    pub name: FieldId,
    pub ty: Ty,
}
// ANCHOR_END: FieldDecl

// ANCHOR: Atomic
#[term]
#[derive(Default)]
pub enum Atomic {
    #[default]
    #[grammar(nonatomic)]
    No,

    #[grammar(atomic)]
    Yes,
}
// ANCHOR_END: Atomic

#[term(fn $name $binder)]
pub struct MethodDecl {
    pub name: MethodId,
    pub binder: Binder<MethodDeclBoundData>,
}

// FIXME: need to guard `$inputs` by a comma and output by `->`, using customized parse
#[term(($this $,inputs) -> $output $:where $,predicates $body)]
#[customize(parse)]
pub struct MethodDeclBoundData {
    pub this: ThisDecl,
    pub inputs: Vec<LocalVariableDecl>,
    pub output: Ty,
    pub predicates: Vec<Predicate>,
    pub body: MethodBody,
}
mod method_impls;

#[term]
pub enum MethodBody {
    #[grammar( ...;)]
    Trusted,

    #[cast]
    Block(Block),
}

#[term($perm self)]
pub struct ThisDecl {
    pub perm: Perm,
}

#[term($name : $ty)]
pub struct LocalVariableDecl {
    pub name: ValueId,
    pub ty: Ty,
}

#[term({ $*statements })]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[term]
pub enum Statement {
    #[grammar($v0 ;)]
    #[cast]
    Expr(Expr),

    #[grammar(let $v0 $?v1 = $v2 ;)]
    Let(ValueId, Ascription, Arc<Expr>),

    #[grammar($v0 = $v1 ;)]
    Reassign(Place, Expr),

    #[grammar(loop { $v0 })]
    Loop(Arc<Expr>),

    #[grammar(break ;)]
    Break,

    #[grammar(return $v0 ;)]
    Return(Expr),
}

#[term]
#[derive(Default)]
pub enum Ascription {
    #[default]
    NoTy,

    #[grammar(: $v0)]
    #[cast]
    Ty(Ty),
}

#[term]
pub enum Expr {
    #[cast]
    Block(Block),

    #[grammar($v0)]
    Integer(usize),

    #[grammar($v0 + $v1)]
    #[precedence(0)]
    Add(Arc<Expr>, Arc<Expr>),

    #[cast]
    Place(PlaceExpr),

    #[grammar($v0.share)]
    Share(Arc<Expr>),

    #[grammar(($*v0))]
    Tuple(Vec<Expr>),

    #[grammar($v0 . $v1 $[?v2] $(v3))]
    Call(Arc<Expr>, MethodId, Vec<Parameter>, Vec<Expr>),

    #[grammar(new $v0 $[?v1] $(v2))]
    New(ValueId, Vec<Parameter>, Vec<Expr>),

    #[grammar($$clear($v0))]
    Clear(ValueId),

    #[grammar(if $v0 $v1 else $v2)]
    If(Arc<Expr>, Arc<Expr>, Arc<Expr>),

    /// `!` panics the progarm, but it's main purpose is to simplify writing tests by allowing us
    /// to produce a value of any type. `!` can only be used in places where we have an expected type from context.
    #[grammar(!)]
    Panic,
}

#[term]
#[derive(Copy, Default)]
pub enum Access {
    #[default]
    #[grammar(ref)]
    Rf,

    #[grammar(share)]
    Sh,

    #[grammar(move)]
    Mv,

    #[grammar(mut)]
    Mt,

    #[grammar(drop)]
    Drop,
}

impl Access {
    pub fn move_to_drop(self) -> Self {
        match self {
            Access::Sh | Access::Rf | Access::Mt => self,
            Access::Mv | Access::Drop => Access::Drop,
        }
    }
}

#[term($place . $access)]
pub struct PlaceExpr {
    pub place: Place,
    pub access: Access,
}

#[term]
pub enum Kind {
    Ty,
    Perm,
}

impl Copy for Kind {}

#[term]
pub enum Parameter {
    #[cast]
    Ty(Ty),

    #[cast]
    Perm(Perm),
}

impl Parameter {
    pub fn is_var(&self, v: impl Upcast<Variable>) -> bool {
        let Some(u) = self.downcast::<Variable>() else {
            return false;
        };

        let v: Variable = v.upcast();
        v == u
    }
}

impl formality_core::language::HasKind<FormalityLang> for Parameter {
    fn kind(&self) -> formality_core::language::CoreKind<FormalityLang> {
        match self {
            Parameter::Ty(_) => Kind::Ty,
            Parameter::Perm(_) => Kind::Perm,
        }
    }
}

#[term]
pub enum Ty {
    #[cast]
    NamedTy(NamedTy),

    #[variable(Kind::Ty)]
    Var(Variable),

    #[grammar($v0 $v1)]
    ApplyPerm(Perm, Arc<Ty>),
}

impl Ty {
    pub fn unit() -> Ty {
        NamedTy {
            name: TypeName::Tuple(0),
            parameters: vec![],
        }
        .upcast()
    }

    pub fn int() -> Ty {
        NamedTy {
            name: TypeName::Int,
            parameters: vec![],
        }
        .upcast()
    }

    pub fn tuple(parameters: impl Upcast<Vec<Ty>>) -> Ty {
        let parameters: Vec<Ty> = parameters.upcast();
        NamedTy {
            name: TypeName::Tuple(parameters.len()),
            parameters: parameters.upcast(),
        }
        .upcast()
    }

    pub fn strip_perm(&self) -> Ty {
        match self {
            Ty::NamedTy(_) | Ty::Var(_) => self.clone(),
            Ty::ApplyPerm(_, ty) => ty.strip_perm(),
        }
    }
}
pub mod ty_impls;

#[term]
pub enum Perm {
    #[grammar(my)]
    My,

    #[grammar(our)]
    Our,

    #[grammar(moved $[?v0])]
    Mv(Set<Place>),

    #[grammar(ref $[?v0])]
    Rf(Set<Place>),

    #[grammar(mut $[v0])]
    Mt(Set<Place>),

    #[variable(Kind::Perm)]
    Var(Variable),

    #[grammar($v0 $v1)]
    Apply(Arc<Perm>, Arc<Perm>),
}
pub mod perm_impls;

#[term($name $[?parameters])]
#[customize(parse, debug)]
pub struct NamedTy {
    pub name: TypeName,
    pub parameters: Parameters,
}
mod named_ty_impls;

#[term]
pub enum TypeName {
    Tuple(usize),

    #[grammar(Int)]
    Int,

    #[cast]
    Id(ValueId),
}

pub type Parameters = Vec<Parameter>;

#[term($var $*projections)]
pub struct Place {
    pub var: Var,
    pub projections: Vec<Projection>,
}

impl Place {
    /// True if `self` is a prefix of `place`.
    pub fn is_overlapping_with(&self, place: &Place) -> bool {
        self.is_prefix_of(place) || place.is_prefix_of(self)
    }

    /// True if `self` is disjoint from `place`.
    pub fn is_disjoint_from(&self, place: &Place) -> bool {
        !self.is_overlapping_with(place)
    }

    /// True if self is a prefix of `place` (and not equal to it).
    pub fn is_strict_prefix_of(&self, place: &Place) -> bool {
        self != place && self.is_prefix_of(place)
    }

    /// True if self is a prefix of `place` (or equal to it).
    pub fn is_prefix_of(&self, place: &Place) -> bool {
        self.var == place.var
            && self.projections.len() <= place.projections.len()
            && self
                .projections
                .iter()
                .zip(&place.projections)
                .all(|(p1, p2)| p1 == p2)
    }

    pub fn project(&self, projection: impl Upcast<Projection>) -> Place {
        let projection = projection.upcast();
        Place {
            var: self.var.clone(),
            projections: self
                .projections
                .iter()
                .chain(std::iter::once(&projection))
                .cloned()
                .collect(),
        }
    }

    /// Returns all "strict prefixes" of this place -- so for `foo.bar.baz`,
    /// returns `[foo, foo.bar]` (but not `foo.bar.baz`).
    pub fn strict_prefixes(&self) -> Vec<Place> {
        (0..self.projections.len())
            .map(|i| Place {
                var: self.var.clone(),
                projections: self.projections[..i].to_vec(),
            })
            .collect()
    }

    /// Returns this place but without one layer of projection (e.g., given `a.b.c` returns `a.b`).
    /// If the place has just a variable (e.g., `a`), returns `None`.
    pub fn owner(&self) -> Option<Place> {
        self.owner_field().map(|pair| pair.0)
    }

    /// Returns this place but without one layer of projection (e.g., given `a.b.c` returns `a.b`).
    /// If the place has just a variable (e.g., `a`), returns `None`.
    pub fn owner_field(&self) -> Option<(Place, Projection)> {
        if let Some((f, p)) = self.projections.split_last() {
            Some((
                Place {
                    var: self.var.clone(),
                    projections: p.to_vec(),
                },
                f.clone(),
            ))
        } else {
            None
        }
    }
}

impl UpcastFrom<Var> for Place {
    fn upcast_from(var: Var) -> Self {
        Place {
            var,
            projections: vec![],
        }
    }
}

#[term]
pub enum Projection {
    #[grammar(. $v0 $!)]
    Field(FieldId),
}

#[term]
pub enum Var {
    #[grammar(self)]
    This,

    /// Special variable (nameable by user) representing
    /// the return value of a function.
    #[grammar(return)]
    Return,

    /// A special variable used only in the type-system.
    /// Represents a value that is in the process of being moved.
    #[grammar(@ in_flight)]
    InFlight,

    /// Fresh values introduced during type check
    #[grammar(@ fresh($v0))]
    Fresh(usize),

    #[cast]
    Id(ValueId),
}

impl Var {
    pub fn dot(&self, f: impl Upcast<FieldId>) -> Place {
        Place {
            var: self.clone(),
            projections: vec![Projection::field(f)],
        }
    }
}

formality_core::id!(BasicBlockId);
formality_core::id!(ValueId);
formality_core::id!(FieldId);
formality_core::id!(MethodId);

/// Predicates:
///
/// # Permission predicates
///
/// The following predices divide permissions into categories
/// (written with *emphasis*):
///
/// |         | *Move*      | *Copy*      |
/// | ---     | ---         | ---         |
/// | *Owned* | `my`        | `our`       |
/// | *Lent*  | `mut[_]` | `ref[_]` |
///
/// There are also *leased* and *shared* predicates for the
/// `leased` and `shared` permissions.
#[term]
pub enum Predicate {
    #[grammar($v0($v1))]
    Parameter(ParameterPredicate, Parameter),

    #[grammar($v0($v1))]
    Class(ClassPredicate, Parameter),

    #[grammar($v0($v1))]
    Variance(VarianceKind, Parameter),
}

impl Predicate {
    pub fn shared(parameter: impl Upcast<Parameter>) -> Predicate {
        Predicate::parameter(ParameterPredicate::Shared, parameter)
    }

    pub fn move_(parameter: impl Upcast<Parameter>) -> Predicate {
        Predicate::parameter(ParameterPredicate::Unique, parameter)
    }

    pub fn owned(parameter: impl Upcast<Parameter>) -> Predicate {
        Predicate::parameter(ParameterPredicate::Owned, parameter)
    }

    pub fn lent(parameter: impl Upcast<Parameter>) -> Predicate {
        Predicate::parameter(ParameterPredicate::Lent, parameter)
    }
}

#[term]
#[derive(Copy)]
pub enum ParameterPredicate {
    /// A parameter `a` is **share** when a value of this type, or of a type
    /// with this permission, is non-affine and hence is copied upon being
    /// given rather than moved.
    ///
    /// Note that "share" does not respect Liskov Substitution Principle:
    /// `my` is not `share` but is a subtype of `our` which *is* copy.
    Shared,

    /// A parameter `a` is **unique** when a value of this type, or of a type
    /// with this permission, is affine and hence is moved rather than copied
    /// upon being given.
    Unique,

    /// A parameter `a` is **owned** when a value of this type, or of a type
    /// with this permission, contains no **lent** values.
    Owned,

    /// A parameter `a` is **lent** when a value of this type, or of a type
    /// with this permission, contains a **leased** or **shared** value.
    Lent,
}

#[term]
#[derive(Copy)]
pub enum VarianceKind {
    /// `relative(p)` is used to express variance.
    /// Whenever a type `P T` appears in a struct
    /// and `P != my`, `Relative(T)` must hold.
    /// This indicates that `T`
    /// appears in a position that is relative to some
    /// permission.
    Relative,

    /// `atomic(p)` is used to express variance.
    /// It has to hold for all types appearing
    /// an atomic field.
    Atomic,
}

impl VarianceKind {
    pub fn apply(self, parameter: impl Upcast<Parameter>) -> Predicate {
        Predicate::variance(self, parameter)
    }
}
