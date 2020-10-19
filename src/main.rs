use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arrayvec::ArrayVec;
use fastrand::Rng;
use clap::Clap;

use city::{City, LayerDesc, Tick};

mod city;
mod console;
mod vec2d;

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    fps: Option<u64>,
    #[clap(short = 't', long)]
    step: Option<Tick>,
    #[clap(short, long)]
    seed: Option<u64>,
    width: Option<usize>,
    height: Option<usize>,
}

fn unix_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn info_center(msg: impl fmt::Display, width: usize) {
    println!("{:^w$}", msg, w = width);
}

macro_rules! av {($($x:expr),*$(,)*) => {{
    let mut vec = ArrayVec::new();
    vec.try_extend_from_slice(&[$($x,)*]).unwrap();
    vec
}}}

fn main() {
    let opts: Opts = Opts::parse();

    let fps = opts.fps.unwrap_or(60);
    let step = opts.step.unwrap_or(1);
    let seed = opts.seed.unwrap_or_else(unix_time);
    let (width, height) = (opts.width.unwrap_or(150), opts.height.unwrap_or(40));

    if step < 1 || step > (width / 2) as u32 {
        panic!("Invalid step")
    }

    if width < 30 || height < 30 {
        panic!("Size is too small")
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
            window_colors: av![107],
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

    console::setup_console();
    info_center("oO0OoO0OoO0Oo CiTY oO0OoO0OoO0Oo", width);
    println!();
    println!("seed: {}", seed);

    let rng = Rng::with_seed(seed);
    let mut console_buf = String::new();
    let mut city_state = City::new(width, height, step, &rng, bg_color, &layers);

    console::prepare_canvas(height);
    let out = std::io::stdout();
    let mut out_lock = out.lock();

    let frame_time = Duration::from_millis(1000 / fps);
    let zero_d = Duration::new(0, 0);
    let mut r_times = Vec::new();

    while running.load(Ordering::Relaxed) {
        let start = SystemTime::now();
        city_state.next_tick();
        console::draw_to_console(&city_state, &mut console_buf, &mut out_lock);

        let diff = SystemTime::now().duration_since(start).unwrap_or(zero_d);
        let sleep_d = frame_time.checked_sub(diff).unwrap_or(zero_d);
        print!("\x1b[0m\x1b[2Krender time: {}ms", diff.as_millis());
        r_times.push(diff.as_millis());
        sleep(sleep_d);
    }

    console::destroy_console();

    r_times.sort_unstable();
    let rtl = r_times.len();
    println!("render time: avg {}, 1th {}, 50th {}, 97th {}",
             r_times.iter().fold(0.0, |b, t| b + *t as f32) / rtl as f32,
             r_times[rtl / 100 * 1], r_times[rtl / 100 * 50], r_times[rtl / 100 * 97]);
}
