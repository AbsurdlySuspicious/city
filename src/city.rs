use crate::vec2d::Vec2D;
use std::collections::VecDeque;
use fastrand::Rng;
use arrayvec::ArrayVec;

pub type WHSize = (usize, usize);
pub type PaletteColor = usize;

pub type Tick = u32;
const TICK_WRAP: Tick = Tick::MAX / 4;
const COLLISION_GAP: usize = 2;
const PROBABILITY_CURVE: f32 = 2.5;

#[derive(Debug)]
pub struct City<'a> {
    rng: &'a Rng,
    size: WHSize,
    step: Tick,
    tick: Tick,
    background: PaletteColor,
    layers_desc: &'a [LayerDesc],
    layers: Vec<Layer>,
    canvas: Vec2D<PaletteColor>,
}

#[derive(Debug, Clone)]
pub struct LayerDesc {
    pub density: f32, // 0.0 (min) .. 1.0 (max)
    pub collision: f32, // ^ same
    pub speed: Tick, // move each N ticks: 1 (faster) .. inf (slower)
    pub wall_color: ArrayVec<[PaletteColor; 32]>,
    pub draw_windows: bool,
    pub window_colors: ArrayVec<[PaletteColor; 32]>,
}

#[derive(Debug, Clone, Default)]
struct Layer {
    ring: VecDeque<Building>,
    rightmost_building_rcx: usize,
}

#[derive(Debug, Clone)]
struct Building {
    size_x: usize,
    size_y: usize,
    spawn_tick: Tick,
    color: PaletteColor,
}

impl<'a> City<'a> {
    pub fn new(
        width: usize,
        height: usize,
        step: Tick,
        rng: &'a Rng,
        bg_color: PaletteColor,
        layers: &'a [LayerDesc],
    ) -> City<'a> {
        City {
            rng, step,
            size: (width, height),
            tick: 1,
            background: bg_color,
            canvas: Vec2D::new(width as usize, height as usize, || bg_color),
            layers: vec![Layer::default(); layers.len()],
            layers_desc: layers,
        }
    }

    #[inline]
    pub fn get_size(&self) -> WHSize {
        self.size
    }

    #[inline]
    pub fn get_tick(&self) -> Tick {
        self.tick
    }

    #[inline]
    pub fn get_canvas(&self) -> &Vec2D<PaletteColor> {
        &self.canvas
    }

    pub fn next_tick(&mut self) {
        let City { rng, size, tick: tick_ref, background, layers_desc, layers, canvas, step } = self;
        let (sx, sy) = *size;
        let mut tick = *tick_ref;
        let step = *step;

        let bsz_minmax_w = (6, 25);
        let bsz_minmax_h = (10, sy + 2);

        // wipe canvas
        canvas.fill_with(*background);

        for (d, l) in layers_desc.iter().zip(layers.iter_mut()) {
            // spawn a new building on this layer
            // don't spawn if not moving on this tick && spawn decision
            let threshold =
                if l.rightmost_building_rcx > sx { d.collision } else { d.density };

            if tick % d.speed == 0 && rng.f32() < threshold.powf(PROBABILITY_CURVE) {
                let colors_len = d.wall_color.len();
                let color_i = if colors_len > 1 { rng.usize(..colors_len) } else { 0 };

                let b = Building {
                    size_x: rng.usize(bsz_minmax_w.0..=bsz_minmax_w.1),
                    size_y: rng.usize(bsz_minmax_h.0..=bsz_minmax_h.1),
                    spawn_tick: tick,
                    color: d.wall_color[color_i],
                };
                l.ring.push_back(b);
            }

            // draw buildings on canvas
            let mut rightmost_rc = 0;
            let b_count = l.ring.len();
            for _ in 0..b_count {
                let b = match l.ring.pop_front() {
                    Some(b) => b,
                    None => break,
                };

                let mut wrap_tick = tick;
                if b.spawn_tick > wrap_tick {
                    wrap_tick += TICK_WRAP;
                }

                let (bsz_x, bsz_y) = (b.size_x, b.size_y);
                let x = sx as i32 - ((wrap_tick - b.spawn_tick) * step / d.speed) as i32;
                let (offset_x, x) = if x < 0 { (x.abs() as usize, 0) } else { (0, x as usize) };
                let (offset_y, y) = if bsz_y > sy { (bsz_y - sy, 0) } else { (0, sy - bsz_y) };

                if offset_x > bsz_x {
                    continue; // don't requeue buildings that can't be seen anymore
                }

                let (w, h) = (bsz_x - offset_x, bsz_y - offset_y);
                let (w, h) = (w.min(sx - x), h.min(sy));

                rightmost_rc =
                    rightmost_rc.max(x + bsz_x + COLLISION_GAP);

                for cy in y..y+h {
                    let row = canvas.get_row_mut(cy as usize);
                    for char in &mut row[x..][..w] {
                        *char = b.color;
                    }
                }

                l.ring.push_back(b);
            }

            l.rightmost_building_rcx = rightmost_rc;
        }

        // next tick
        tick += 1;
        if tick > TICK_WRAP {
            tick = 1;
        }
        *tick_ref = tick;
    }
}
