/// Normalizes a 2D vector `(dx, dy)`, returning its unit direction and
/// length. Returns `None` if the length is shorter than `epsilon`, in which
/// case no meaningful direction exists (callers decide their own fallback).
pub(crate) fn normalize(dx: f64, dy: f64, epsilon: f64) -> Option<(f64, f64, f64)> {
    let len = (dx * dx + dy * dy).sqrt();
    if len < epsilon {
        None
    } else {
        Some((dx / len, dy / len, len))
    }
}
