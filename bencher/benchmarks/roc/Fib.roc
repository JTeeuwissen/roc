app "fib"
    packages { pf: "../../../crates/cli_testing_examples/benchmarks/platform/main.roc" }
    imports [pf.Task]
    provides [main] to pf

# based on: https://github.com/koka-lang/koka/blob/master/test/bench/haskell/deriv.hs
IO a : Task.Task a []

main : Task.Task {} []
main =
    r = fib 20
    Task.putLine "done"

fib : Nat -> Nat
fib = \len ->
    list = List.repeat len (len * 1000000)
    fibHelp list

fibHelp : List Nat -> Nat
fibHelp = \list ->
    len = (List.len list) // 1000000
    when len is
        0 -> 0
        1 -> 1
        _ -> fib (len - 1) +
             fib (len - 2)