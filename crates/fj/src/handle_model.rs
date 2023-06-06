use std::ops::Deref;

use fj_core::algorithms::{approx::Tolerance, triangulate::Triangulate};
use fj_interop::model::Model;
use fj_math::Aabb;

use crate::Args;

/// Export or display a model, according to CLI arguments
///
/// This function is intended to be called by applications that define a model
/// and want to provide a standardized CLI interface for dealing with that
/// model.
///
/// This function is used by Fornjot's own testing infrastructure, but is useful
/// beyond that, when using Fornjot directly to define a model.
pub fn handle_model<M>(
    model: impl Deref<Target = M>,
    tolerance: impl Into<Tolerance>,
) -> Result
where
    for<'r> (&'r M, Tolerance): Triangulate,
{
    let mesh = (model.deref(), tolerance.into()).triangulate();

    let args = Args::parse();
    if let Some(path) = args.export {
        crate::export::export(&mesh, &path)?;
        return Ok(());
    }

    let aabb = Aabb::<3>::from_points(mesh.vertices());
    let model = Model { mesh, aabb };

    crate::window::display(model, false)?;

    Ok(())
}

/// Return value of [`handle_model`]
pub type Result = std::result::Result<(), Error>;

/// Error returned by [`handle_model`]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error displaying model
    #[error("Error displaying model")]
    Display(#[from] crate::window::Error),

    /// Error exporting model
    #[error("Error exporting model")]
    Export(#[from] crate::export::Error),
}