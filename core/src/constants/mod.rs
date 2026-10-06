pub const DEFAULT_BEZIER_CURVE_FACTOR: f64 = 0.3;

/// Maximum number of phases (`action` entries) a playbook may contain.
pub const MAX_PHASES: usize = 3;

/// Default distance a defender marking a player is offset towards the
/// center of the court, when no explicit distance is given.
pub const DEFAULT_DEFENSE_OFFSET: f64 = 20.0;

/// Default maximum input size in bytes (100KB) accepted by the lexer, when
/// the `MAX_INPUT_SIZE` env var is not set.
pub const DEFAULT_MAX_INPUT_SIZE: usize = 100 * 1024;
