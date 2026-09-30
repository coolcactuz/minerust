use bevy::math::IVec2;

/// Computes chunk coordinates that are inside `Box(curr, radius)` but were NOT inside `Box(prev, radius)`.
#[must_use]
pub fn get_entered_chunks(prev: IVec2, curr: IVec2, radius: i32) -> Vec<IVec2> {
    let dx = curr.x - prev.x;
    let dz = curr.y - prev.y;
    let mut result = Vec::new();

    // Horizontal bands (movement along Z / Y-axis)
    if dz > 0 {
        let min_z = (curr.y + radius - dz + 1).max(curr.y - radius);
        let max_z = curr.y + radius;
        for z in min_z..=max_z {
            for x in (curr.x - radius)..=(curr.x + radius) {
                result.push(IVec2::new(x, z));
            }
        }
    } else if dz < 0 {
        let min_z = curr.y - radius;
        let max_z = (curr.y - radius - dz - 1).min(curr.y + radius);
        for z in min_z..=max_z {
            for x in (curr.x - radius)..=(curr.x + radius) {
                result.push(IVec2::new(x, z));
            }
        }
    }

    // Vertical bands (movement along X-axis, avoiding coordinates already covered by horizontal bands)
    if dx > 0 {
        let min_x = (curr.x + radius - dx + 1).max(curr.x - radius);
        let max_x = curr.x + radius;
        for x in min_x..=max_x {
            for z in (curr.y - radius)..=(curr.y + radius) {
                let prev_z_diff = (z - prev.y).abs();
                if prev_z_diff <= radius {
                    result.push(IVec2::new(x, z));
                }
            }
        }
    } else if dx < 0 {
        let min_x = curr.x - radius;
        let max_x = (curr.x - radius - dx - 1).min(curr.x + radius);
        for x in min_x..=max_x {
            for z in (curr.y - radius)..=(curr.y + radius) {
                let prev_z_diff = (z - prev.y).abs();
                if prev_z_diff <= radius {
                    result.push(IVec2::new(x, z));
                }
            }
        }
    }

    result
}

/// Computes chunk coordinates that were inside `Box(prev, radius)` but are NOT inside `Box(curr, radius)`.
#[inline]
#[must_use]
pub fn get_exited_chunks(prev: IVec2, curr: IVec2, radius: i32) -> Vec<IVec2> {
    get_entered_chunks(curr, prev, radius)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn brute_force_entered(prev: IVec2, curr: IVec2, radius: i32) -> HashSet<IVec2> {
        let mut set = HashSet::new();
        for x in (curr.x - radius)..=(curr.x + radius) {
            for z in (curr.y - radius)..=(curr.y + radius) {
                let diff_prev = IVec2::new(x, z) - prev;
                if diff_prev.x.abs() > radius || diff_prev.y.abs() > radius {
                    set.insert(IVec2::new(x, z));
                }
            }
        }
        set
    }

    fn brute_force_exited(prev: IVec2, curr: IVec2, radius: i32) -> HashSet<IVec2> {
        let mut set = HashSet::new();
        for x in (prev.x - radius)..=(prev.x + radius) {
            for z in (prev.y - radius)..=(prev.y + radius) {
                let diff_curr = IVec2::new(x, z) - curr;
                if diff_curr.x.abs() > radius || diff_curr.y.abs() > radius {
                    set.insert(IVec2::new(x, z));
                }
            }
        }
        set
    }

    #[test]
    fn test_entered_chunks_stationary() {
        let prev = IVec2::new(5, 5);
        let curr = IVec2::new(5, 5);
        assert!(get_entered_chunks(prev, curr, 10).is_empty());
        assert!(get_exited_chunks(prev, curr, 10).is_empty());
    }

    #[test]
    fn test_entered_and_exited_chunks_single_steps() {
        for radius in [1, 2, 8, 16, 64] {
            for dir in [
                IVec2::new(0, 1),
                IVec2::new(0, -1),
                IVec2::new(1, 0),
                IVec2::new(-1, 0),
                IVec2::new(1, 1),
                IVec2::new(-1, -1),
                IVec2::new(1, -1),
                IVec2::new(-1, 1),
            ] {
                let prev = IVec2::new(10, 20);
                let curr = prev + dir;

                let entered = get_entered_chunks(prev, curr, radius);
                let expected_entered = brute_force_entered(prev, curr, radius);
                assert_eq!(
                    entered.len(),
                    expected_entered.len(),
                    "Entered count mismatch for dir {dir:?} radius {radius}"
                );
                let entered_set: HashSet<IVec2> = entered.into_iter().collect();
                assert_eq!(
                    entered_set, expected_entered,
                    "Entered set mismatch for dir {dir:?} radius {radius}"
                );

                let exited = get_exited_chunks(prev, curr, radius);
                let expected_exited = brute_force_exited(prev, curr, radius);
                assert_eq!(
                    exited.len(),
                    expected_exited.len(),
                    "Exited count mismatch for dir {dir:?} radius {radius}"
                );
                let exited_set: HashSet<IVec2> = exited.into_iter().collect();
                assert_eq!(
                    exited_set, expected_exited,
                    "Exited set mismatch for dir {dir:?} radius {radius}"
                );
            }
        }
    }
}
