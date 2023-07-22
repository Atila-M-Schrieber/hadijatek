use std::{error, fmt, rc::Rc};

use eyre::Result;
use itertools::Itertools;
use petgraph::{csr::Csr, visit::IntoNodeReferences, Undirected};
use prelude::{
    draw::{Color, Point, Shape},
    region::{Border, Region, RegionType},
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

/// Simply constructs a graph of all PreRegions
fn graphify(pre_regions: Vec<PreRegion>) -> Result<Csr<PreRegion, (), Undirected>> {
    let mut graph: Csr<PreRegion, (), Undirected> = Csr::new();
    let indeces: Vec<_> = pre_regions.into_iter().map(|n| graph.add_node(n)).collect();

    for &i in indeces.iter() {
        for &j in indeces.iter().skip(i as usize) {
            let s1 = &graph[i].2;
            let s2 = &graph[j].2;
            let ss = vec![s1, s2];
            if are_neighboring_shapes(&ss) {
                graph.add_edge(i, j, ());
            }
            // NEED TO CHECK FOR INTERNAL (ISLAND-LIKE) REGIONS
        }
    }

    for &i in indeces.iter() {
        if graph.neighbors_slice(i).is_empty() {
            let s = &graph[i].2;
            return Err(UnconnectedRegionError(s.clone()).into());
        }
    }

    Ok(graph)
}

/// Turns pre-regions into full regions
fn to_full_regions(
    graph: Csr<PreRegion, (), Undirected>,
    water_color: Color,
) -> Result<Csr<Rc<Region>, Border, Undirected>> {
    let mut new_graph: Csr<Rc<Region>, Border, Undirected> = Csr::new();

    for (i, (name, base, shape, color, _)) in graph.node_references() {
        use RegionType::*;
        let mut rtype = Land;
        if color == &water_color {
            rtype = Sea;
        } else if let (true, _) = strait(shape) {
            rtype = Strait;
        } else if graph.neighbors_slice(i).iter().any(|&i| {
            let color = graph[i].3;
            color == water_color
        }) {
            rtype = Shore
        }
        new_graph.add_node(Rc::new(Region::new(
            name.clone(),
            rtype,
            base.clone(),
            shape.clone(),
            *color,
        )?));
    }

    for (i, _) in graph.node_references() {
        let i_neighbors = graph.neighbors_slice(i);
        for &j in i_neighbors.iter() {
            let j_neighbors: Vec<_> = graph
                .neighbors_slice(j)
                .iter()
                // .map(|&k| new_indeces[k as usize])
                .collect();
            let mut common_neighbors = Vec::new();
            for &l in i_neighbors {
                if j_neighbors.contains(&&l) {
                    common_neighbors.push(Rc::clone(&new_graph[l]));
                }
            }

            new_graph.add_edge(
                i,
                j,
                get_border(
                    Rc::clone(&new_graph[i]),
                    Rc::clone(&new_graph[j]),
                    &common_neighbors,
                ),
            );
        }
        println!(
            "{}: {}",
            &new_graph[i].name(),
            &new_graph
                .neighbors_slice(i)
                .iter()
                .map(|&j| new_graph[j].name().to_string())
                .join(", ")
        )
    }

    assert_eq!(graph.node_count(), new_graph.node_count());
    assert_eq!(graph.edge_count(), new_graph.edge_count());

    Ok(new_graph)
}

/// Classifies the border between two regions
/// If i != i_old then must map
fn get_border(r1: Rc<Region>, r2: Rc<Region>, common_neighbors: &[Rc<Region>]) -> Border {
    use Border as B;
    use RegionType::*;
    match (r1.region_type(), r2.region_type()) {
        (Land, _) | (_, Land) => B::Land,
        (Sea, Sea) => {
            // check for strait
            if let Some(strait) = common_neighbors.iter().find(|&region| {
                strait(region.shape()).0
                    && are_neighboring_shapes(&[region.shape(), r1.shape(), r2.shape()])
            }) {
                return B::Strait(Rc::clone(strait));
            }
            B::Sea
        }
        (Sea, _) | (_, Sea) => B::Sea,
        (Shore, Shore) | (Shore, Strait) | (Strait, Shore) | (Strait, Strait) => {
            // check if wo'ah traversible
            if common_neighbors.iter().any(|region| {
                region.region_type() == Sea
                    && are_neighboring_shapes(&[region.shape(), r1.shape(), r2.shape()])
            }) {
                return B::Shore;
            }
            B::Land
        }
    }
}

/// Turns a list of PreRegions into a complete map
pub fn mapify(
    pre_regions: Vec<PreRegion>,
    water_color: Color,
) -> Result<Csr<Rc<Region>, Border, Undirected>> {
    let graph = graphify(pre_regions)?;
    let graph = to_full_regions(graph, water_color)?;
    Ok(graph)
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
