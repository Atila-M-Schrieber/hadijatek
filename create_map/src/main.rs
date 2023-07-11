mod read;

use crate::read::*;

fn main() {
    let path = "svg.svg";

    get_regions(path).unwrap();
}
