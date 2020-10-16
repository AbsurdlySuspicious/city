use crate::vec2d::Vec2D;
use std::collections::VecDeque;
use fastrand::Rng;
use arrayvec::ArrayVec;

pub type WHSize = (usize, usize);
pub type PaletteColor = usize;

type Tick = u64;
const TICK_WRAP: Tick = Tick::MAX / 2;

#[derive(Debug)]
pub struct City {
    rng: Rng,
    size: WHSize,
    tick: Tick,
    background: PaletteColor,
    layers_desc: Vec<LayerDesc>,
    layers: Vec<Layer>,
    canvas: Vec2D<PaletteColor>,
}

#[derive(Debug, Clone)]
pub struct LayerDesc {
    pub density: u32,
    // 0 (max) .. 31 (min) .. >= 32 (no)
    pub speed: u64,
    // move each N ticks: 1 (faster) .. inf (slower)
    pub wall_color: PaletteColor,
    pub draw_windows: bool,
    pub window_colors: ArrayVec<[PaletteColor; 32]>,
}

#[derive(Debug, Clone, Default)]
struct Layer {
    ring: VecDeque<Building>,
}

#[derive(Debug, Clone)]
struct Building {
    size_x: usize,
    size_y: usize,
    spawn_tick: Tick,
}

impl City {
    pub fn new(
        width: usize,
        height: usize,
        rng: Rng,
        bg_color: PaletteColor,
        layers: Vec<LayerDesc>,
    ) -> City {
        City {
            rng,
            size: (width, height),
            tick: 0,
            background: bg_color,
            canvas: Vec2D::new(width as usize, height as usize, || bg_color),
            layers: vec![Layer::default(); layers.len()],
            layers_desc: layers,
        }
    }

    pub fn get_canvas(&self) -> &Vec2D<PaletteColor> {
        &self.canvas
    }

    pub fn next_tick(&mut self) {
        let City { rng, size, tick: tick_ref, background, layers_desc, layers, canvas } = self;
        let (sx, sy) = *size;
        let mut tick = *tick_ref;

        let bsz_minmax_w = (6, 25);
        let bsz_minmax_h = (10, sy + 2);

        // wipe canvas
        canvas.fill_with(*background);

        for (d, l) in layers_desc.iter().zip(layers.iter_mut()) {
            // spawn a new building on this layer
            // don't spawn if not moving on this tick && spawn decision
            if d.speed % tick == 0 && rng.u32(..).leading_zeros() >= d.density {
                let b = Building {
                    size_x: rng.usize(bsz_minmax_w.0..=bsz_minmax_w.1),
                    size_y: rng.usize(bsz_minmax_h.0..=bsz_minmax_h.1),
                    spawn_tick: tick,
                };
                l.ring.push_back(b)
            }

            let mut trunc_front = 0;

            // draw buildings on canvas
            for (i, b) in l.ring.iter().enumerate() {
                let mut wrap_tick = tick;
                if b.spawn_tick > wrap_tick {
                    wrap_tick += TICK_WRAP;
                }

                if (b.spawn_tick + (b.size_x + sx) as Tick) * d.speed < wrap_tick {
                    trunc_front = i; // mark buildings that can't be seen anymore
                }

                let y = b.size_y as usize;
                let x = sx as i32 - ((wrap_tick - b.spawn_tick) / d.speed) as i32;
                let (offset_x, x) = if x < 0 { (x.abs() as usize, 0) } else { (0, x as usize) };
                let (offset_y, y) = if y > sy { (y - sy, 0) } else { (0, sy - y) };
                let (w, h) = (b.size_x - offset_x, b.size_y - offset_y);

                for cy in y..y+h {
                    let row = canvas.get_row_mut(cy as usize);
                    for char in &mut row[x..][..w] {
                        *char = d.wall_color;
                    }
                }
            }

            // remove "expired" buildings
            if trunc_front > 0 {
                l.ring.drain(..trunc_front);
            }
        }

        // next tick
        tick += 1;
        if tick > TICK_WRAP {
            tick = 0;
        }
        *tick_ref = tick;
    }
}
