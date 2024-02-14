use std::{env::args, io::stdin};

fn main() -> Result<(), u32> {
    let source = args().nth(1).ok_or(1u32)?;
    if source == "--config" {
        print!("{}", include_str!("config.kak"));
        return Ok(())
    }
    for line in stdin().lines() {
        let line = line.map_err(|_| 1u32)?;
        println!("{}", recase(&source, &line));
    }
    Ok(())
}

/// Changes the casing of `target` to placewise match that of `source`.
/// If `target` is longer than `source`,
/// the last character of `source` is reused for the remainder.
fn recase(target: &str, source: &str) -> String {
    let mut res = String::new();

    // First match the strings till one of them runs out
    for (t, c) in target.chars().zip(source.chars()) {
        if c.is_uppercase() {
            res.extend(t.to_uppercase());
        } else {
            res.extend(t.to_lowercase());
        }
    }

    // Then handle any remaining length of the target, if any
    let upper = source.chars().last().unwrap_or(' ').is_uppercase();
    let rem = target.chars();
    let rem = rem.skip(source.chars().count());
    for t in rem {
        if upper {
            res.extend(t.to_uppercase());
        } else {
            res.extend(t.to_lowercase());
        }
    }

    res
}
