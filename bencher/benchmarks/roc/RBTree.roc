app "rbtree"
    packages { pf: "https://github.com/roc-lang/basic-cli/releases/download/0.3.2/tE4xS_zLdmmxmHwHih9kHWQ7fsXtJr7W7h3425-eZFk.tar.br" }
    imports [pf.Stdout, pf.Task.{ Task }]
    provides [main] to pf

main : Task.Task {} []
main =
    t = makeTree 4200000
    val = fold (\_, v, r -> if v then r + 1 else r) t 0

    val
    |> Num.toStr
    |> Stdout.line

Color : [Red, Black]

Tree a b : [Leaf, Node Color (Tree a b) a b (Tree a b)]

isRed : Tree a b -> Bool
isRed = \tree ->
    when tree is
        Node Red _ _ _ _ -> Bool.true
        _ -> Bool.false

balanceLeft : Tree a b, a, b, Tree a b -> Tree a b
balanceLeft = \l, k, v, r ->
    when l is
        Leaf ->
            Leaf

        Node _ (Node Red lx kx vx rx) ky vy ry ->
            Node Red (Node Black lx kx vx rx) ky vy (Node Black ry k v r)

        Node _ ly ky vy (Node Red lx kx vx rx) ->
            Node Red (Node Black ly ky vy lx) kx vx (Node Black rx k v r)

        Node _ lx kx vx rx ->
            Node Black (Node Red lx kx vx rx) k v r

balanceRight : Tree a b, a, b, Tree a b -> Tree a b
balanceRight = \l, k, v, r ->
    when r is
        Leaf ->
            Leaf

        Node _ (Node Red lx kx vx rx) ky vy ry ->
            Node Red (Node Black l k v lx) kx vx (Node Black rx ky vy ry)

        Node _ lx kx vx (Node Red ly ky vy ry) ->
            Node Red (Node Black l k v lx) kx vx (Node Black ly ky vy ry)

        Node _ lx kx vx rx ->
            Node Black l k v (Node Red lx kx vx rx)

ins : Tree I32 Bool, I32, Bool -> Tree I32 Bool
ins = \tree, kx, vx ->
    when tree is
        Leaf ->
            Node Red Leaf kx vx Leaf

        Node Red a ky vy b ->
            when Num.compare kx ky is
                LT -> Node Red (ins a kx vx) ky vy b
                GT -> Node Red a ky vy (ins b kx vx)
                EQ -> Node Red a ky vy (ins b kx vx)

        Node Black a ky vy b ->
            when Num.compare kx ky is
                LT ->
                    if isRed a
                        then balanceLeft (ins a kx vx) ky vy b
                        else Node Black (ins a kx vx) ky vy b

                GT ->
                    if isRed b
                        then balanceRight a ky vy (ins b kx vx)
                        else Node Black a ky vy (ins b kx vx)

                EQ ->
                    Node Black a kx vx b

setBlack : Tree a b -> Tree a b
setBlack = \tree ->
    when tree is
        Node _ l k v r -> Node Black l k v r
        _ -> tree

insert : Tree I32 Bool, I32, Bool -> Tree I32 Bool
insert = \t, k, v -> if isRed t then setBlack (ins t k v) else ins t k v

fold : (a, b, omega -> omega), Tree a b, omega -> omega
fold = \f, tree, b ->
    when tree is
        Leaf -> b
        Node _ l k v r -> fold f r (f k v (fold f l b))

makeTreeHelp : I32, Tree I32 Bool -> Tree I32 Bool
makeTreeHelp = \n, t1 ->
    when n is
        0 -> t1
        _ ->
            n1 = n - 1
            t2 = insert t1 n (Num.isMultipleOf n 10)
            makeTreeHelp n1 t2

makeTree : I32 -> Tree I32 Bool
makeTree = \n ->
    makeTreeHelp n Leaf

