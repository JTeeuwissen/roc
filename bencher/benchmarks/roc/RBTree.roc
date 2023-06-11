app "rbtree"
    packages { pf: "https://github.com/roc-lang/basic-cli/releases/download/0.3.2/tE4xS_zLdmmxmHwHih9kHWQ7fsXtJr7W7h3425-eZFk.tar.br" }
    imports [pf.Stdout, pf.Task.{ Task }]
    provides [main] to pf

# pub fun main()
#   val n = get-args().head("").parse-int.default(4200000).int32
#   val t = make-tree(n)
#   val v = t.fold(zero) fn(k,v,r:int32){ if (v) then r.inc else r }
#   v.show.println
main : Task.Task {} []
main =
    t = makeTree 4200000
    val = fold (\_, v, r -> if v then r + 1 else r) t 0

    val
    |> Num.toStr
    |> Stdout.line

# type color
#   Red
#   Black
Color : [Red, Black]

# type tree
#   Node(color : color, lchild : tree, key : int32, value : bool, rchild : tree)
#   Leaf()
Tree a b : [Leaf, Node Color (Tree a b) a b (Tree a b)]
  

# fun is-red(t : tree) : bool
#   match t
#     Node(Red) -> True
#     _         -> False
isRed : Tree a b -> Bool
isRed = \tree ->
    when tree is
        Node Red _ _ _ _ -> Bool.true
        _ -> Bool.false


# fun balance-left(l :tree, k : int32, v : bool, r : tree) : tree
#   match l
#     Node(_, Node(Red, lx, kx, vx, rx), ky, vy, ry)
#       -> Node(Red, Node(Black, lx, kx, vx, rx), ky, vy, Node(Black, ry, k, v, r))
#     Node(_, ly, ky, vy, Node(Red, lx, kx, vx, rx))
#       -> Node(Red, Node(Black, ly, ky, vy, lx), kx, vx, Node(Black, rx, k, v, r))
#     Node(_, lx, kx, vx, rx)
#       -> Node(Black, Node(Red, lx, kx, vx, rx), k, v, r)
#     Leaf -> Leaf
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

# fun balance-right(l : tree, k : int32, v : bool, r : tree) : tree
#   match r
#     Node(_, Node(Red, lx, kx, vx, rx), ky, vy, ry)
#       -> Node(Red, Node(Black, l, k, v, lx), kx, vx, Node(Black, rx, ky, vy, ry))
#     Node(_, lx, kx, vx, Node(Red, ly, ky, vy, ry))
#       -> Node(Red, Node(Black, l, k, v, lx), kx, vx, Node(Black, ly, ky, vy, ry))
#     Node(_, lx, kx, vx, rx)
#       -> Node(Black, l, k, v, Node(Red, lx, kx, vx, rx))
#     Leaf -> Leaf
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


# fun ins(t : tree, k : int32, v : bool) : tree
#   match t
#     Node(Red, l, kx, vx, r)
#       -> if k < kx then Node(Red, ins(l, k, v), kx, vx, r)
#          elif k > kx then Node(Red, l, kx, vx, ins(r, k, v))
#          else Node(Red, l, k, v, r)
#     Node(Black, l, kx, vx, r)
#       -> if k < kx then (if is-red(l) then balance-left(ins(l,k,v), kx, vx, r)
#                                       else Node(Black, ins(l, k, v), kx, vx, r))
#          elif k > kx then (if is-red(r) then balance-right(l, kx, vx, ins(r,k,v))
#                                         else Node(Black, l, kx, vx, ins(r, k, v)))
#          else Node(Black, l, k, v, r)
#     Leaf -> Node(Red, Leaf, k, v, Leaf)
ins : Tree I64 Bool, I64, Bool -> Tree I64 Bool
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


# fun set-black(t : tree) : tree
#   match t
#     Node(_, l, k, v, r) -> Node(Black, l, k, v, r)
#     _ -> t
setBlack : Tree a b -> Tree a b
setBlack = \tree ->
    when tree is
        Node _ l k v r -> Node Black l k v r
        _ -> tree


# fun insert(t : tree, k : int32, v : bool) : tree
#   ins(t, k, v).set-black
insert : Tree I64 Bool, I64, Bool -> Tree I64 Bool
insert = \t, k, v -> if isRed t then setBlack (ins t k v) else ins t k v


# fun fold(t : tree, b : a, f: (int32, bool, a) -> a) : a
#   match t
#     Node(_, l, k, v, r) -> r.fold( f(k, v, l.fold(b, f)), f)
#     Leaf                -> b
fold : (a, b, omega -> omega), Tree a b, omega -> omega
fold = \f, tree, b ->
    when tree is
        Leaf -> b
        Node _ l k v r -> fold f r (f k v (fold f l b))

# fun make-tree-aux(n : int32, t : tree) : div tree
#   if n <= zero then t else
#     val n1 = n.dec
#     make-tree-aux(n1, insert(t, n1, n1 % 10.int32 == zero))
makeTreeHelp : I64, Tree I64 Bool -> Tree I64 Bool
makeTreeHelp = \n, t1 ->
    when n is
        0 -> t1
        _ ->
            n1 = n - 1
            t2 = insert t1 n (Num.isMultipleOf n 10)
            makeTreeHelp n1 t2

# pub fun make-tree(n : int32) : div tree
#   make-tree-aux(n, Leaf)
makeTree : I64 -> Tree I64 Bool
makeTree = \n ->
    makeTreeHelp n Leaf

