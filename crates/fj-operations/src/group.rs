use fj_interop::debug::DebugInfo;
use fj_kernel::{
    algorithms::validate::{
        Validate, Validated, ValidationConfig, ValidationError,
    },
    objects::{Faces, Objects},
};
use fj_math::Aabb;

use crate::planes::Planes;

use super::Shape;

impl Shape for fj::Group {
    type Brep = Faces;

    fn compute_brep(
        &self,
        config: &ValidationConfig,
        stores: &Objects,
        planes: &Planes,
        debug_info: &mut DebugInfo,
    ) -> Result<Validated<Self::Brep>, ValidationError> {
        let mut faces = Faces::new();

        let a = self.a.compute_brep(config, stores, planes, debug_info)?;
        let b = self.b.compute_brep(config, stores, planes, debug_info)?;

        faces.extend(a.into_inner());
        faces.extend(b.into_inner());

        faces.validate_with_config(config)
    }

    fn bounding_volume(&self) -> Aabb<3> {
        let a = self.a.bounding_volume();
        let b = self.b.bounding_volume();

        a.merged(&b)
    }
}
