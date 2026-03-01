use crate::query::{QueryData, QueryFilter, QueryState};
use crate::world::World;

impl World {
    pub fn query<D: QueryData>(&mut self) -> QueryState<D, ()> {
        QueryState::new(self)
    }

    pub fn query_with<D: QueryData, F: QueryFilter>(&mut self) -> QueryState<D, F> {
        QueryState::new(self)
    }
}
