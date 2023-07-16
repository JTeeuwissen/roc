-- Adapted from https://github.com/leanprover/lean4/blob/IFL19/tests/bench/rbmap.hs
-- Modified to be strict in the Tree fields
import System.Environment

data Color =
  Red | Black

data Tree α β =
  Leaf
  | Node !Color !(Tree α β) !α !β !(Tree α β)

fold :: (α -> β  -> σ  -> σ) -> Tree α β -> σ  -> σ
fold _ Leaf b               = b
fold f (Node _ l k v r)   b = fold f r (f k v (fold f l b))

lt x y = x < y

ins :: Ord α => Tree α β -> α -> β -> Tree α β
ins Leaf                 kx vx = Node Red Leaf kx vx Leaf
ins (Node Red a ky vy b) kx vx =
   (if lt kx ky then Node Red (ins a kx vx) ky vy b
    else if lt ky kx then Node Red a ky vy (ins b kx vx)
    else Node Red a ky vy (ins b kx vx))
ins (Node Black a ky vy b) kx vx =
    if lt kx ky then
      case a of
        (Node Red _ _ _ _) -> case (ins a kx vx) of
          (Node _ (Node Red l2 kx2 vx2 r₁2) ky2 vy2 r₂2) -> Node Red (Node Black l2 kx2 vx2 r₁2) ky2 vy2 (Node Black r₂2 ky vy b)
          (Node _ l₁2 ky2 vy2 (Node Red l₂2 kx2 vx2 r2)) -> Node Red (Node Black l₁2 ky2 vy2 l₂2) kx2 vx2 (Node Black r2 ky vy b)
          (Node _ l2 ky2 vy2 r2)                         -> Node Black (Node Red l2 ky2 vy2 r2) ky vy b
          _                                              -> Leaf
        _ -> Node Black (ins a kx vx) ky vy b
    else if lt ky kx then
      case b of
        (Node Red _ _ _ _) -> case (ins b kx vx) of
          (Node _ (Node Red l2 kx₁2 vx₁2 r₁2) ky2 vy2 r₂2)  -> Node Red (Node Black a ky vy l2) kx₁2 vx₁2 (Node Black r₁2 ky2 vy2 r₂2)
          (Node _ l₁2 ky2 vy2 (Node Red l₂2 kx₂2 vx₂2 r₂2)) -> Node Red (Node Black a ky vy l₁2) ky2 vy2 (Node Black l₂2 kx₂2 vx₂2 r₂2)
          (Node _ l2 ky2 vy2 r2)                            -> Node Black a ky vy (Node Red l2 ky2 vy2 r2)
          _                                                 -> Leaf
        _ -> Node Black a ky vy (ins b kx vx)
    else Node Black a kx vx b

set_black :: Tree α β -> Tree α β
set_black (Node _ l k v r) = Node Black l k v r
set_black e                = e

insert t k v = case t of
  (Node Red _ _ _ _) -> set_black (ins t k v)
  _ -> ins t k v

type Map = Tree Int Bool

mk_Map_aux :: Int -> Map -> Map
mk_Map_aux 0 m = m
mk_Map_aux n m = let n' = n-1 in mk_Map_aux n' (insert m n' (n' `mod` 10 == 0))

mk_Map n = mk_Map_aux n Leaf

main = do
  -- [arg] <- getArgs
  -- let n :: Int = read arg
  let n = 4200000
  let m = mk_Map n
  let v = fold (\_ v r -> if v then r + 1 else r) m 0
  print v
