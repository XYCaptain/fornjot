use crate::{
    operations::{derive::DeriveFrom, insert::Insert},
    topology::Region,
    Core,
};

use super::{Reverse, ReverseCurveCoordinateSystems};

impl Reverse for Region {
    fn reverse(&self, core: &mut Core) -> Self {
        let exterior = self
            .exterior()
            .reverse(core)
            .insert(core)
            .derive_from(self.exterior(), core);
        let interiors = self.interiors().iter().map(|cycle| {
            cycle.reverse(core).insert(core).derive_from(cycle, core)
        });

        Region::new(exterior, interiors)
    }
}

impl ReverseCurveCoordinateSystems for &Region {
    type Reversed = Region;

    fn reverse_curve_coordinate_systems(
        self,
        core: &mut Core,
    ) -> Self::Reversed {
        let region = self;

        let exterior = region
            .exterior()
            .reverse_curve_coordinate_systems(core)
            .insert(core)
            .derive_from(region.exterior(), core);
        let interiors = region.interiors().iter().map(|cycle| {
            cycle
                .reverse_curve_coordinate_systems(core)
                .insert(core)
                .derive_from(cycle, core)
        });

        Region::new(exterior, interiors)
    }
}
