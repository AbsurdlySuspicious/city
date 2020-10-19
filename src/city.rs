use crate::vec2d::Vec2D;
use std::collections::VecDeque;
use fastrand::Rng;
use arrayvec::ArrayVec;

pub type WHSize = (usize, usize);
pub type PaletteColor = usize;

pub type Tick = u32;
const TICK_WRAP: Tick = Tick::MAX / 4;
const PROBABILITY_CURVE: f32 = 2.5;

const COLLISION_GAP: usize = 2;
const ROOF_GAP_X: usize = 2;
const ROOF_GAP_Y: usize = 1;
const WINDOW_X: usize = 2;
const WINDOW_Y: usize = 1;
const WINDOW_PAD: usize = 1;
const WINDOW_SPC_Y: usize = 1;
const WINDOW_SPC_X: usize = 1;

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
        // todo alloc second canvas for building drawing then [offset_x..][..w], etc..

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

                draw_building(rng, canvas, &b, d, (x, y), (offset_x, offset_y), (w, h));
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

fn draw_building(rng: &Rng, canvas: &mut Vec2D<PaletteColor>, b: &Building, layer: &LayerDesc,
                 pos_xy: (usize, usize), offset_xy: (usize, usize), limits_xy: (usize, usize)) {
    let ((ox, oy), (lw, lh)) = (offset_xy, limits_xy);
    let (cx, cy) = pos_xy;
    let (sw, sh) = (b.size_x, b.size_y);
    let (iw, ih) = (sw.min(lw), sh.min(lh));
    if lw == 0 || lh == 0 || sw < ROOF_GAP_X * 2 || sw < WINDOW_PAD * 2 + WINDOW_X {
        return; // skip on too small buildings and views
    }

    let right_gap_x = sw - ROOF_GAP_X;
    let wnd_unix_x = WINDOW_X + WINDOW_SPC_X;
    let wnd_unit_y = WINDOW_Y + WINDOW_SPC_Y;
    let wnd_fst_xy = (WINDOW_PAD, ROOF_GAP_Y + WINDOW_PAD);
    let wnd_lim_xy = (sw - WINDOW_PAD, sh - wnd_unit_y);

    let wnd_colors = &layer.window_colors;
    let wnd_colors_len = wnd_colors.len();
    let wnd_draw = layer.draw_windows && wnd_colors_len > 0;
    let wall_color = b.color;

    let row_x = move || ox..ox+iw;
    let row_i = move |x| cx + (x - ox);

    for y in oy..oy+ih {
        let r = canvas.get_row_mut(cy + (y - oy));

        if y < ROOF_GAP_Y {
            // draw upper corners

            for x in row_x() {
                if x >= ROOF_GAP_X && x < right_gap_x {
                    r[row_i(x)] = wall_color;
                }
            }
        } else {
            // draw normal walls and windows

            let mut wnd_drawn_y = false;

            if wnd_draw && y >= wnd_fst_xy.1 && y < wnd_lim_xy.1 {
                let cwnd_pos_y  = (y - wnd_fst_xy.1) % wnd_unit_y;

                if cwnd_pos_y < WINDOW_Y {
                    wnd_drawn_y = true;

                    for x in row_x() {
                        let mut clr = wall_color;

                        if x >= WINDOW_PAD && x < wnd_lim_xy.0 {
                            let cwnd_pos_x = (x - wnd_fst_xy.0) % wnd_unix_x;

                            if cwnd_pos_x < WINDOW_X {
                                let i = rng.usize(..wnd_colors_len);
                                clr = wnd_colors[i];
                            }
                        }

                        r[row_i(x)] = clr;
                    }
                }
            }

            if !wnd_drawn_y {
                for x in row_x() {
                    r[row_i(x)] = wall_color;
                }
            }
        }
    }
}
