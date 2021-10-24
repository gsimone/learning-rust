use num::Complex;
use std::str::FromStr;

/// Determines wether c belongs in the mandlebrot set, using at most "limit" iterations
///
/// If c is not a member, return Some(i) where i is the number of iterations
/// it took for c to leave the circle of radius 2 around the origin.__rust_force_expr!
///
/// If c is a member, (reached the iteration limit without proving it's NOT a member)
/// we return None.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }

        z = z * z + c;
    }

    None
}

#[test]
fn escape_time_test() {
    assert_eq!(escape_time(Complex { re: 0.0, im: 0.0 }, 1000), None);

    assert_eq!(escape_time(Complex { re: 400., im: 600. }, 1000), Some(1));
}

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    // this will match if it finds the separator anywhere in the string
    match s.find(separator) {
        None => None,
        // index is the index of the matched 'separator' char
        Some(index) => {
            // notice how we use index to get the left and right members
            // so it creates a string from the s reference from TO index ( ..index )
            // and another one FROM index+1
            // FROM..TO
            // in this case, 'index' would be the separator

            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                // if both l and r are matched, return them
                (Ok(l), Ok(r)) => Some((l, r)),
                // this wildcard matcher will match if the other conditions
                // aren't met (eg. one of the tuple is not Ok())
                _ => None,
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    // NOTE: notice the ' around , => this is to indicate a character.assert_eq!
    // using "," would make it be a string (failing compilation)
    assert_eq!(parse_pair::<i32>("10,10", ','), Some((10, 10)));

    assert_eq!(parse_pair::<i32>("10.,10", ','), None);
    assert_eq!(parse_pair::<i32>("10.", ','), None);
    assert_eq!(parse_pair::<i32>("10.,", ','), None);

    assert_eq!(parse_pair::<f64>("0.5x0.5", 'x'), Some((0.5, 0.5)));
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,10.42"),
        Some(Complex {
            re: 1.25,
            im: 10.42
        })
    );
    assert_eq!(
        parse_complex("2,10.42"),
        Some(Complex { re: 2., im: 10.42 })
    );
}

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
            Complex { re: -1., im: 1. },
            Complex { re: 1., im: -1. }
        ),
        { Complex { re: 0.5, im: -0.75 } }
    )
}

// Render a rectangle of the mandlebrot set into  a buffer of pixels
fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {

    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for col in 0..bounds.0 {
            let point = pixel_to_point(bounds, (col, row), upper_left, lower_right);
            pixels[row * bounds.0 + col] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8
            }
        }
    }

}

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize)
) -> Result<(), std::io::Error> {

    // 1. create a file with filename
    // NOTE: The ? here at the end of the statement matches the return of the statement
    // if it would return an error, the function returns with the error!

    // let x = match Whatever::something() {
    //    Ok(f) => f,
    //    Err(e) => {
    //       return Err(e); 
    //    }
    // }
    let output = File::create(filename)?;

    // setup encoder passing the created file
    let encoder = PNGEncoder::new(output);

    // encode pixels into file
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;
    
    Ok(())
}

use std::env;

fn main() {
    // get all args in a vector
    let args: Vec<String>  = env::args().collect();

    if args.len() != 5 {
        eprintln!("Invalid input");
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");

    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");

    let lower_right = parse_complex(&args[4])
        .expect("error parsing upper left corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    // let's go
    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;

    {
        let bands: Vec<&mut [u8]> = 
            pixels.chunks_mut(rows_per_band * bounds.0).collect();
        
        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {

                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0,top), upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top+height), upper_left, lower_right);

                spawner.spawn(move |_| {
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });

            }
        }).unwrap();
    }

    write_image(&args[1], &pixels, bounds)
        .expect("error writing PNG file");

}