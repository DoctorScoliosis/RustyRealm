use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

/// Stable, machine-facing identity for a world location.
///
/// Display names may change without changing a location's identity. The
/// transparent serde representation keeps the persisted form a plain string
/// when this ID is added to save state in a later implementation slice.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocationId(String);

impl LocationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl From<&str> for LocationId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for LocationId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for LocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationCategory {
    Settlement,
    Wilderness,
    Ruins,
    Dungeon,
    Landmark,
}

/// Capabilities a location explicitly provides. Location category is descriptive;
/// it never implies a service such as lodging, healing or safe camping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationService {
    Tavern,
    Guildhouse,
    Bed,
    Healer,
    FieldCamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub description: String,
    pub category: LocationCategory,
    /// Explicit services; legacy location data defaults to providing none.
    #[serde(default)]
    pub services: Vec<LocationService>,
}

impl Location {
    pub fn new(
        id: impl Into<LocationId>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: LocationCategory,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category,
            services: Vec::new(),
        }
    }

    /// Declare the exact services available here. Duplicates are removed so
    /// eligibility checks remain deterministic.
    pub fn with_services(mut self, services: impl IntoIterator<Item = LocationService>) -> Self {
        for service in services {
            if !self.services.contains(&service) {
                self.services.push(service);
            }
        }
        self
    }

    pub fn has_service(&self, service: LocationService) -> bool {
        self.services.contains(&service)
    }
}

/// A directed edge from its owning location to its destination ID.
///
/// travel_cost is the elapsed in-game travel duration in hours. Directionality
/// is explicit: use World::add_bidirectional_route for two-way travel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub destination_id: LocationId,
    /// Elapsed in-game travel hours for this directed route.
    pub travel_cost: u32,
    pub danger_level: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachableDestination {
    pub location_id: LocationId,
    /// Minimum cumulative travel cost from the query's starting location.
    pub total_travel_cost: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    InvalidLocationId,
    InvalidLocationName(LocationId),
    DuplicateLocation(LocationId),
    UnknownLocation(LocationId),
    SelfRoute(LocationId),
    ZeroTravelCost,
    DuplicateRoute {
        from: LocationId,
        to: LocationId,
    },
    LocationIdMismatch {
        key: LocationId,
        definition_id: LocationId,
    },
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLocationId => f.write_str("Location ID cannot be blank."),
            Self::InvalidLocationName(id) => {
                write!(f, "Location '{id}' must have a non-blank display name.")
            }
            Self::DuplicateLocation(id) => write!(f, "Location ID '{id}' already exists."),
            Self::UnknownLocation(id) => write!(f, "Unknown location ID '{id}'."),
            Self::SelfRoute(id) => write!(f, "Location '{id}' cannot route to itself."),
            Self::ZeroTravelCost => f.write_str("Travel cost must be greater than zero."),
            Self::DuplicateRoute { from, to } => {
                write!(f, "A route from '{from}' to '{to}' already exists.")
            }
            Self::LocationIdMismatch { key, definition_id } => write!(
                f,
                "Location map key '{key}' does not match definition ID '{definition_id}'."
            ),
        }
    }
}

impl std::error::Error for WorldError {}

/// Data-driven world graph and pure queries over its locations and routes.
///
/// The graph is private so callers cannot bypass validation by mutating the
/// maps directly. Routes are directed unless explicitly added bidirectionally.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct World {
    locations: HashMap<LocationId, Location>,
    routes: HashMap<LocationId, Vec<Route>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_location(&mut self, location: Location) -> Result<(), WorldError> {
        if location.id.is_blank() {
            return Err(WorldError::InvalidLocationId);
        }
        if location.name.trim().is_empty() {
            return Err(WorldError::InvalidLocationName(location.id));
        }
        if self.locations.contains_key(&location.id) {
            return Err(WorldError::DuplicateLocation(location.id));
        }

        self.routes.entry(location.id.clone()).or_default();
        self.locations.insert(location.id.clone(), location);
        Ok(())
    }

    pub fn add_route(
        &mut self,
        from_id: &LocationId,
        to_id: &LocationId,
        travel_cost: u32,
        danger_level: u32,
    ) -> Result<(), WorldError> {
        self.validate_new_route(from_id, to_id, travel_cost)?;
        self.routes.entry(from_id.clone()).or_default().push(Route {
            destination_id: to_id.clone(),
            travel_cost,
            danger_level,
        });
        Ok(())
    }

    /// Adds both directed edges, validating both before mutating the graph.
    /// If either direction is invalid or already exists, neither edge is added.
    pub fn add_bidirectional_route(
        &mut self,
        first_id: &LocationId,
        second_id: &LocationId,
        travel_cost: u32,
        danger_level: u32,
    ) -> Result<(), WorldError> {
        self.validate_new_route(first_id, second_id, travel_cost)?;
        self.validate_new_route(second_id, first_id, travel_cost)?;

        self.routes
            .entry(first_id.clone())
            .or_default()
            .push(Route {
                destination_id: second_id.clone(),
                travel_cost,
                danger_level,
            });
        self.routes
            .entry(second_id.clone())
            .or_default()
            .push(Route {
                destination_id: first_id.clone(),
                travel_cost,
                danger_level,
            });
        Ok(())
    }

    fn validate_new_route(
        &self,
        from_id: &LocationId,
        to_id: &LocationId,
        travel_cost: u32,
    ) -> Result<(), WorldError> {
        if !self.locations.contains_key(from_id) {
            return Err(WorldError::UnknownLocation(from_id.clone()));
        }
        if !self.locations.contains_key(to_id) {
            return Err(WorldError::UnknownLocation(to_id.clone()));
        }
        if from_id == to_id {
            return Err(WorldError::SelfRoute(from_id.clone()));
        }
        if travel_cost == 0 {
            return Err(WorldError::ZeroTravelCost);
        }
        if self
            .routes
            .get(from_id)
            .is_some_and(|routes| routes.iter().any(|route| &route.destination_id == to_id))
        {
            return Err(WorldError::DuplicateRoute {
                from: from_id.clone(),
                to: to_id.clone(),
            });
        }
        Ok(())
    }

    /// Validates the complete in-memory graph. The insertion methods reject
    /// invalid definitions early; this method also provides a testable invariant
    /// gate for the static region and future data-loading paths.
    pub fn validate(&self) -> Result<(), WorldError> {
        for (key, location) in &self.locations {
            if key != &location.id {
                return Err(WorldError::LocationIdMismatch {
                    key: key.clone(),
                    definition_id: location.id.clone(),
                });
            }
            if location.id.is_blank() {
                return Err(WorldError::InvalidLocationId);
            }
            if location.name.trim().is_empty() {
                return Err(WorldError::InvalidLocationName(location.id.clone()));
            }
        }

        for (from_id, routes) in &self.routes {
            if !self.locations.contains_key(from_id) {
                return Err(WorldError::UnknownLocation(from_id.clone()));
            }

            let mut destinations = HashSet::new();
            for route in routes {
                if !self.locations.contains_key(&route.destination_id) {
                    return Err(WorldError::UnknownLocation(route.destination_id.clone()));
                }
                if from_id == &route.destination_id {
                    return Err(WorldError::SelfRoute(from_id.clone()));
                }
                if route.travel_cost == 0 {
                    return Err(WorldError::ZeroTravelCost);
                }
                if !destinations.insert(route.destination_id.clone()) {
                    return Err(WorldError::DuplicateRoute {
                        from: from_id.clone(),
                        to: route.destination_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn get_location(&self, id: &LocationId) -> Option<&Location> {
        self.locations.get(id)
    }

    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Returns routes leaving this location. An unknown ID yields an empty
    /// slice, allowing read-only callers to query without panicking.
    pub fn get_direct_routes(&self, current_location: &LocationId) -> &[Route] {
        self.routes
            .get(current_location)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns locations reachable in one move, ordered by stable location ID.
    pub fn get_direct_destinations(&self, current_location: &LocationId) -> Vec<&Location> {
        let mut destinations: Vec<&Location> = self
            .get_direct_routes(current_location)
            .iter()
            .filter_map(|route| self.locations.get(&route.destination_id))
            .collect();
        destinations.sort_by(|left, right| left.id.cmp(&right.id));
        destinations
    }

    /// Returns every location reachable through one or more graph edges except
    /// the starting location, ordered by minimum total route cost and then ID.
    pub fn get_reachable_destinations(
        &self,
        start_location: &LocationId,
    ) -> Vec<ReachableDestination> {
        let distances = self.shortest_distances(start_location);
        let mut destinations: Vec<ReachableDestination> = distances
            .into_iter()
            .filter(|(location_id, _)| location_id != start_location)
            .map(|(location_id, total_travel_cost)| ReachableDestination {
                location_id,
                total_travel_cost,
            })
            .collect();
        destinations.sort_by(|left, right| {
            left.total_travel_cost
                .cmp(&right.total_travel_cost)
                .then_with(|| left.location_id.cmp(&right.location_id))
        });
        destinations
    }

    /// Returns whether a single direct route exists from from_id to to_id.
    pub fn can_travel(&self, from_id: &LocationId, to_id: &LocationId) -> bool {
        self.get_direct_routes(from_id)
            .iter()
            .any(|route| &route.destination_id == to_id)
    }

    /// Returns whether to_id can be reached through zero or more routes.
    /// An existing location is reachable from itself at cost zero.
    pub fn is_reachable(&self, from_id: &LocationId, to_id: &LocationId) -> bool {
        self.shortest_distances(from_id).contains_key(to_id)
    }

    /// Finds the reachable location with an explicit service, preferring the
    /// lowest cumulative route cost and then the stable location ID.
    pub fn get_nearest_location_with_service(
        &self,
        start_location: &LocationId,
        service: LocationService,
    ) -> Option<LocationId> {
        self.shortest_distances(start_location)
            .into_iter()
            .filter(|(id, _)| {
                self.locations
                    .get(id)
                    .is_some_and(|location| location.has_service(service))
            })
            .min_by(|(left_id, left_cost), (right_id, right_cost)| {
                left_cost
                    .cmp(right_cost)
                    .then_with(|| left_id.cmp(right_id))
            })
            .map(|(id, _)| id)
    }

    /// Finds the nearest reachable healer/treatment facility. A settlement
    /// without the explicit Healer service is not eligible.
    pub fn get_nearest_treatment_location(
        &self,
        start_location: &LocationId,
    ) -> Option<LocationId> {
        self.get_nearest_location_with_service(start_location, LocationService::Healer)
    }

    /// Finds the reachable settlement with the lowest cumulative travel cost.
    /// Equal-cost settlements are resolved by lexicographically smallest stable
    /// location ID, independent of HashMap iteration order.
    pub fn get_nearest_settlement(&self, start_location: &LocationId) -> Option<LocationId> {
        self.shortest_distances(start_location)
            .into_iter()
            .filter(|(id, _)| {
                self.locations
                    .get(id)
                    .is_some_and(|location| location.category == LocationCategory::Settlement)
            })
            .min_by(|(left_id, left_cost), (right_id, right_cost)| {
                left_cost
                    .cmp(right_cost)
                    .then_with(|| left_id.cmp(right_id))
            })
            .map(|(id, _)| id)
    }

    fn shortest_distances(&self, start_location: &LocationId) -> HashMap<LocationId, u64> {
        if !self.locations.contains_key(start_location) {
            return HashMap::new();
        }

        let mut distances = HashMap::new();
        let mut queue = BinaryHeap::new();

        distances.insert(start_location.clone(), 0_u64);
        queue.push(Reverse((0_u64, start_location.clone())));

        while let Some(Reverse((current_cost, current_id))) = queue.pop() {
            if distances
                .get(&current_id)
                .is_some_and(|known_cost| current_cost > *known_cost)
            {
                continue;
            }

            for route in self.get_direct_routes(&current_id) {
                let next_cost = current_cost.saturating_add(u64::from(route.travel_cost));
                let should_update = distances
                    .get(&route.destination_id)
                    .is_none_or(|known_cost| next_cost < *known_cost);

                if should_update {
                    distances.insert(route.destination_id.clone(), next_cost);
                    queue.push(Reverse((next_cost, route.destination_id.clone())));
                }
            }
        }

        distances
    }
}

/// Builds a small static region used to exercise the graph before save/UI
/// integration. The village has three nearby landmarks, a dangerous wilderness
/// route, a ruins site, and a dungeon destination.
pub fn build_initial_world() -> Result<World, WorldError> {
    let mut world = World::new();

    let village = LocationId::from("village_start");
    let watchtower = LocationId::from("crumbling_watchtower");
    let standing_stones = LocationId::from("standing_stones");
    let abandoned_mill = LocationId::from("abandoned_mill");
    let sunken_ruins = LocationId::from("sunken_ruins");
    let whispering_woods = LocationId::from("whispering_woods");
    let goblin_caves = LocationId::from("goblin_caves");

    for location in [
        Location::new(
            village.clone(),
            "Oakhaven Village",
            "A quiet settlement surrounded by sturdy oak trees.",
            LocationCategory::Settlement,
        )
        .with_services([
            LocationService::Tavern,
            LocationService::Guildhouse,
            LocationService::Bed,
            LocationService::Healer,
        ]),
        Location::new(
            watchtower.clone(),
            "Crumbling Watchtower",
            "An old watchtower overlooking the valley.",
            LocationCategory::Landmark,
        )
        .with_services([LocationService::FieldCamp]),
        Location::new(
            standing_stones.clone(),
            "Standing Stones",
            "A ring of weathered stones beside the old road.",
            LocationCategory::Landmark,
        )
        .with_services([LocationService::FieldCamp]),
        Location::new(
            abandoned_mill.clone(),
            "Abandoned Mill",
            "A disused mill with a view over the village fields.",
            LocationCategory::Landmark,
        )
        .with_services([LocationService::FieldCamp]),
        Location::new(
            sunken_ruins.clone(),
            "Sunken Ruins",
            "Ancient stonework half-swallowed by the forest floor.",
            LocationCategory::Ruins,
        ),
        Location::new(
            whispering_woods.clone(),
            "Whispering Woods",
            "A dense forest where the wind sounds like distant voices.",
            LocationCategory::Wilderness,
        )
        .with_services([LocationService::FieldCamp]),
        Location::new(
            goblin_caves.clone(),
            "Goblin Caves",
            "A cavern system known to shelter hostile creatures.",
            LocationCategory::Dungeon,
        ),
    ] {
        world.add_location(location)?;
    }

    world.add_bidirectional_route(&village, &watchtower, 8, 0)?;
    world.add_bidirectional_route(&village, &standing_stones, 6, 1)?;
    world.add_bidirectional_route(&village, &abandoned_mill, 5, 1)?;
    world.add_bidirectional_route(&village, &whispering_woods, 20, 3)?;
    world.add_bidirectional_route(&whispering_woods, &sunken_ruins, 12, 4)?;
    world.add_bidirectional_route(&whispering_woods, &goblin_caves, 15, 5)?;
    // A second high-danger route connects the ruins to the caves without letting
    // a starting party bypass the wilderness.
    world.add_bidirectional_route(&sunken_ruins, &goblin_caves, 10, 5)?;

    world.validate()?;
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(id: &str, category: LocationCategory) -> Location {
        Location::new(id, id, "Test location.", category)
    }

    #[test]
    fn initial_world_has_required_categories_and_nearby_landmarks() {
        let world = build_initial_world().expect("initial world definitions should be valid");

        assert_eq!(world.location_count(), 7);
        assert!(world.validate().is_ok());
        assert_eq!(
            world
                .get_direct_destinations(&LocationId::from("village_start"))
                .len(),
            4
        );

        for (id, expected_category) in [
            ("village_start", LocationCategory::Settlement),
            ("crumbling_watchtower", LocationCategory::Landmark),
            ("standing_stones", LocationCategory::Landmark),
            ("abandoned_mill", LocationCategory::Landmark),
            ("sunken_ruins", LocationCategory::Ruins),
            ("whispering_woods", LocationCategory::Wilderness),
            ("goblin_caves", LocationCategory::Dungeon),
        ] {
            assert_eq!(
                world
                    .get_location(&LocationId::from(id))
                    .map(|item| item.category),
                Some(expected_category)
            );
        }

        let woods_route = world
            .get_direct_routes(&LocationId::from("village_start"))
            .iter()
            .find(|route| route.destination_id == LocationId::from("whispering_woods"))
            .expect("village should connect to the woods");
        assert_eq!(woods_route.travel_cost, 20);
        assert_eq!(woods_route.danger_level, 3);
    }

    #[test]
    fn direct_travel_rejects_non_adjacent_destinations() {
        let world = build_initial_world().expect("initial world definitions should be valid");
        let village = LocationId::from("village_start");
        let woods = LocationId::from("whispering_woods");
        let caves = LocationId::from("goblin_caves");

        assert!(world.can_travel(&village, &woods));
        assert!(!world.can_travel(&village, &caves));
        assert!(world.is_reachable(&village, &caves));
        assert!(!world.is_reachable(&caves, &LocationId::from("unknown")));
    }

    #[test]
    fn reachable_destinations_are_sorted_by_total_cost_then_id() {
        let world = build_initial_world().expect("initial world definitions should be valid");
        let reachable = world.get_reachable_destinations(&LocationId::from("village_start"));

        let expected = [
            ("abandoned_mill", 5),
            ("standing_stones", 6),
            ("crumbling_watchtower", 8),
            ("whispering_woods", 20),
            ("sunken_ruins", 32),
            ("goblin_caves", 35),
        ];
        assert_eq!(reachable.len(), expected.len());
        for (destination, (expected_id, expected_cost)) in reachable.iter().zip(expected) {
            assert_eq!(destination.location_id.as_str(), expected_id);
            assert_eq!(destination.total_travel_cost, expected_cost);
        }
    }

    #[test]
    fn nearest_settlement_uses_total_route_cost_not_hop_count() {
        let mut world = World::new();
        let start = LocationId::from("start");
        let bridge = LocationId::from("bridge");
        let expensive = LocationId::from("settlement_direct");
        let cheap = LocationId::from("settlement_via_bridge");

        world
            .add_location(location("start", LocationCategory::Wilderness))
            .unwrap();
        world
            .add_location(location("bridge", LocationCategory::Landmark))
            .unwrap();
        world
            .add_location(location("settlement_direct", LocationCategory::Settlement))
            .unwrap();
        world
            .add_location(location(
                "settlement_via_bridge",
                LocationCategory::Settlement,
            ))
            .unwrap();

        world.add_route(&start, &expensive, 10, 0).unwrap();
        world.add_route(&start, &bridge, 1, 0).unwrap();
        world.add_route(&bridge, &cheap, 1, 0).unwrap();

        assert_eq!(world.get_nearest_settlement(&start), Some(cheap));
    }

    #[test]
    fn equal_cost_settlement_ties_use_stable_location_id() {
        let mut world = World::new();
        let start = LocationId::from("start");
        let settlement_b = LocationId::from("settlement_b");
        let settlement_a = LocationId::from("settlement_a");

        world
            .add_location(location("start", LocationCategory::Wilderness))
            .unwrap();
        world
            .add_location(location("settlement_b", LocationCategory::Settlement))
            .unwrap();
        world
            .add_location(location("settlement_a", LocationCategory::Settlement))
            .unwrap();
        world.add_route(&start, &settlement_b, 5, 0).unwrap();
        world.add_route(&start, &settlement_a, 5, 0).unwrap();

        assert_eq!(world.get_nearest_settlement(&start), Some(settlement_a));
    }

    #[test]
    fn disconnected_locations_have_no_nearest_settlement() {
        let mut world = build_initial_world().expect("initial world definitions should be valid");
        world
            .add_location(location("isolated_island", LocationCategory::Wilderness))
            .unwrap();

        let island = LocationId::from("isolated_island");
        assert_eq!(world.get_nearest_settlement(&island), None);
        assert!(!world.is_reachable(&LocationId::from("village_start"), &island));
        assert_eq!(
            world.get_nearest_settlement(&LocationId::from("unknown")),
            None
        );
    }

    #[test]
    fn invalid_locations_and_routes_are_rejected() {
        let mut world = World::new();
        let a = LocationId::from("a");
        let b = LocationId::from("b");
        let missing = LocationId::from("missing");

        assert_eq!(
            world.add_location(Location::new(
                " ",
                "Blank ID",
                "",
                LocationCategory::Landmark
            )),
            Err(WorldError::InvalidLocationId)
        );
        world
            .add_location(location("a", LocationCategory::Landmark))
            .unwrap();
        world
            .add_location(location("b", LocationCategory::Settlement))
            .unwrap();

        assert_eq!(
            world.add_location(location("a", LocationCategory::Wilderness)),
            Err(WorldError::DuplicateLocation(a.clone()))
        );
        assert_eq!(
            world.add_location(Location::new(
                "empty_name",
                "  ",
                "",
                LocationCategory::Landmark
            )),
            Err(WorldError::InvalidLocationName(LocationId::from(
                "empty_name"
            )))
        );
        assert_eq!(
            world.add_route(&a, &missing, 1, 0),
            Err(WorldError::UnknownLocation(missing))
        );
        assert_eq!(
            world.add_route(&a, &a, 1, 0),
            Err(WorldError::SelfRoute(a.clone()))
        );
        assert_eq!(
            world.add_route(&a, &b, 0, 0),
            Err(WorldError::ZeroTravelCost)
        );

        world.add_route(&a, &b, 1, 0).unwrap();
        assert_eq!(
            world.add_route(&a, &b, 2, 0),
            Err(WorldError::DuplicateRoute { from: a, to: b })
        );
        assert!(world.validate().is_ok());
    }

    #[test]
    fn bidirectional_route_addition_is_atomic_on_duplicate_reverse_edge() {
        let mut world = World::new();
        let a = LocationId::from("a");
        let b = LocationId::from("b");

        world
            .add_location(location("a", LocationCategory::Landmark))
            .unwrap();
        world
            .add_location(location("b", LocationCategory::Settlement))
            .unwrap();
        world.add_route(&b, &a, 1, 0).unwrap();

        assert_eq!(
            world.add_bidirectional_route(&a, &b, 1, 0),
            Err(WorldError::DuplicateRoute {
                from: b.clone(),
                to: a.clone(),
            })
        );
        assert!(!world.can_travel(&a, &b));
        assert!(world.can_travel(&b, &a));
    }
}
