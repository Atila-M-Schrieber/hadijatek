use std::{cell::RefCell, error, fmt, rc::Rc};

use eyre::Result;
use petgraph::{csr::Csr, Undirected};
use prelude::{
    draw::{Color, Point, Shape},
    region::{Base, Border, Region, RegionType},
};

use crate::read::PreRegion;

/// Returns (this shape is a strait, &[points which comprise the shape])
/// A strait contains two pairs of identical points, which are one after another
fn strait(shape: &Shape) -> (bool, &[Point]) {
    let points = shape.points();
    // By default reference to empty slice
    let mut strait_points = &points[0..0];
    for i in 0..points.len() - 3 {
        let (p1, p2) = (points[i], points[i + 1]);
        for j in i..points.len() - 1 {
            let (p3, p4) = (points[j], points[j + 1]);
            if p1 == p4 && p2 == p3 {
                assert_eq!(
                    strait_points.len(),
                    0,
                    "More than one strait detected at {:?}",
                    strait_points
                );
                strait_points = &points[i..=i + 1];
            }
        }
    }

    (!strait_points.is_empty(), strait_points)
}

/// Returns true if all shapes in the slice have points in common
fn are_neighboring_shapes(shapes: &[&Shape]) -> bool {
    let mut are_neighbors = true;
    'outer: for (i, shape1) in shapes.iter().enumerate() {
        for shape2 in shapes.iter().skip(i + 1) {
            are_neighbors = are_neighbors
                && shape1
                    .into_iter()
                    .any(|p| shape2.into_iter().collect::<Vec<_>>().contains(&p));
            if !are_neighbors {
                break 'outer;
            }
        }
    }

    are_neighbors
}

/// Returns the region type of a (pre)region, so it can then be classified
fn classify(
    (_, _, shape, color): &PreRegion,
    others: &Csr<PreRegion, (), Undirected>,
    water_color: Color,
) -> RegionType {
    todo!()
}

/// Simply constructs a graph of all PreRegions
fn graphify(pre_regions: Vec<PreRegion>) -> Result<Csr<PreRegion, (), Undirected>> {
    let mut graph: Csr<PreRegion, (), Undirected> = Csr::new();
    let indeces: Vec<_> = pre_regions.into_iter().map(|n| graph.add_node(n)).collect();

    for &i in indeces.iter() {
        for &j in indeces.iter().skip(i as usize) {
            let (_, _, s1, _) = &graph[i];
            let (_, _, s2, _) = &graph[j];
            let ss = vec![s1, s2];
            if are_neighboring_shapes(&ss) {
                graph.add_edge(i, j, ());
            }
            // NEED TO CHECK FOR INTERNAL (ISLAND-LIKE) REGIONS
        }
    }

    for &i in indeces.iter() {
        if graph.neighbors_slice(i).is_empty() {
            let (_, _, s, _) = &graph[i];
            return Err(UnconnectedRegionError(s.clone()).into());
        }
    }

    Ok(graph)
}

pub fn mapify(
    pre_regions: Vec<PreRegion>,
    water_color: Color,
) -> Result<Csr<Rc<Region>, Border, Undirected>> {
    let graph = graphify(pre_regions)?;
    todo!()
}

#[derive(Debug)]
pub struct UnconnectedRegionError(Shape);

impl fmt::Display for UnconnectedRegionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Region of the following shape is not connected to anything else: {:?}",
            self.0
        )
    }
}

impl error::Error for UnconnectedRegionError {}
