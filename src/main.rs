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
    let mut z = Complex { re: 0.0, im: 0.0};
    for i in 0..limit {
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
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])){
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",           ','), None);
    assert_eq!(parse_pair::<i32>("10,",        ','), None);
    assert_eq!(parse_pair::<i32>(",10",        ','), None);
    assert_eq!(parse_pair::<i32>("10,20",      ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy",    ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",       'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5",   'x'), Some((0.5, 1.5)));
}

fn main() {
    println!("Hello, world!");
}
