use std::iter;
use std::mem;

use once_cell::sync::Lazy;
use test::Bencher;

use crate::mv;
use slipstream::prelude::*;

type Bools = bx32;
type Counts = u8x32;

#[derive(Clone, Debug, PartialEq)]
struct Life {
    edge: usize,
    cells: Vec<bool>,
    next: Vec<bool>,
}

const NEIGHS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
];

const SIZE: usize = 1026;

impl Life {
    fn at(&self, x: usize, y: usize) -> usize {
        y * self.edge + x
    }
    fn set(&mut self, x: usize, y: usize, val: bool) {
        let idx = self.at(x, y);
        self.cells[idx] = val;
    }
    fn set_next(&mut self, x: usize, y: usize, val: bool) {
        let idx = self.at(x, y);
        self.next[idx] = val;
    }
    fn get(&self, x: usize, y: usize) -> bool {
        self.cells[self.at(x, y)]
    }

    /// Place a frame of always dead cells which won't participate in the game.
    ///
    /// These just solve the issue what to do with edges of the game plan.
    fn frame(&mut self) {
        for i in 0..self.edge {
            self.set(0, i, false);
            self.set(self.edge - 1, i, false);
            self.set(i, 0, false);
            self.set(i, self.edge - 1, false);
        }
    }
    fn gen() -> Self {
        fn inner() -> Life {
            let cells = iter::repeat_with(rand::random).take(SIZE * SIZE).collect();
            let mut me = Life {
                edge: SIZE,
                cells,
                next: Vec::new(),
            };
            me.frame();
            me.next = me.cells.clone();
            me
        }

        static CACHED: Lazy<Life> = Lazy::new(inner);
        CACHED.clone()
    }

    fn step(&mut self) {
        for y in 1..self.edge - 1 {
            for x in 1..self.edge - 1 {
                let cnt = NEIGHS
                    .iter()
                    .filter(|&&(xd, yd)| {
                        self.get(((x as isize) + xd) as usize, ((y as isize) + yd) as usize)
                    })
                    .count();
                let alive = match cnt {
                    2 if self.get(x, y) => true,
                    3 => true,
                    _ => false,
                };
                self.set_next(x, y, alive);
            }
        }
        mem::swap(&mut self.cells, &mut self.next);
    }

    mv! {
        fn step_vectorized(&mut self) {
            assert_eq!(mem::align_of::<Counts>(), mem::align_of::<Bools>());
            assert_eq!(mem::size_of::<Counts>(), mem::size_of::<Bools>());
            let twos = Counts::splat(2);
            let threes = Counts::splat(3);
            let dead = Bools::default();
            let alive = Bools::splat(true);

            let mut neighs: [_; 8] = Default::default();
            for y in 1..self.edge - 1 {
                let cells = &self.cells;
                for (ndest, &(xd, yd)) in neighs.iter_mut().zip(&NEIGHS) {
                    let idx = self.at((1 + xd) as usize, ((y as isize) + yd) as usize);
                    *ndest = &cells[idx..idx + self.edge - 2];
                }

                let center_idx = self.at(1, y);
                let center = &cells[center_idx..center_idx + self.edge - 2];
                let dst = &mut self.next[center_idx..center_idx + self.edge - 2];

                let iter = slipstream::vectorize::<([Bools; 8], Bools, _), _>((neighs, center, dst));

                for (neighs, center, mut dst) in iter {
                    let mut live_neigh_cnt = Counts::default();
                    // FIXME: Using sum here unfortunately prevents inlining, which leads to
                    // performance drop *and* barrier across which we don't get the AVX
                    // instructions. So manually expanding the loop.
                    for n in &neighs {
                        // TODO: We want some safe transforms in here.
                        live_neigh_cnt += unsafe { mem::transmute::<_, Counts>(*n) };
                    }
                    let survive = live_neigh_cnt.eq(twos);
                    *dst = dead.blend(alive, survive) & center;
                    let born = live_neigh_cnt.eq(threes);
                    *dst |= dead.blend(alive, born);
                }
            }
            mem::swap(&mut self.cells, &mut self.next);
        }
    }
}

#[bench]
fn basic(b: &mut Bencher) {
    let mut life = Life::gen();

    b.iter(|| {
        life.step();
    });
}

#[bench]
fn vectorize_detect(b: &mut Bencher) {
    let mut life = Life::gen();

    b.iter(|| {
        life.step_vectorized();
    });
}

#[bench]
fn vectorize_default(b: &mut Bencher) {
    let mut life = Life::gen();

    b.iter(|| {
        life.step_vectorized_default_version();
    });
}

#[test]
fn same_results() {
    let mut l1 = Life::gen();
    let mut l2 = l1.clone();

    for i in 0..100 {
        assert_eq!(l1, l2, "Lifes differ in step {}", i);
        l1.step();
        l2.step_vectorized();
    }
}

// TODO: Anyone wants to volunteer and write a manually-vectorized version?
