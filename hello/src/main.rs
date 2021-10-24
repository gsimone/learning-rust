use std::str::FromStr;
use std::env;

fn main() {

    // notice how Vec is a generic but it's T is inferred when pushing a u64 in the next lines
    let mut numbers = Vec::new();

    for arg in env::args().skip(1) {
        numbers.push(u64::from_str(&arg).expect("error parsing argument"))
    }

    if numbers.len() == 0 {
        eprintln!("Usage: gcd NUMBER ...");
    }

    // take the first item in numbers
    let mut d = numbers[0];

    // borrow the number member as reference
    for m in &numbers[1..] {
        // the * operator yields a value from the reference for m
        d = gcd(d, *m);
    }

    println!("The gdc of {:?} is {}", numbers, d);
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }

        m = m % n;
    }

    n
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(4, 10), 2);
}