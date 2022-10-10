use fj_math::Transform;

use crate::objects::{Objects, Shell};

use super::TransformObject;

impl TransformObject for Shell {
    fn transform(self, transform: &Transform, stores: &Objects) -> Self {
        let faces = self
            .into_faces()
            .into_iter()
            .map(|face| face.transform(transform, stores));
        Self::new().with_faces(faces)
    }
}
