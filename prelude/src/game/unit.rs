//! Units are the vessels of Teams' wills
//!
//! Units occupy a Region, and are controlled by a Team.
//! New units may be placed every fall turn on an unoccupied home base, if the Team has more bases
//! than units.
//! Units may be Simple or Super.
//! Super units are granted every 3 bases + 1 for having all home bases
//!
//! # Unit types:
//! * Tank:
//!     * Simple unit
//!     * 1 strength
//!     * 1 move range on land
//!     * 1 support range on any
//! * Ship:
//!     * Simple unit
//!     * 1 strength
//!     * 1 move range on sea
//!     * 1 support range on sea
//! * Plane:
//!     * Super unit
//!     * 1 strength
//!     * 2 move range on any
//!         * If it starts a turn on a Sea region, and ends it on a Sea region, it runs out of
//!         fuel, and is removed.
//!     * 1 support range on any
//! * Supertank:
//!     * Super unit
//!     * 2 strength
//!     * 1 move range on land
//!     * 1 support range on any
//! * Submarine:
//!     * Super unit
//!     * 1 strength
//!         * 2 strength when attacking a neighboring Sea region
//!     * 2 move range on sea
//!     * 1 support range on sea
//! * Artillery:
//!     * Super unit
//!     * 1 strength
//!     * 1 move range on land
//!     * 2 support range on any
//!     * May bombard a region within support range - this is equivalent to placing a defending
//!     unit on that region, which blocks all units attempting to traverse that region.

use serde::{Deserialize, Serialize};

use super::region::Region;
use super::team::Team;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Tank,
    Ship,
    Plane,
    Supertank,
    Submarine,
    Artillery,
}

#[derive(Debug, Clone)]
pub struct Unit {
    unit_type: UnitType,
    region: Rc<Region>,
    owner: Rc<Team>,
}
