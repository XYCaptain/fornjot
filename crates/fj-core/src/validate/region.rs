use crate::{geometry::Geometry, objects::Region};

use super::{Validate, ValidationConfig, ValidationError};

impl Validate for Region {
    fn validate(
        &self,
        _: &ValidationConfig,
        _: &mut Vec<ValidationError>,
        _: &Geometry,
    ) {
    }
}
