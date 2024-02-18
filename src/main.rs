use std::{env::args, io::stdin};

use strum::{EnumCount, EnumIter, IntoEnumIterator};

fn main() -> Result<(), u32> {
    let args = args().collect::<Vec<String>>();
    let source = args.get(1).ok_or(1u32)?;
    if source == "--generate-config" {
        print!("{}", include_str!("config.kak"));
        return Ok(());
    }

    let priorities: Result<Vec<Style>, ()> = args[2..]
        .iter()
        .map(|s| Style::try_from(s.as_str()))
        .collect();
    let priorities = priorities.map_err(|_| 2u32)?;

    for line in stdin().lines() {
        let line = line.map_err(|_| 1u32)?;
        println!("{}", recase(source, &line, &priorities));
    }
    Ok(())
}

/// Changes the casing of `target` to index-wise match that of `source`.
/// If `target` is longer than `source`,
/// the last character of `source` is reused for the remainder.
fn recase_naive(target: &str, source: &str) -> String {
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

fn recase(target: &str, source: &str, priorities: &[Style]) -> String {
    let style = identify(source, priorities);
    construct(target, style)
}

fn construct(mut ident: &str, style: Style) -> String {
    let mut res = String::new();

    if ident.as_bytes()[0] == b' ' {
        style.pre.add_to(&mut res);
        ident = &ident[1..];
    }

    let mut new_word = style.case != Case::Camel;
    for c in ident.chars() {
        if "_- ".contains(c) {
            style.sep.add_to(&mut res);
            new_word = true;
        } else {
            let uppercase = match (style.case, new_word) {
                (Case::Lower, _) => false,
                (Case::AllCaps, _) => true,
                (_, w) => w,
            };
            if uppercase {
                res.extend(c.to_uppercase());
            } else {
                res.push(c);
            }
            new_word = false;
        }
    }
    res
}

fn identify(mut ident: &str, priorities: &[Style]) -> Style {
    let mut possibs = Possibs::new();

    // Handle pre-separator (typically an unused indicator)
    let pre = match ident.as_bytes()[0] {
        b'_' => Sep::Snake,
        b'-' => Sep::Kebab,
        b' ' => Sep::Space,
        _ => Sep::None,
    };
    if pre != Sep::None {
        ident = &ident[1..];
        possibs.set_pre(pre);
    }

    // Rule out either camelCase or CapsCase based on the first character.
    if ident.chars().next().is_some_and(|c| c.is_uppercase()) {
        possibs.rem_case(Case::Camel);
    } else {
        possibs.rem_case(Case::Caps);
    }

    for c in ident.chars() {
        // The separator is chosen to be the latest one seen.
        match c {
            '_' => possibs.set_sep(Sep::Snake),
            '-' => possibs.set_sep(Sep::Kebab),
            ' ' => possibs.set_sep(Sep::Space),
            _ => {}
        }

        // Rule out uppercase and lowercase.
        if c.is_lowercase() {
            possibs.rem_case(Case::AllCaps);
        } else if c.is_uppercase() {
            possibs.rem_case(Case::Lower);
        }
    }

    // List of all possible styles.
    let all = Sep::iter()
        .zip(Case::iter())
        .zip(Sep::iter())
        .map(|((pre, case), sep)| Style { pre, case, sep });

    // Find a style in the priorities, otherwise find any that matches.
    for style in priorities.iter().cloned().chain(all) {
        if possibs.include(style) {
            return style;
        }
    }

    // If all else fail, use a fallback.
    Style {
        pre: Sep::Snake,
        case: Case::Lower,
        sep: Sep::Snake,
    }
}

struct Possibs {
    pre: [bool; Sep::COUNT],
    case: [bool; Case::COUNT],
    sep: [bool; Sep::COUNT],
}

impl Possibs {
    /// New set containing all possibilities
    fn new() -> Self {
        Self {
            pre: [true; Sep::COUNT],
            case: [true; Case::COUNT],
            sep: [true; Sep::COUNT],
        }
    }

    fn include(&self, style: Style) -> bool {
        self.pre[style.pre as usize]
            && self.case[style.case as usize]
            && self.sep[style.sep as usize]
    }

    fn set_pre(&mut self, pre: Sep) {
        for p in Sep::iter() {
            self.pre[pre as usize] = p == pre;
        }
    }

    fn set_sep(&mut self, sep: Sep) {
        for p in Sep::iter() {
            self.sep[sep as usize] = p == sep;
        }
    }

    fn rem_case(&mut self, case: Case) {
        self.case[case as usize] = false;
    }
}

/// The internal string should be formatted in lowercase with separating spaces.
/// A pre-separator is indicated by a leading space.
/// Uppercase, dashes, and underscores overrule whichever style is used.
struct Ident(str);

/// Identifies an identifier style.
#[derive(Clone, Copy)]
struct Style {
    pre: Sep,
    case: Case,
    sep: Sep,
}

impl TryFrom<&str> for Style {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        let mut bytes = value.as_bytes();

        // Pre-separator
        let pre = match bytes[0] {
            b'_' => Sep::Snake,
            b'-' => Sep::Kebab,
            b' ' => Sep::Space,
            _ => Sep::None,
        };

        if pre != Sep::None {
            bytes = &bytes[1..];
        }

        // First letter
        let first = match bytes[0] {
            b'a' => false,
            b'A' => true,
            _ => return Err(()),
        };

        // Separator
        let sep = match bytes[1] {
            b'_' => Sep::Snake,
            b'-' => Sep::Kebab,
            b' ' => Sep::Space,
            _ => Sep::None,
        };

        if sep != Sep::None {
            bytes = &bytes[1..];
        }

        // Second letter
        let second = match bytes[1] {
            b'a' => false,
            b'A' => true,
            _ => return Err(()),
        };

        // Find case
        let case = match (first, second) {
            (true, true) => Case::AllCaps,
            (true, false) => Case::Caps,
            (false, true) => Case::Camel,
            (false, false) => Case::Lower,
        };

        Ok(Style { pre, case, sep })
    }
}

/// Which letters to uppercase.
#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter)]
enum Case {
    /// All letters are lowercase.
    Lower,
    /// The first letter in subsequent words are uppercase.
    Camel,
    /// The first letter all words are uppercase.
    Caps,
    /// All letters are uppercase.
    AllCaps,
}

/// Separator types.
#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter)]
enum Sep {
    /// No separator at all.
    None,
    /// An underscore.
    Snake,
    /// A dash.
    Kebab,
    /// A space.
    Space,
}

impl Sep {
    fn add_to(&self, ident: &mut String) {
        match self {
            Sep::None => {}
            Sep::Snake => ident.push('_'),
            Sep::Kebab => ident.push('-'),
            Sep::Space => ident.push(' '),
        }
    }
}
