use cgmath::{InnerSpace, Matrix4, Vector3};
use yhwh_core::math::aabb::Aabb;

#[derive(Clone, Copy, Debug)]
pub struct FrustumPlane {
    pub normal: Vector3<f32>,
    pub offset: f32
}

#[derive(Debug)]
pub struct Frustum {
    pub planes: [FrustumPlane; 6],
    pub corners: [Vector3<f32>; 8],
    pub bounds_min: Vector3<f32>,
    pub bounds_max: Vector3<f32>
}

impl Frustum {
    pub fn new() -> Self {
        Self { 
            planes: [FrustumPlane { offset: 0.0, normal: Vector3::new(0.0, 1.0, 0.0) }; 6],
            corners: [Vector3::new(0.0, 0.0, 0.0); 8],
            bounds_min: Vector3::new(0.0, 0.0, 0.0),
            bounds_max: Vector3::new(0.0, 0.0, 0.0)
         }
    }

    pub fn update(&mut self, projection_view: &Matrix4<f32>) {
        // Left clipping plane
        self.planes[0].normal.x = projection_view[0][3] + projection_view[0][0];
        self.planes[0].normal.y = projection_view[1][3] + projection_view[1][0];
        self.planes[0].normal.z = projection_view[2][3] + projection_view[2][0];
        self.planes[0].offset = projection_view[3][3] + projection_view[3][0];

        // Right clipping plane
        self.planes[1].normal.x = projection_view[0][3] - projection_view[0][0];
        self.planes[1].normal.y = projection_view[1][3] - projection_view[1][0];
        self.planes[1].normal.z = projection_view[2][3] - projection_view[2][0];
        self.planes[1].offset = projection_view[3][3] - projection_view[3][0];

        // Top clipping plane
        self.planes[2].normal.x = projection_view[0][3] - projection_view[0][1];
        self.planes[2].normal.y = projection_view[1][3] - projection_view[1][1];
        self.planes[2].normal.z = projection_view[2][3] - projection_view[2][1];
        self.planes[2].offset = projection_view[3][3] - projection_view[3][1];

        // Bottom clipping plane
        self.planes[3].normal.x = projection_view[0][3] + projection_view[0][1];
        self.planes[3].normal.y = projection_view[1][3] + projection_view[1][1];
        self.planes[3].normal.z = projection_view[2][3] + projection_view[2][1];
        self.planes[3].offset = projection_view[3][3] + projection_view[3][1];

        // Near clipping plane
        self.planes[4].normal.x = projection_view[0][3] + projection_view[0][2];
        self.planes[4].normal.y = projection_view[1][3] + projection_view[1][2];
        self.planes[4].normal.z = projection_view[2][3] + projection_view[2][2];
        self.planes[4].offset = projection_view[3][3] + projection_view[3][2];

        // Far clipping plane
        self.planes[5].normal.x = projection_view[0][3] - projection_view[0][2];
        self.planes[5].normal.y = projection_view[1][3] - projection_view[1][2];
        self.planes[5].normal.z = projection_view[2][3] - projection_view[2][2];
        self.planes[5].offset = projection_view[3][3] - projection_view[3][2];

        // Normalize planes
        for i in 0..6 {
            let magnitude = self.planes[i].normal.magnitude();
            self.planes[i].normal /= magnitude;
            self.planes[i].offset /= magnitude;
        }

        self.corners[0] = Frustum::intersect_planes(&self.planes[0].normal, self.planes[0].offset, &self.planes[2].normal, self.planes[2].offset, &self.planes[4].normal, self.planes[4].offset);
        self.corners[1] = Frustum::intersect_planes(&self.planes[1].normal, self.planes[1].offset, &self.planes[2].normal, self.planes[2].offset, &self.planes[4].normal, self.planes[4].offset);
        self.corners[2] = Frustum::intersect_planes(&self.planes[0].normal, self.planes[0].offset, &self.planes[3].normal, self.planes[3].offset, &self.planes[4].normal, self.planes[4].offset);
        self.corners[3] = Frustum::intersect_planes(&self.planes[1].normal, self.planes[1].offset, &self.planes[3].normal, self.planes[3].offset, &self.planes[4].normal, self.planes[4].offset);
        self.corners[4] = Frustum::intersect_planes(&self.planes[0].normal, self.planes[0].offset, &self.planes[2].normal, self.planes[2].offset, &self.planes[5].normal, self.planes[5].offset);
        self.corners[5] = Frustum::intersect_planes(&self.planes[1].normal, self.planes[1].offset, &self.planes[2].normal, self.planes[2].offset, &self.planes[5].normal, self.planes[5].offset);
        self.corners[6] = Frustum::intersect_planes(&self.planes[0].normal, self.planes[0].offset, &self.planes[3].normal, self.planes[3].offset, &self.planes[5].normal, self.planes[5].offset);
        self.corners[7] = Frustum::intersect_planes(&self.planes[1].normal, self.planes[1].offset, &self.planes[3].normal, self.planes[3].offset, &self.planes[5].normal, self.planes[5].offset);

        self.bounds_min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds_max = Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX);

        for i in 0..8 {
            self.bounds_min = self.bounds_min.zip(self.corners[i], f32::min);
            self.bounds_max = self.bounds_max.zip(self.corners[i], f32::max);
        }

    }

    pub fn intersects_aabb(&self, aabb: &Aabb<f32>) -> bool {
        let aabb_corners: [Vector3<f32>; 8] = [
            Vector3::new(aabb.min.x, aabb.min.y, aabb.min.z),
            Vector3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vector3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vector3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            Vector3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vector3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vector3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vector3::new(aabb.max.x, aabb.max.y, aabb.max.z),
        ];

        for i in 0..6 {
            let mut points_outside = 0;
            for j in 0..8 {
                if Frustum::signed_distance(&aabb_corners[j], &self.planes[i]) < 0.0 {
                    points_outside += 1;
                }
            }
            if points_outside == 8 {
                return false
            }
        }

        return true
    }

    pub fn intersects_aabb_fast(&self, aabb: &Aabb<f32>) -> bool {
        for i in 0..6 {
            let min_corner: Vector3<f32> = Vector3::new(
                if self.planes[i].normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if self.planes[i].normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if self.planes[i].normal.z >= 0.0 { aabb.max.z } else { aabb.min.z }
            );

            if Frustum::signed_distance(&min_corner, &self.planes[i]) < 0.0 {
                return false
            }
        }

        return true
    }

    fn intersect_planes(n1: &Vector3<f32>, d1: f32, n2: &Vector3<f32>, d2: f32, n3: &Vector3<f32>, d3: f32) -> Vector3<f32> {
        let cross_n2n3 = n2.cross(*n3);
        let denom = n1.dot(cross_n2n3);
        if denom.abs() < 1e-6 {
            return Vector3::new(0.0, 0.0, 0.0)
        }

        -(d1 * cross_n2n3 + d2 * n3.cross(*n1) + d3 * n1.cross(*n2)) / denom
    }

    fn signed_distance(point: &Vector3<f32>, plane: &FrustumPlane) -> f32 {
        return plane.normal.dot(*point) + plane.offset
    }
}