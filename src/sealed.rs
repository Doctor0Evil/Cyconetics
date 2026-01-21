pub trait NeurorightsMarkerSealed {}

/// Public trait exposed to the rest of Cyber-Retrieval.
/// Only types inside `neurorights-core` can implement it.
pub trait NeurorightsMarker: NeurorightsMarkerSealed {}

impl<T: NeurorightsMarkerSealed> NeurorightsMarker for T {}
