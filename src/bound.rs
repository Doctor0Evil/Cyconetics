use crate::NeurorightsMarker;

/// Generic gate that binds a payload `P` to a neurorights envelope `N`.
#[derive(Clone, Debug)]
pub struct NeurorightsBound<P, N: NeurorightsMarker> {
    payload: P,
    envelope: N,
}

impl<P, N: NeurorightsMarker> NeurorightsBound<P, N> {
    pub fn new(payload: P, envelope: N) -> Self {
        Self { payload, envelope }
    }

    pub fn payload(&self) -> &P {
        &self.payload
    }

    pub fn into_inner(self) -> P {
        self.payload
    }

    pub fn neurorights_envelope(&self) -> &N {
        &self.envelope
    }
}
