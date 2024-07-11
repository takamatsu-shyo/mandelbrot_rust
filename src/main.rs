#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

/// https://doc.rust-lang.org/rustc/lints/groups.html
/// Lints to nudge you toward idiomatic features of Rust 2018

/// https://users.rust-lang.org/t/what-is-the-elided-lifetimes-in-paths-lint-for/28005/2
/// If a function signature has an elided lifetime parameter in return position,
/// the error message will point the root of the problem.
use num::Complex;

/// Try to determine if 'c' is in the Mandelbrot set, using at most 'limit'
/// iterations to decide
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 1..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

use std::str::FromStr;

/// Parse the string 's' as a coordinate pair, like "400x600" or "1.0,0.5"
/// 's' is the "separator" argument
/// If 's' has the proper form, return 'Some<(x, y)>'
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Parse a pair of floating-point numbers
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex {
            re: -0.5,
            im: -0.75
        }
    );
}

/// Render a rectanble of the Mandelbrot set int to a buffer of pixels.
fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            };
        }
    }
}

/// Simple multithread render
fn bands(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
    threads: usize,
) {
    let row_per_band = bounds.1 / threads + 1;
    let bands: Vec<&mut [u8]> = pixels.chunks_mut(row_per_band * bounds.0).collect();
    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = row_per_band * i;
            let height = band.len() / bounds.0;
            let band_bounds = (bounds.0, height);
            let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);

            spawner.spawn(move |_| {
                render(band, band_bounds, band_upper_left, band_lower_right);
            });
        }
    })
    .unwrap();
}

use std::sync::Mutex;

/// Task queue
fn task_queue(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
    threads: usize,
) {
    let row_per_band = bounds.1 / threads + 1;
    {
        let bands = Mutex::new(pixels.chunks_mut(row_per_band * bounds.0).enumerate());
        crossbeam::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|_| loop {
                    match {
                        let mut guard = bands.lock().unwrap();
                        guard.next()
                    } {
                        None => {
                            return;
                        }
                        Some((i, band)) => {
                            let top = row_per_band * i;
                            let height = band.len() / bounds.0;
                            let band_bounds = (bounds.0, height);
                            let band_upper_left =
                                pixel_to_point(bounds, (0, top), upper_left, lower_right);
                            let band_lower_right = pixel_to_point(
                                bounds,
                                (bounds.0, top + height),
                                upper_left,
                                lower_right,
                            );

                            render(band, band_bounds, band_upper_left, band_lower_right);
                        }
                    }
                });
            }
        })
        .unwrap();
    }
}

use std::env;
use std::time::{Duration, Instant};

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        args = vec![
            args[0].clone(),
            "rust_mandel.png".into(),
            "2000x1500".into(),
            "-1.20,0.35".into(),
            "-1.0,0.20".into(),
        ];
    } else if args.len() != 5 {
        eprintln!("Usage: {} File Pixels Upper_Left Lower_Right", args[0]);
        eprintln!(
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    // Multithreading test part
    // num_cpus
    let num_logical_cores = num_cpus::get();
    let num_physical_cores = num_cpus::get_physical();
    println!("Logical core : {}", num_logical_cores);
    println!("Physical core: {}", num_physical_cores);

    let iteration = 5;

    // Single thread
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..iteration {
        let start = Instant::now();

        render(&mut pixels, bounds, upper_left, lower_right);

        let duration = start.elapsed();
        total_duration += duration;
    }
    let average_duration = total_duration / iteration;
    println!("render {:?}", average_duration);

    // -------------
    // Bnads logical
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..iteration {
        let start = Instant::now();

        bands(
            &mut pixels,
            bounds,
            upper_left,
            lower_right,
            num_logical_cores,
        );

        let duration = start.elapsed();
        total_duration += duration;
    }
    let average_duration = total_duration / iteration;
    println!("band {} {:?}", num_logical_cores, average_duration);

    // Bnads physical
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..iteration {
        let start = Instant::now();

        bands(
            &mut pixels,
            bounds,
            upper_left,
            lower_right,
            num_physical_cores,
        );

        let duration = start.elapsed();
        total_duration += duration;
    }
    let average_duration = total_duration / iteration;
    println!("band {} {:?}", num_physical_cores, average_duration);

    // -------------
    // Task queue logical
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..iteration {
        let start = Instant::now();

        task_queue(
            &mut pixels,
            bounds,
            upper_left,
            lower_right,
            num_logical_cores,
        );

        let duration = start.elapsed();
        total_duration += duration;
    }
    let average_duration = total_duration / iteration;
    println!("task queue {} {:?}", num_logical_cores, average_duration);

    // Task queue physical
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..iteration {
        let start = Instant::now();

        task_queue(
            &mut pixels,
            bounds,
            upper_left,
            lower_right,
            num_physical_cores,
        );

        let duration = start.elapsed();
        total_duration += duration;
    }
    let average_duration = total_duration / iteration;
    println!("task queue {} {:?}", num_physical_cores, average_duration);
}
