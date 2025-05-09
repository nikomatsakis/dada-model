use crate::{dada_lang::term, type_system::check_program};
use formality_core::{test, test_util::ResultTestExt};

/// Check we are able to type check an empty method.
#[test]
fn empty_method() {
    check_program(&term(
        "
        class TheClass {
            fn empty_method(my self) {}
        }
        ",
    ))
    .assert_ok(expect_test::expect!["()"]);
}

/// Check that empty blocks return unit (and that is not assignable to Int)
#[test]
fn bad_int_return_value() {
    check_program(
        &term(
            "
            class TheClass {
                fn empty_method(my self) -> Int {}
            }
        ",
        )
    ).assert_err(
        expect_test::expect![[r#"
            check program `class TheClass { fn empty_method (my self) -> Int { } }`

            Caused by:
                0: check class named `TheClass`
                1: check method named `empty_method`
                2: check function body
                3: judgment `can_type_expr_as { expr: { }, as_ty: Int, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                     the rule "can_type_expr_as" failed at step #0 (src/file.rs:LL:CC) because
                       judgment `type_expr_as { expr: { }, as_ty: Int, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                         the rule "type_expr_as" failed at step #1 (src/file.rs:LL:CC) because
                           judgment had no applicable rules: `sub { a: (), b: Int, live_after: LivePlaces { accessed: {}, traversed: {} }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 } }`"#]],
    )
}

/// Check that empty blocks return unit (and that is not assignable to Int)
#[test]
fn bad_int_ascription() {
    check_program(
        &term(
            "
            class TheClass {
                fn empty_method(my self) {
                    let x: Int = ();
                }
            }
        ",
        )
    ).assert_err(
        expect_test::expect![[r#"
            check program `class TheClass { fn empty_method (my self) -> () { let x : Int = () ; } }`

            Caused by:
                0: check class named `TheClass`
                1: check method named `empty_method`
                2: check function body
                3: judgment `can_type_expr_as { expr: { let x : Int = () ; }, as_ty: (), env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                     the rule "can_type_expr_as" failed at step #0 (src/file.rs:LL:CC) because
                       judgment `type_expr_as { expr: { let x : Int = () ; }, as_ty: (), env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                         the rule "type_expr_as" failed at step #0 (src/file.rs:LL:CC) because
                           judgment `type_expr { expr: { let x : Int = () ; }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                             the rule "block" failed at step #0 (src/file.rs:LL:CC) because
                               judgment `type_block { block: { let x : Int = () ; }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                                 the rule "place" failed at step #0 (src/file.rs:LL:CC) because
                                   judgment `type_statements_with_final_ty { statements: [let x : Int = () ;], ty: (), env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                                     the rule "cons" failed at step #1 (src/file.rs:LL:CC) because
                                       judgment `type_statement { statement: let x : Int = () ;, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                                         the rule "let" failed at step #0 (src/file.rs:LL:CC) because
                                           judgment `type_expr_as { expr: (), as_ty: Int, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                                             the rule "type_expr_as" failed at step #1 (src/file.rs:LL:CC) because
                                               judgment had no applicable rules: `sub { a: (), b: Int, live_after: LivePlaces { accessed: {}, traversed: {} }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 } }`"#]],
    )
}

/// Check returning an integer with return type of Int.
#[test]
fn good_int_return_value() {
    check_program(&term(
        "
        class TheClass {
            fn empty_method(my self) -> Int {
                22;
            }
        }
    ",
    ))
    .assert_ok(expect_test::expect!["()"]);
}

/// Check returning an instance of a class.
#[test]
#[allow(non_snake_case)]
fn return_instance_of_Foo() {
    check_program(&term(
        "
        class Foo { }

        class TheClass {
            fn empty_method(my self) -> Foo {
                new Foo();
            }
        }
    ",
    ))
    .assert_ok(expect_test::expect!["()"]);
}

/// Check returning an instance of a class.
#[test]
#[allow(non_snake_case)]
fn return_from_variable() {
    check_program(&term(
        "
        class Foo { }

        class TheClass {
            fn empty_method(my self) -> Foo {
                let foo = new Foo();
                foo.move;
            }
        }
    ",
    ))
    .assert_ok(expect_test::expect!["()"]);
}

/// Check returning a shared instance of a class when an owned instance is expected.
#[test]
#[allow(non_snake_case)]
fn return_shared_not_give() {
    check_program(
        &term(
            "
            class Foo { }
    
            class TheClass {
                fn empty_method(my self) -> Foo {
                    let foo = new Foo();
                    foo.ref;
                }
            }
        ",
        ),
    ).assert_err(
        expect_test::expect![[r#"
            check program `class Foo { } class TheClass { fn empty_method (my self) -> Foo { let foo = new Foo () ; foo . ref ; } }`

            Caused by:
                0: check class named `TheClass`
                1: check method named `empty_method`
                2: check function body
                3: judgment `can_type_expr_as { expr: { let foo = new Foo () ; foo . ref ; }, as_ty: Foo, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                     the rule "can_type_expr_as" failed at step #0 (src/file.rs:LL:CC) because
                       judgment `type_expr_as { expr: { let foo = new Foo () ; foo . ref ; }, as_ty: Foo, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass}, assumptions: {}, fresh: 0 }, live_after: LivePlaces { accessed: {}, traversed: {} } }` failed at the following rule(s):
                         the rule "type_expr_as" failed at step #1 (src/file.rs:LL:CC) because
                           judgment `sub { a: ref [foo] Foo, b: Foo, live_after: LivePlaces { accessed: {}, traversed: {} }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass, foo: Foo}, assumptions: {}, fresh: 0 } }` failed at the following rule(s):
                             the rule "sub-classes" failed at step #3 (src/file.rs:LL:CC) because
                               judgment `sub_perms { perm_a: ref [foo], perm_b: my, live_after: LivePlaces { accessed: {}, traversed: {} }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass, foo: Foo}, assumptions: {}, fresh: 0 } }` failed at the following rule(s):
                                 the rule "sub_red_perms" failed at step #2 (src/file.rs:LL:CC) because
                                   judgment `red_chain_sub_perm { red_chain_a: RedChain { links: [Rfd(foo)] }, red_perm_b: RedPerm { chains: {RedChain { links: [] }} }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass, foo: Foo}, assumptions: {}, fresh: 0 } }` failed at the following rule(s):
                                     the rule "sub_red_perms" failed at step #1 (src/file.rs:LL:CC) because
                                       judgment `red_chain_sub_chain { red_chain_a: RedChain { links: [Rfd(foo)] }, red_chain_b: RedChain { links: [] }, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass, foo: Foo}, assumptions: {}, fresh: 0 } }` failed at the following rule(s):
                                         the rule "(ref-dead::P) vs Q ~~> (our::P) vs Q" failed at step #2 (src/file.rs:LL:CC) because
                                           judgment `prove_is_lent { a: my, env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass, foo: Foo}, assumptions: {}, fresh: 0 } }` failed at the following rule(s):
                                             the rule "is-lent" failed at step #0 (src/file.rs:LL:CC) because
                                               judgment `prove_predicate { predicate: lent(my), env: Env { program: "...", universe: universe(0), in_scope_vars: [], local_variables: {self: my TheClass, foo: Foo}, assumptions: {}, fresh: 0 } }` failed at the following rule(s):
                                                 the rule "parameter" failed at step #0 (src/file.rs:LL:CC) because
                                                   pattern `true` did not match value `false`"#]],
    )
}

/// Check returning a shared instance of a class when an owned instance is expected.
#[test]
#[allow(non_snake_case)]
fn return_int_field_from_class_with_int_field() {
    check_program(&term(
        "
        class Foo {
            i: Int;
        }

        class TheClass {
            fn empty_method(my self) -> Int {
                let foo = new Foo(22);
                foo.i.move;
            }
        }
    ",
    ))
    .assert_ok(expect_test::expect!["()"]);
}

/// Check returning a shared instance of a class when an owned instance is expected.
#[test]
#[allow(non_snake_case)]
fn return_modified_int_field_from_class_with_int_field() {
    check_program(&term(
        "
        class Foo {
            i: Int;
        }

        class TheClass {
            fn empty_method(my self) -> Int {
                let foo = new Foo(22);
                foo.i = foo.i.move + 1;
                foo.i.move;
            }
        }
    ",
    ))
    .assert_ok(expect_test::expect!["()"]);
}
