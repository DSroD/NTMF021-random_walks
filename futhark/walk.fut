-- https://futhark-lang.org/

import "lib/github.com/diku-dk/cpprandom/random"
module dist = uniform_int_distribution u8 pcg32

type grid_type = #square | #triangular
type walk_type = #simple | #no_returns

def get_walk_type (n: i32): walk_type = match n
        case 1 -> #no_returns
        case _ -> #simple

def get_grid_type (n: i32): grid_type = match n
        case 1 -> #triangular
        case _ -> #square

def get_dir_range (grid_type: grid_type): (i8, i8) =  match grid_type
        case #square -> (1, 4)
        case #triangular -> (1, 6)

type point = {x: i32, y: i32}
def zero: point = {x = 0, y = 0}

def norm (pt: point) = f64.hypot (f64.i32 pt.x) (f64.i32 pt.y)

def tirangular_norm (pt: point) = 
    let x = f64.i32 pt.x
    let y = f64.i32 pt.y
    in x*x + y*y - x*y |> f64.sqrt

-- associative and commutative operation with (0, 0) as a neutral element - can be parallelized using SOACs
def add_pt (pt1: point) (pt2: point) =
    {x = pt1.x + pt2.x, y = pt1.y + pt2.y}

-- adds to the second parameter in a u8 tuple
def add2 x (t: (i8, i8)) = (t.0, t.1 + x)
def unsign8 (t: (i8, i8)) = (u8.i8 t.0, u8.i8 t.1)

def step2 (dir: u8) =
    match dir
        case 1 -> {x = 1, y = 0}
        case 2 -> {x = -1, y = 0}
        case 3 -> {x = 0, y = 1}
        case 4 ->{x = 0, y = -1}
        -- triangular grid is dual to hexagonal tiling so axial coordinates for
        -- hexagonal tiling can be used, see https://www.redblobgames.com/grids/hexagons/
        case 5 -> {x = 1, y = -1}
        case 6 -> {x = -1, y = 1}
        case _ -> copy zero

def step (dir: u8) (pt: point) =
    match dir
        case 1 -> {x = pt.x + 1, y = pt.y}
        case 2 -> {x = pt.x - 1, y = pt.y}
        case 3 -> {x = pt.x, y = pt.y + 1}
        case 4 -> {x = pt.x, y = pt.y - 1}
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
    pcg32.split_rng (i64.i32 length) rng |> map (dist.rand dir_range) |> map (\l -> l.1) |> map step2 |> reduce_comm add_pt (zero)

-- no returns walk
def no_returns_walk dir_range (length: i32) rng =
    loop (rng_state, last_dir, pos) = (rng, 42, zero) for _i < length do
        let forbidden_dir = get_opposite_dir last_dir -- We can not move in this direction
        let (state, generated) = dist.rand dir_range rng_state
        let dir = if generated == forbidden_dir then (dir_range.1 + 1) else generated
        in (state, dir, step dir pos)

-- generate n walks of given length and return euclidean distance walked
def n_walks_distances rng (walk_type: walk_type) (grid_type: grid_type) n length: [n]f64 =
    let dir_range = get_dir_range grid_type
    let rngs = pcg32.split_rng n rng
    let final_point = match walk_type
        case #simple -> rngs |> map (simple_walk (unsign8 dir_range) length)
        case #no_returns -> rngs |> map (no_returns_walk (add2 (-1) dir_range |> unsign8) length) |> map (\l -> l.2)
    in match grid_type
        case #square -> map norm final_point
        case #triangular -> map tirangular_norm final_point

def mean [n] (vs: [n]f64) = (f64.sum vs) / (f64.i64 n)

-- https://futhark-lang.org/examples/variance.html
-- https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford%27s_online_algorithm
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

entry walk seed walk_type_n grid_type_n num_walks length =
    let walk_type = get_walk_type walk_type_n
    let grid_type = get_grid_type grid_type_n
    let rng = pcg32.rng_from_seed [seed]
    let distances = n_walks_distances rng walk_type grid_type num_walks length
    let mean = mean distances
    let std = variance distances
    in (mean, std)