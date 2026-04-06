use std::{cmp::Ordering, ops::Mul};
use cgmath::{BaseFloat, Bounded, Matrix3, Matrix4, Vector3, Vector4};
use cgmath::num_traits::cast;

/// Returns the min value of two PartialOrd values.
pub fn min<S: PartialOrd>(v1: S, v2: S) -> S {
    match v1.partial_cmp(&v2) {
        Some(Ordering::Less) => v1,
        _ => v2,
    }
}

/// Returns the max value of two PartialOrd values.
pub fn max<S: PartialOrd>(v1: S, v2: S) -> S {
    match v1.partial_cmp(&v2) {
        Some(Ordering::Greater) => v1,
        _ => v2,
    }
}

/// Return the partial minimum from an Iterator of PartialOrd if it exists.
pub fn partial_min<I, S>(iter: I) -> Option<S>
where
    S: PartialOrd,
    I: Iterator<Item = S>,
{
    iter.min_by(|v1, v2| v1.partial_cmp(v2).unwrap_or(Ordering::Equal))
}

/// Return the partial maximum from an Iterator of PartialOrd if it exists.
pub fn partial_max<I, S>(iter: I) -> Option<S>
where
    S: PartialOrd,
    I: Iterator<Item = S>,
{
    iter.max_by(|v1, v2| v1.partial_cmp(v2).unwrap_or(Ordering::Equal))
}

#[derive(Copy, Clone, Debug)]
pub struct Aabb<S> {
    pub min: Vector3<S>,
    pub max: Vector3<S>,
}

impl<S> Aabb<S> {
    /// Create a new AABB.
    pub fn new(min: Vector3<S>, max: Vector3<S>) -> Self {
        Aabb { min, max }
    }
}

impl<S: BaseFloat> Aabb<S> {
    /// Compute the union of several AABBs.
    pub fn union(aabbs: &[Aabb<S>]) -> Option<Self> {
        if aabbs.is_empty() {
            None
        } else if aabbs.len() == 1 {
            Some(aabbs[0])
        } else {
            let min_x = partial_min(aabbs.iter().map(|aabb| aabb.min.x)).unwrap();
            let min_y = partial_min(aabbs.iter().map(|aabb| aabb.min.y)).unwrap();
            let min_z = partial_min(aabbs.iter().map(|aabb| aabb.min.z)).unwrap();
            let min = Vector3::new(min_x, min_y, min_z);

            let max_x = partial_max(aabbs.iter().map(|aabb| aabb.max.x)).unwrap();
            let max_y = partial_max(aabbs.iter().map(|aabb| aabb.max.y)).unwrap();
            let max_z = partial_max(aabbs.iter().map(|aabb| aabb.max.z)).unwrap();
            let max = Vector3::new(max_x, max_y, max_z);

            Some(Aabb::new(min, max))
        }
    }

    /// Get the size of the larger side of the AABB.
    pub fn get_larger_side_size(&self) -> S {
        let size = self.max - self.min;
        let x = size.x.abs();
        let y = size.y.abs();
        let z = size.z.abs();

        if x > y && x > z {
            x
        } else if y > z {
            y
        } else {
            z
        }
    }

    /// Get the center of the AABB.
    pub fn get_center(&self) -> Vector3<S> {
        let two = S::one() + S::one();
        self.min + (self.max - self.min) / two
    }

    /// Transform AABB to world space.
     pub fn transform(&self, matrix: Matrix4<f32>) -> Aabb<f32> {
        let min = self.min.map(|v| cast(v).unwrap());
        let max = self.max.map(|v| cast(v).unwrap());

        let corners: [Vector3<f32>; 8] = [
            Vector3::new(min.x, min.y, min.z),
            Vector3::new(max.x, min.y, min.z),
            Vector3::new(min.x, max.y, min.z),
            Vector3::new(max.x, max.y, min.z),
            Vector3::new(min.x, min.y, max.z),
            Vector3::new(max.x, min.y, max.z),
            Vector3::new(min.x, max.y, max.z),
            Vector3::new(max.x, max.y, max.z),
        ];

        let mut new_min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut new_max = Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX);

        for i in 0..8 {
            let corner_4d = cgmath::Vector4::new(corners[i].x, corners[i].y, corners[i].z, 1.0);
            let transformed_4d = matrix * corner_4d;

            let transformed = cgmath::Vector3::new(transformed_4d.x, transformed_4d.y, transformed_4d.z);
            new_min = new_min.zip(transformed, |a, b| a.min(b));
            new_max = new_max.zip(transformed, |a, b| a.max(b));
        }

        Aabb { 
            min: new_min,
            max: new_max
         }
    }
}

/// Transform the AABB by multiplying it with a Matrix4.
impl<S: BaseFloat> Mul<Matrix4<S>> for Aabb<S> {
    type Output = Aabb<S>;

    fn mul(self, rhs: Matrix4<S>) -> Self::Output {
        let min = self.min;
        let min = rhs * Vector4::new(min.x, min.y, min.z, S::one());

        let max = self.max;
        let max = rhs * Vector4::new(max.x, max.y, max.z, S::one());

        Aabb::new(min.truncate(), max.truncate())
    }
}

/// Scale the AABB by multiplying it by a BaseFloat
impl<S: BaseFloat> Mul<S> for Aabb<S> {
    type Output = Aabb<S>;

    fn mul(self, rhs: S) -> Self::Output {
        Aabb::new(self.min * rhs, self.max * rhs)
    }
}