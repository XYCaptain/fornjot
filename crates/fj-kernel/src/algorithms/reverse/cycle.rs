use crate::{
    insert::Insert,
    objects::{Cycle, Objects},
    storage::Handle,
    validate::ValidationError,
};

use super::Reverse;

impl Reverse for Handle<Cycle> {
    fn reverse(self, objects: &mut Objects) -> Result<Self, ValidationError> {
        let mut edges = self
            .half_edges()
            .cloned()
            .map(|edge| edge.reverse(objects))
            .collect::<Result<Vec<_>, _>>()?;

        edges.reverse();

        Ok(Cycle::new(edges).insert(objects)?)
    }
}
