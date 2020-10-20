use std::{env, fmt, slice};
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrayvec::ArrayVec;
use bounded_vec_deque::BoundedVecDeque;
use fastrand::Rng;

use city::{City, LayerDesc, Tick};

use crate::console::{SIZE_DEFAULT_H, SIZE_DEFAULT_W, SIZE_MIN_H, SIZE_MIN_W};

mod city;
mod console;
mod vec2d;

#[derive(Debug, Default)]
struct Opts {
    fps: Option<u64>,
    step: Option<Tick>,
    seed: Option<u64>,
    auto_size: bool,
    width: Option<usize>,
    height: Option<usize>,
}

fn parse_args() -> Opts {
    fn parse<A: Borrow<str>, T: FromStr>(arg: Option<A>, name: &str) -> Option<T>
    where <T as FromStr>::Err: fmt::Debug {
        match arg.map(|a| a.borrow().parse()) {
            Some(Ok(value)) => Some(value),
            Some(Err(e)) => panic!("Can't parse {} value: {:?}", name, e),
            None => panic!("Expected value for {}", name),
        }
    }

    const HELP: &str = r#"
Usage: city [options] [width] [height]
-f      Set target fps (default: 60)
-t      Set step per frame (default: 1)
-s      Custom seed
-a      Use terminal size (auto-size)
"#;

    let pos_names = ["", "[width]", "[height]"];
    let mut pos_i = 0;
    let mut opts = Opts::default();
    let mut args = env::args();

    while let Some(a) = args.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                exit(0);
            }
            "-f" => opts.fps = parse(args.next(), &a),
            "-t" => opts.step = parse(args.next(), &a),
            "-s" => opts.seed = parse(args.next(), &a),
            "-a" => opts.auto_size = true,
            u if u.starts_with("-") => panic!("Unknown arg {}", u),
            pos => {
                match pos_i {
                    0 => {},
                    1 => opts.width = parse(Some(pos), pos_names[pos_i]),
                    2 => opts.height = parse(Some(pos), pos_names[pos_i]),
                    _ => panic!("Unknown arg at position {}", pos_i)
                }
                pos_i += 1;
            }
        }
    }

    opts
}

fn unix_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn info_center(msg: impl fmt::Display, width: usize) {
    println!("{:^w$}", msg, w = width);
}

unsafe fn deque_raw_slice<T>(d: &mut VecDeque<T>) -> &mut [T] {
    let len = d.len();
    let (right, left) = d.as_mut_slices();
    let leftmost = if left.is_empty() { right } else { left };
    slice::from_raw_parts_mut(leftmost.as_mut_ptr(), len)
}

macro_rules! av {($($x:expr),*$(,)*) => {{
    let mut vec = ArrayVec::new();
    vec.try_extend_from_slice(&[$($x,)*]).unwrap();
    vec
}}}

fn main() {
    let opts = parse_args();

    let fps = opts.fps.unwrap_or(60);
    let step = opts.step.unwrap_or(1);
    let seed = opts.seed.unwrap_or_else(unix_time);

    let auto_size = opts.auto_size;
    let (mut width, mut height) = if auto_size {
        console::get_term_size()
    } else {
        (opts.width.unwrap_or(SIZE_DEFAULT_W),
         opts.height.unwrap_or(SIZE_DEFAULT_H))
    };

    if !auto_size && (width < SIZE_MIN_W || height < SIZE_MIN_H) {
        panic!("Size is too small")
    }

    if step < 1 || step > (width / 2) as u32 {
        panic!("Invalid step")
    }

    if !(1..1000).contains(&fps) {
        panic!("Invalid fps")
    }

    let bg_color = 107;
    let layers = vec![
        LayerDesc {
            density: 0.75,
            collision: 0.4,
            speed: 4,
            wall_color: av![47],
            draw_windows: false,
            window_colors: Default::default(),
        },
        LayerDesc {
            density: 0.6,
            collision: 0.1,
            speed: 3,
            wall_color: av![100, 101],
            draw_windows: false,
            window_colors: Default::default(),
        },
        LayerDesc {
            density: 0.4,
            collision: 0.05,
            speed: 1,
            wall_color: av![40],
            draw_windows: true,
            window_colors: av![40, 107, 101],
        }
    ];

    let running = {
        let r1 = Arc::new(AtomicBool::new(true));
        let r2 = Arc::clone(&r1);
        ctrlc::set_handler(move || {
            r2.store(false, Ordering::SeqCst);
        }).unwrap();
        r1
    };

    let target_frame_time_ms = 1000 / fps;
    let frame_time = Duration::from_millis(target_frame_time_ms);
    let error_refresh_time = Duration::from_millis(500);
    let zero_d = Duration::new(0, 0);
    let mut r_times = BoundedVecDeque::new(1000);

    let rng = Rng::with_seed(seed);
    let mut console_buf = String::new();
    let mut city_state = City::new(width, height, step, &rng, bg_color, &layers);
    let mut skip_ticks = layers.iter().map(|d| d.speed).max().unwrap_or(0) * width as u32;

    let mut reset_console = true;

    while reset_console {
        reset_console = false;
        console_buf.clear();
        console_buf.shrink_to_fit();

        console::setup_console();
        info_center("oO0OoO0OoO0Oo CiTY oO0OoO0OoO0Oo", width);
        println!();
        println!("seed: {}", seed);

        console::prepare_canvas(height);
        let out = std::io::stdout();
        let mut out_lock = out.lock();

        while running.load(Ordering::Relaxed) {
            let start = SystemTime::now();

            if skip_ticks == 0 {
                if auto_size {
                    let (w, h) = console::get_term_size();
                    if w != width || h != height {
                        if w >= SIZE_MIN_W && h >= SIZE_MIN_H {
                            city_state.set_wh(w, h);
                        }
                        width = w;
                        height = h;
                        reset_console = true;
                        break;
                    }
                }

                if width < SIZE_MIN_W || height < SIZE_MIN_H {
                    console::clear_line_msg(&mut out_lock,
                                            format_args!("Too small ({}x{}) < ({}x{})",
                                                         width, height, SIZE_MIN_W, SIZE_MIN_H));
                    sleep(error_refresh_time);
                    continue;
                }
            }

            city_state.next_tick();

            if skip_ticks > 0 {
                skip_ticks -= 1;
                continue;
            }

            let before_draw = SystemTime::now();
            console::draw_to_console(&city_state, &mut console_buf, &mut out_lock);

            let diff = SystemTime::now().duration_since(start).unwrap_or(zero_d);
            let diff_tick = before_draw.duration_since(start).unwrap_or(zero_d);
            let sleep_d = frame_time.checked_sub(diff).unwrap_or(zero_d);

            let real_fps = match sleep_d.as_millis() as u64 {
                0 => 1000_u64 / diff.as_millis() as u64,
                _ => fps,
            };

            print!("\x1b[0m\x1b[2Krender time: {: >4}ms / tick: {: >4}us / fps: {: >4}",
                   diff.as_millis(), diff_tick.as_micros(), real_fps);

            r_times.push_back(diff.as_millis() as u32);
            sleep(sleep_d);
        }

        console::destroy_console();
    }

    let mut r_times = r_times.into_unbounded();
    let r_times = unsafe { deque_raw_slice(&mut r_times) };

    r_times.sort_unstable();
    let rtl = r_times.len();
    println!("render time: avg {}, 1th {}, 50th {}, 97th {}",
             r_times.iter().fold(0.0, |b, t| b + *t as f32) / rtl as f32,
             r_times[rtl / 100 * 1], r_times[rtl / 100 * 50], r_times[rtl / 100 * 97]);
}
