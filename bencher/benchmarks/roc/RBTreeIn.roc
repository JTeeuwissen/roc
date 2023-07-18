app "rbtree-in"
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
                        Leaf -> Node Black (ins a kx vx) ky vy b
                        Node c _ _ _ _ -> when c is
                            Black -> Node Black (ins a kx vx) ky vy b
                            _ -> when (ins a kx vx) is
                                Leaf ->
                                    Leaf

                                Node _ lz kz vz rz -> when lz is
                                    (Node c2 lx2 kx2 vx2 rx2) -> when c2 is
                                        Red -> Node Red (Node Black lx2 kx2 vx2 rx2) kz vz (Node Black rz ky vy b)
                                        _ -> when rz is 
                                            (Node c3 ly2 ky2 vy2 ry2) -> when c3 is
                                                Red -> Node Red (Node Black lz kz vz ly2) ky2 vy2 (Node Black ry2 ky vy b)
                                                _ -> Node Black (Node Red lz kz vz rz) ky vy b
                                            _ -> Node Black (Node Red lz kz vz rz) ky vy b
                                    _ -> when rz is 
                                        Node c2 lx2 kx2 vx2 rx2 -> when c2 is
                                            Red -> Node Red (Node Black lz kz vz lx2) kx2 vx2 (Node Black rx2 ky vy b)
                                            _ -> Node Black (Node Red lz kz vz rz) ky vy b
                                        _ -> Node Black (Node Red lz kz vz rz) ky vy b
                GT ->
                    when b is
                        Node c _ _ _ _ -> when c is
                            Red -> when (ins b kx vx) is
                                Leaf ->
                                    Leaf

                                Node _ lz kz vz rz -> when lz is
                                    (Node c2 lx2 kx2 vx2 rx2) -> when c2 is
                                        Red -> Node Red (Node Black a ky vy lx2) kx2 vx2 (Node Black rx2 kz vz rz)
                                        _ -> when rz is
                                            (Node c3 ly2 ky2 vy2 ry2) -> when c3 is
                                                Red -> Node Red (Node Black a ky vy lz) kz vz (Node Black ly2 ky2 vy2 ry2)
                                                _ -> Node Black a ky vy (Node Red lz kz vz rz)
                                            _ -> Node Black a ky vy (Node Red lz kz vz rz)
                                    _ -> when rz is
                                        (Node c2 ly2 ky2 vy2 ry2) -> when c2 is
                                            Red -> Node Red (Node Black a ky vy lz) kz vz (Node Black ly2 ky2 vy2 ry2)
                                            _ -> Node Black a ky vy (Node Red lz kz vz rz)
                                        _ -> Node Black a ky vy (Node Red lz kz vz rz)
                            _ -> Node Black a ky vy (ins b kx vx)
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

