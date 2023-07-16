app "rbtree"
    packages { pf: "../../../crates/cli_testing_examples/benchmarks/platform/main.roc" }
    imports [pf.Task]
    provides [main] to pf

main : Task.Task {} []
main =
    t = makeTree 4200000
    
    val = fold (\_, v, r -> if v then r + 1 else r) t 0

    val
    |> Num.toStr
    |> Task.putLine

Color : [Red, Black]

Tree a b : [Leaf, Node Color (Tree a b) a b (Tree a b)]

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
                    when a is
                        Node Red _ _ _ _  -> when (ins a kx vx) is
                            Leaf ->
                                Leaf

                            Node _ (Node Red lx2 kx2 vx2 rx2) ky2 vy2 ry2 ->
                                Node Red (Node Black lx2 kx2 vx2 rx2) ky2 vy2 (Node Black ry2 ky vy b)

                            Node _ ly2 ky2 vy2 (Node Red lx2 kx2 vx2 rx2) ->
                                Node Red (Node Black ly2 ky2 vy2 lx2) kx2 vx2 (Node Black rx2 ky vy b)

                            Node _ lx2 kx2 vx2 rx2 ->
                                Node Black (Node Red lx2 kx2 vx2 rx2) ky vy b
                        
                        _ -> Node Black (ins a kx vx) ky vy b

                GT ->
                    when b is
                        Node Red _ _ _ _ -> when (ins b kx vx) is
                            Leaf ->
                                Leaf

                            Node _ (Node Red lx2 kx2 vx2 rx2) ky2 vy2 ry2 ->
                                Node Red (Node Black a ky vy  lx2) kx2 vx2 (Node Black rx2 ky2 vy2 ry2)

                            Node _ lx2 kx2 vx2 (Node Red ly2 ky2 vy2 ry2) ->
                                Node Red (Node Black a ky vy  lx2) kx2 vx2 (Node Black ly2 ky2 vy2 ry2)

                            Node _ lx2 kx2 vx2 rx2 ->
                                Node Black a ky vy  (Node Red lx2 kx2 vx2 rx2)

                        _ -> Node Black a ky vy (ins b kx vx)

                EQ ->
                    Node Black a kx vx b

setBlack : Tree a b -> Tree a b
setBlack = \tree ->
    when tree is
        Node _ l k v r -> Node Black l k v r
        _ -> tree

insert : Tree I32 Bool, I32, Bool -> Tree I32 Bool
insert = \t, k, v -> when t is
    Node Red _ _ _ _  -> setBlack (ins t k v)
    _ -> ins t k v

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

