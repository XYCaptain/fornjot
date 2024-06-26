use std::ops::Deref;

use crate::{
    operations::{derive::DeriveFrom, insert::Insert},
    storage::Handle,
    topology::{Region, Surface},
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

impl ReverseCurveCoordinateSystems for (&Region, &Handle<Surface>) {
    type Reversed = Region;

    fn reverse_curve_coordinate_systems(
        self,
        core: &mut Core,
    ) -> Self::Reversed {
        let (region, surface) = self;

        let exterior = (region.exterior().deref(), surface)
            .reverse_curve_coordinate_systems(core)
            .insert(core)
            .derive_from(region.exterior(), core);
        let interiors = region.interiors().iter().map(|cycle| {
            (cycle.deref(), surface)
                .reverse_curve_coordinate_systems(core)
                .insert(core)
                .derive_from(cycle, core)
        });

        Region::new(exterior, interiors)
    }
}
