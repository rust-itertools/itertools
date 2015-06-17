///
/// This example parses, sorts and groups the iris dataset
/// and does some simple manipulations.
///
/// Iterators and itertools functionality are used throughout.
///
/// 

extern crate itertools;

use itertools::Itertools;
use std::str::FromStr;
use std::collections::HashMap;
use std::num::ParseFloatError;

static DATA: &'static str = include_str!("iris.data");

#[derive(Clone, Debug)]
struct Iris {
    name: String,
    data: [f32; 4],
}

#[derive(Clone, Debug)]
enum ParseError {
    Numeric(ParseFloatError),
    Other(&'static str),
}

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        ParseError::Numeric(err)
    }
}

/// Parse an Iris from a comma-separated line
impl FromStr for Iris {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iris = Iris { name: "".into(), data: [0.; 4] };
        let mut parts = s.split(",").map(str::trim);

        // using Iterator::by_ref()
        for (index, part) in parts.by_ref().take(4).enumerate() {
            iris.data[index] = try!(part.parse::<f32>());
        }
        if let Some(name) = parts.next() {
            iris.name = name.into();
        } else {
            return Err(ParseError::Other("Missing name"))
        }
        Ok(iris)
    }
}

fn main() {
    // using itertools::Itertools::fold_results to create the result of parsing
    let irises = DATA.lines()
                     .map(str::parse)
                     .fold_results(Vec::new(), |mut v, iris: Iris| {
                         v.push(iris);
                         v
                     }).unwrap();

    // Sort them and group them
    let mut irises = irises;
    irises.sort_by(|a, b| Ord::cmp(&a.name, &b.name));

    // using Iterator::cycle()
    let mut plot_symbols = "+ox".chars().cycle();
    let mut symbolmap = HashMap::new();

    // using itertools::Itertools::group_by
    for (species, species_data) in irises.iter().group_by(|iris| &iris.name) {
        // assign a plot symbol
        symbolmap.entry(species).or_insert_with(|| {
            plot_symbols.next().unwrap()
        });

        println!("Species = {} has symbol {}", species, symbolmap[species]);

        for column in 0..4 {
            for &iris in &species_data {
                print!("{:>3.1}, ", iris.data[column]);
            }
            println!("");
        }

    }

    // Look at all combinations of the four columns
    //
    // See https://en.wikipedia.org/wiki/Iris_flower_data_set
    //
    // using itertools::Itertools::combinations
    for (a, b) in (0..4).combinations() {
        println!("Column {} vs {}:", a, b);
        let n = 30;
        let mut plot = vec![' '; n * n];

        // using itertools::Itertools::fold1
        let min_max = |vec: &[Iris], col| {
            vec.iter()
               .map(move |iris| iris.data[col])
               .map(|x| (x, x))
               .fold1(|(min, max), (elt, _)|
                   (f32::min(min, elt), f32::max(max, elt))
               ).expect("Can't find min/max of empty iterator")
        };
        let (min_x, max_x) = min_max(&irises, a);
        let (min_y, max_y) = min_max(&irises, b);

        for (symbol, x, y) in irises.iter()
            .map(|ir| (symbolmap[&ir.name], ir.data[a], ir.data[b]))
        {
            // round to the grid
            let ix = ((x - min_x) / (max_x - min_x) * ((n - 1) as f32)) as usize;
            let iy = ((y - min_y) / (max_y - min_y) * ((n - 1) as f32)) as usize;
            let iy = n - 1 - iy; // reverse y axis' direction

            plot[n * iy + ix] = symbol;
        }

        // render plot
        //
        // using itertools::Itertools::join
        for line in plot.chunks(n) {
            println!("{}", line.iter().join(" "))
        }
    }
}
