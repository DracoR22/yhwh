use cgmath::Vector3;
use yhwh_core::math::aabb::Aabb;

pub fn ray_intersects_aabb(origin: Vector3<f32>, dir: Vector3<f32>, aabb: &Aabb<f32>) -> Option<f32> {
    let mut tmin = (aabb.min.x - origin.x) / dir.x;
    let mut tmax = (aabb.max.x - origin.x) / dir.x;

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    let mut tymin = (aabb.min.y - origin.y) / dir.y;
    let mut tymax = (aabb.max.y - origin.y) / dir.y;

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if (tmin > tymax) || (tymin > tmax) {
        return None;
    }

    tmin = tmin.max(tymin);
    tmax = tmax.min(tymax);

    let mut tzmin = (aabb.min.z - origin.z) / dir.z;
    let mut tzmax = (aabb.max.z - origin.z) / dir.z;

    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
    }

    if (tmin > tzmax) || (tzmin > tmax) {
        return None;
    }

    tmin = tmin.max(tzmin);
    tmax = tmax.min(tzmax);

    if tmax < 0.0 {
        return None; 
    }

    Some(tmin.max(0.0))
}