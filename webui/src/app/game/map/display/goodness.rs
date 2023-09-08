use leptos::log;
use map_utils::{Color, Point, PreRegion};
use std::{collections::HashMap, f32::consts::TAU};

#[derive(Clone, Copy)]
pub(super) enum Position {
    Inside,
    Edge,
    Corner,
}

// Position of the preregion
pub fn get_pos(pr: &PreRegion, extent: (f32, f32)) -> Position {
    let horiz = [0.0, extent.0];
    let vert = [0.0, extent.1];
    let edges = [horiz, vert];
    let at_edge = |point: &Point, nth: usize| {
        let (x, y) = (*point).into();
        edges[nth].contains(&[x, y][nth])
    };

    let at_horiz = pr.shape.points().iter().any(|p| at_edge(p, 0));
    let at_vert = pr.shape.points().iter().any(|p| at_edge(p, 1));

    match (at_horiz, at_vert) {
        (true, true) => Position::Corner,
        (false, false) => Position::Inside,
        _ => Position::Edge,
    }
}

// calculating goodness
// How many neighbors compared to ideal (more is worse than less)
fn neighbor_deviation((position, neighbors): (Position, &HashMap<u32, (f32, f32)>)) -> f32 {
    let ideal_count = match position {
        Position::Inside => 6,
        Position::Edge => 3,
        Position::Corner => 3,
    };
    let diff = neighbors.len() as isize - ideal_count as isize;
    if diff < 0 {
        (-diff as f32).sqrt()
    } else {
        (diff * diff) as f32
    }
}

// Small differences in angles are really bad (overlapping border lines)
fn angular_deviation((position, neighbors): (Position, &HashMap<u32, (f32, f32)>)) -> f32 {
    let mut angles_and_lines: Vec<(f32, f32)> = neighbors.iter().map(|(_, &anl)| anl).collect();
    angles_and_lines.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

    // only the difference is important
    let (mut angles, mut lines): (Vec<f32>, Vec<f32>) = angles_and_lines
        .iter()
        .enumerate()
        .map(|(i, (l, a))| {
            let a2 = if i + 1 != angles_and_lines.len() {
                angles_and_lines[i + 1].1
            } else {
                angles_and_lines[0].1 + TAU // such that a2 > a
            };
            (l, a2 - a)
        })
        .unzip();
    let idx_biggest_gap = angles
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0;
    // now rotate around such that the biggest gap between angles is with the last element
    angles.rotate_left(idx_biggest_gap + 1);

    let (ideal_diff, ignore_last) = match position {
        Position::Inside => (TAU / 6.0, false),
        Position::Edge => (TAU / 6.0, true),
        Position::Corner => (TAU / 12.0, true),
    };

    // Saves on checking for if I'm at the end of the vec
    if !ignore_last {
        angles.push((angles[0] - angles[angles.len() - 1] + TAU) % TAU);
        lines.push(lines[0]); // damnit you
    }

    let weighed_diff = move |dev, (i, diff): (usize, &f32)| {
        let dev = dev + (ideal_diff - diff).abs();
        let punishment = (diff < &0.5) as u8 as f32
            * (0.5 - diff).exp()
            * lines[i]
            * lines[(i + 1) % lines.len()];
        dev + punishment // high penalty for low angles
    };

    let deviation = angles.iter().enumerate().fold(0.0, weighed_diff);

    deviation / angles.len() as f32
}

// How different lengths are
fn length_deviation((_, neighbors): (Position, &HashMap<u32, (f32, f32)>)) -> f32 {
    let avg_length: f32 =
        neighbors.values().map(|&(length, _)| length).sum::<f32>() / neighbors.len() as f32;
    let mut total_deviation = 0.0;
    for &(length, _) in neighbors.values() {
        total_deviation += (avg_length - length).abs();
    }
    total_deviation / neighbors.len() as f32
}

fn badness(args: (Position, &HashMap<u32, (f32, f32)>)) -> (f32, f32, f32) {
    (
        angular_deviation(args),
        neighbor_deviation(args),
        length_deviation(args),
    )
}

fn goodness(args: (Position, &HashMap<u32, (f32, f32)>)) -> (f32, f32, f32) {
    let (a, n, l) = badness(args);
    (1. / (1. + a), 1. / (1. + n), 1. / (1. + l))
}

fn normalized_goodness((a, n, l): (f32, f32, f32)) -> f32 {
    let angle_wegiht = 0.6;
    let num_neighbors_wegiht = 0.3;
    let length_spread_weight = 0.1;

    angle_wegiht * a + num_neighbors_wegiht * n + length_spread_weight * l
}

pub fn goodnesses(
    args: HashMap<u32, (Position, HashMap<u32, (f32, f32)>)>,
) -> HashMap<u32, ((Color, Color), f32)> {
    let goodnesses: HashMap<u32, ((f32, f32, f32), f32)> = args
        .into_iter()
        .map(|(i, (pos, hm))| {
            let goodness = goodness((pos, &hm));
            (i, (goodness, normalized_goodness(goodness)))
        })
        .collect();

    let gvs = || goodnesses.values();
    let pcmp = |a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal);
    let flatten: fn(&((f32, f32, f32), f32), usize) -> f32 =
        |((a, b, c), d): &((f32, f32, f32), f32), i| *[a, b, c, d][i];

    let worst = |i| {
        gvs()
            .map(|t| flatten(t, i))
            .min_by(|a, b| pcmp(a, b))
            .unwrap_or(0.)
    };
    let best = |i| {
        gvs()
            .map(|t| flatten(t, i))
            .max_by(|a, b| pcmp(a, b))
            .unwrap_or(1.)
    };

    let worst_angles = worst(0);
    let best_angles = best(0);

    let worst_neighbors = worst(1);
    let best_neighbors = best(1);

    let worst_lengths = worst(2);
    let best_lengths = best(2);

    let worst_goodness = worst(3);
    let best_goodness = best(3);

    // "absolute" color: lerp from between 0 and 1 to 0 and 255 for Color
    let colorize_rel = |min: f32, max: f32, x: f32| (min + 255. * (x - min) / (max - min)) as u8;
    let colorize_abs = |x: f32| colorize_rel(0., 1., x);

    log!("goodnesses calculated");

    let colorize = |((a, n, l), t)| {
        (
            (
                Color::new(colorize_abs(a), colorize_abs(n), colorize_abs(l)),
                Color::new(
                    colorize_rel(worst_angles, best_angles, a),
                    colorize_rel(worst_neighbors, best_neighbors, n),
                    colorize_rel(worst_lengths, best_lengths, l),
                ),
            ),
            t,
        )
    };

    goodnesses
        .into_iter()
        .map(|(k, v)| (k, colorize(v)))
        .collect()
}
