// Adapted from https://github.com/leanprover/lean4/blob/IFL19/tests/bench/rbmap.lean
import std/num/int32
import std/os/env

type color
  Red
  Black


type tree
  Node(color : color, lchild : tree, key : int32, value : bool, rchild : tree)
  Leaf()


fun ins(t : tree, k : int32, v : bool) : tree
  match t
    Node(Red, l, kx, vx, r)
      -> if k < kx then Node(Red, ins(l, k, v), kx, vx, r)
         elif k > kx then Node(Red, l, kx, vx, ins(r, k, v))
         else Node(Red, l, k, v, r)
    Node(Black, l, kx, vx, r)
      -> if k < kx then match l
            Node(Red) -> match ins(l,k,v)
                Node(_, Node(Red, lx2, kx2, vx2, rx2), ky2, vy2, ry2)
                  -> Node(Red, Node(Black, lx2, kx2, vx2, rx2), ky2, vy2, Node(Black, ry2, kx, vx, r))
                Node(_, ly2, ky2, vy2, Node(Red, lx2, kx2, vx2, rx2))
                  -> Node(Red, Node(Black, ly2, ky2, vy2, lx2), kx2, vx2, Node(Black, rx2, kx, vx, r))
                Node(_, lx2, kx2, vx2, rx2)
                  -> Node(Black, Node(Red, lx2, kx2, vx2, rx2), kx, vx, r)
                Leaf -> Leaf
            _ -> Node(Black, ins(l, k, v), kx, vx, r)
         elif k > kx then match r
            Node(Red) -> match ins(r,k,v)
                Node(_, Node(Red, lx2, kx2, vx2, rx2), ky2, vy2, ry2)
                  -> Node(Red, Node(Black, l, kx, vx, lx2), kx2, vx2, Node(Black, rx2, ky2, vy2, ry2))
                Node(_, lx2, kx2, vx2, Node(Red, ly2, ky2, vy2, ry2))
                  -> Node(Red, Node(Black, l, kx, vx, lx2), kx2, vx2, Node(Black, ly2, ky2, vy2, ry2))
                Node(_, lx2, kx2, vx2, rx2)
                  -> Node(Black, l, kx, vx, Node(Red, lx2, kx2, vx2, rx2))
                Leaf -> Leaf
            _ -> Node(Black, l, kx, vx, ins(r, k, v))
         else Node(Black, l, k, v, r)
    Leaf -> Node(Red, Leaf, k, v, Leaf)


fun set-black(t : tree) : tree
  match t
    Node(_, l, k, v, r) -> Node(Black, l, k, v, r)
    _ -> t


fun insert(t : tree, k : int32, v : bool) : tree
  ins(t, k, v).set-black


fun fold(t : tree, b : a, f: (int32, bool, a) -> a) : a
  match t
    Node(_, l, k, v, r) -> r.fold( f(k, v, l.fold(b, f)), f)
    Leaf                -> b


fun make-tree-aux(n : int32, t : tree) : div tree
  if n <= zero then t else
    val n1 = n.dec
    make-tree-aux(n1, insert(t, n1, n1 % 10.int32 == zero))

pub fun make-tree(n : int32) : div tree
  make-tree-aux(n, Leaf)
  

pub fun main()
  val n = get-args().head("").parse-int.default(4200000).int32
  val t = make-tree(n)
  val v = t.fold(zero) fn(k,v,r:int32){ if (v) then r.inc else r }
  v.show.println
