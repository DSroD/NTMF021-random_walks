-- https://futhark-lang.org/

-- ==
-- compiled input {1111 0 0 500i64 1000 1000}

import "lib/github.com/diku-dk/cpprandom/random"
module dist = uniform_int_distribution u8 minstd_rand

type grid_type = #square | #triangular | #hexagonal
type walk_type = #simple | #no_returns

def get_walk_type (n: i32): walk_type = match n
        case 1 -> #no_returns
        case _ -> #simple

def get_grid_type (n: i32): grid_type = match n
        case 1 -> #triangular
        case 2 -> #hexagonal
        case _ -> #square

def get_dir_range (grid_type: grid_type): (i8, i8) =  match grid_type
        case #square -> (1, 4)
        case #triangular -> (1, 6)
        case #hexagonal -> (1, 3)


type point = {x: i32, y: i32}
def zero: point = {x = 0, y = 0}
-- hypot calculates pythagorean sum without over/under flows
def norm (pt: point) = f64.hypot (f64.i32 pt.x) (f64.i32 pt.y)
-- triangular norm (under/over flows not accounted for)
def tirangular_norm (pt: point) = 
    let x = f64.i32 pt.x
    let y = f64.i32 pt.y
    in x*x + y*y - x*y |> f64.sqrt
-- hexagonal norm (compute dx and dy and then hypotenuse)
def dx_even x = (f64.sqrt x) * (f64.sqrt (x / 2 + 1))
def dx_odd x =
    let dxp1 = dx_even (x+1)
    -- cosine law
    in dxp1*dxp1 + 1 - dxp1 * 0.8660 |> f64.sqrt
def dy x y: f64 = 3 * y / 2 + y % 2 - (x % 2) * 0.8660
def hexagonal_norm (pt: point) =
    let x = f64.i32 pt.x |> f64.abs
    let y = f64.i32 pt.y
    let dx = if x % 2 == 0 then dx_even x else dx_odd x
    in f64.hypot dx (dy x y)

-- associative and commutative operation with (0, 0) as a neutral element - can be parallelized using SOACs
def add_pt (pt1: point) (pt2: point) =
    {x = pt1.x + pt2.y, y = pt1.x + pt2.y}

-- adds to the second parameter in a u8 tuple
def add2 x (t: (i8, i8)) = (t.0, t.1 + x)
def unsign8 (t: (i8, i8)) = (u8.i8 t.0, u8.i8 t.1)

def step (dir: u8) (pt: point) =
    match dir
        case 1 -> pt with x = pt.x + 1
        case 2 -> pt with x = pt.x - 1
        -- hexagonal grid is dual to triangular tiling - we use coordinates of triangular tiles
        case 3 -> pt with y = if pt.x + pt.y % 2 == 0 then pt.y + 1 else pt.y - 1
        -- complement for non-triangular grids
        case 4 -> pt with y = if pt.x + pt.y % 2 == 0 then pt.y - 1 else pt.y + 1
        -- triangular grid is dual to hexagonal tiling so axial coordinates for
        -- hexagonal tiling can be used, see https://www.redblobgames.com/grids/hexagons/
        case 5 -> {x = pt.x + 1, y = pt.y - 1}
        case 6 -> {x = pt.x - 1, y = pt.y + 1}
        case _ -> pt

-- XOR ing with 1 bitwise swaps LSB -> this swaps
-- 0 <-> 1; 2 <-> 3; 4 <-> 5; ...
-- this exactly corresponds to not being able to return to the previous coordinate 
-- (see 'step' function above and pay attention to -1 and +1 in get_opposite_dir function)
def get_opposite_dir (dir: u8): u8 =
    ((dir - 1) ^ 1) + 1

-- simple walk
def simple_walk dir_range (length: i32) rng =
    loop (rng_state, pos) = (rng, copy zero) for _i < length do
        let (state, next) = dist.rand dir_range rng_state
        in (state, step next pos)

def gen_simple_walk dir_range num_buckets bucket_size rng =
    minstd_rand.split_rng (i64.i32 num_buckets) rng 
    |> map (simple_walk dir_range bucket_size) 
    |> map (\l -> l.1) 
    |> reduce_comm add_pt zero 

-- no returns walk
def no_returns_walk dir_range (length: i32) rng =
    loop (rng_state, last_dir, pos) = (rng, 0, copy zero) for _i < length do
        let forbidden_dir = get_opposite_dir last_dir -- We can not move in this direction
        let (state, generated) = dist.rand dir_range rng_state
        let dir = if generated == forbidden_dir then (dir_range.1 + 1) else generated
        in (state, dir, step dir pos)

def gen_no_returns_walk dir_range num_buckets bucket_size rng =
    minstd_rand.split_rng (i64.i32 num_buckets) rng 
    |> map (no_returns_walk dir_range bucket_size) 
    |> map (\l -> l.2) 
    |> reduce_comm add_pt zero

-- generate n walks given and bucket size and number of buckets (walk length = num_buckets * bucket_size) and return their distances
def gen_n_walk_distances rng (walk_type: walk_type) (grid_type: grid_type) n num_buckets bucket_size: [n]f64 =
    let dir_range = get_dir_range grid_type 
    let rngs = minstd_rand.split_rng n rng
    let final_point = match walk_type
        case #simple -> rngs |> map (gen_simple_walk (unsign8 dir_range) num_buckets bucket_size)
        case #no_returns -> rngs |> map (gen_no_returns_walk (add2 (-1) dir_range |> unsign8) num_buckets bucket_size)
    in match grid_type
        case #square -> map norm final_point
        case #triangular -> map tirangular_norm final_point
        case #hexagonal -> map hexagonal_norm final_point
    
def mean [n] (vs: [n]f64) = f64.sum vs / f64.i64 n

-- Sum of squares can lead to instabilities, different algorithm is employed
-- https://futhark-lang.org/examples/variance.html
-- https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm
-- Associative operator for combinating subsequences
-- from Welford's online algorithm
def var_op (na, ma, m2a) (nb, mb, m2b) =
    let nab = na + nb
    in if nab == 0 then (0, 0, 0) else
        let fa = f64.from_fraction na nab
        let fb = f64.from_fraction nb nab
        let fab = f64.from_fraction (na * nb) nab
        let delta = mb - ma
        let mab = fa * ma + fb * mb
        let m2ab = m2a + m2b + delta * delta * fab
        in (nab, mab, m2ab)

def variance [n] (vs: [n]f64) =
    let (_, _, m2) =
        reduce_comm var_op (0, 0, 0) (map (\a -> (1, a, 0)) vs)
    in (m2 / (f64.i64 (n-1)))

entry one_walk seed walk_type_n grid_type_n num_walks num_buckets bucket_size =
    let walk_type = get_walk_type walk_type_n
    let grid_type = get_grid_type grid_type_n
    let rng = minstd_rand.rng_from_seed [seed]
    let distances = gen_n_walk_distances rng walk_type grid_type num_walks num_buckets bucket_size
    let mean = mean distances
    let s2 = variance distances
    in (mean, s2)

entry many_walks [n] seed walk_type_n grid_type_n num_walks (nums_buckets: [n]i32) (bucket_sizes: [n]i32) =
    let walk_type = get_walk_type walk_type_n
    let grid_type = get_grid_type grid_type_n
    let rng = minstd_rand.rng_from_seed [seed]
    let distances = minstd_rand.split_rng n rng
        |> zip3 nums_buckets bucket_sizes
        |> map (\(num_buckets, bucket_size, r) -> gen_n_walk_distances r walk_type grid_type num_walks num_buckets bucket_size)
    let means = map mean distances
    let s2s = map variance distances
    in [means, s2s]
