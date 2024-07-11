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

use rand::Rng;

/// Render random pixels.
fn random_render(pixels: &mut [u8], bounds: (usize, usize)) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    let mut rng = rand::thread_rng();

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            pixels[row * bounds.0 + column] = rng.gen();
        }
    }
}

use image::png::PNGEncoder;
use image::ColorType;
use std::fs::File;

/// Write the buffer 'pixels', whose dimensions are given by 'bounds', to the
/// file named 'filename'
fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    let color_type = if pixels.len() == bounds.0 * bounds.1 {
        ColorType::Gray(8)
    } else if pixels.len() == bounds.0 * bounds.1 * 3 {
        ColorType::RGB(8)
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid pixel data length for specified bounbds",
        ));
    };

    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, color_type)?;
    Ok(())
}

use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        args = vec![
            args[0].clone(),
            "rust_mandel.png".into(),
            "1920x1200".into(),
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
    render(&mut pixels, bounds, upper_left, lower_right);
    write_image(&args[1], &pixels, bounds).expect("error wrinting Mandelbrot PNG file");

    random_render(&mut pixels, bounds);
    write_image(&String::from("rand.png"), &pixels, bounds).expect("error writing random PNG file");

    let mut pixels_r = vec![0; bounds.0 * bounds.1];
    let mut pixels_g = vec![0; bounds.0 * bounds.1];
    let mut pixels_b = vec![0; bounds.0 * bounds.1];
    random_render(&mut pixels_r, bounds);
    random_render(&mut pixels_g, bounds);
    random_render(&mut pixels_b, bounds);

    let mut pixels_rgb = Vec::with_capacity(bounds.0 * bounds.1 * 3);
    for i in 0..bounds.0 * bounds.1 {
        pixels_rgb.push(pixels_r[i]);
        pixels_rgb.push(pixels_g[i]);
        pixels_rgb.push(pixels_b[i]);
    }
    write_image(&String::from("rand_rgb.png"), &pixels_rgb, bounds)
        .expect("error writing random RGB PNG file");
}
