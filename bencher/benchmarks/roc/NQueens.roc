app "nqueens"
    packages { pf: "../../../crates/cli_testing_examples/benchmarks/platform/main.roc" }
    imports [pf.Task]
    provides [main] to pf

main : Task.Task {} []
main =
    queens 13
        |> Num.toStr
        |> Task.putLine

ConsList a : [Nil, Cons a (ConsList a)]

queens = \n -> length (findSolutions n n)

findSolutions = \n, k ->
    if k <= 0 then # should we use U64 as input type here instead?
        Cons Nil Nil
    else
        extend n Nil (findSolutions n (k - 1))

extend = \n, acc, solutions ->
    when solutions is
        Nil -> acc
        Cons soln rest -> extend n (appendSafe n soln acc) rest

appendSafe : I32, ConsList I32, ConsList (ConsList I32) -> ConsList (ConsList I32)
appendSafe = \k, soln, solns ->
    if k <= 0 then
        solns
    else if safe k 1 soln then
        appendSafe (k - 1) soln (Cons (Cons k soln) solns)
    else
        appendSafe (k - 1) soln solns

safe : I32, I32, ConsList I32 -> Bool
safe = \queen, diagonal, xs ->
    when xs is
        Nil -> Bool.true
        Cons q t ->
            if queen != q && queen != q + diagonal && queen != q - diagonal
                then safe queen (diagonal + 1) t
                else Bool.false

length : ConsList a -> I32
length = \xs -> lengthHelp xs 0

lengthHelp : ConsList a, I32 -> I32
lengthHelp = \foobar, acc ->
    when foobar is
        Cons _ lrest -> lengthHelp lrest (1 + acc)
        Nil -> acc
