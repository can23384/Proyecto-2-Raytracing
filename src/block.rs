use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Block {
    pub min_corner: Vec3, 
    pub max_corner: Vec3, 
    pub material: Material,
}

impl RayIntersect for Block {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let inv_dir = Vec3::new(
            1.0 / ray_direction.x,
            1.0 / ray_direction.y,
            1.0 / ray_direction.z,
        );

        let t_min = (self.min_corner - ray_origin).component_mul(&inv_dir);
        let t_max = (self.max_corner - ray_origin).component_mul(&inv_dir);

        let t1 = Vec3::new(t_min.x.min(t_max.x), t_min.y.min(t_max.y), t_min.z.min(t_max.z));
        let t2 = Vec3::new(t_min.x.max(t_max.x), t_min.y.max(t_max.y), t_min.z.max(t_max.z));

        let t_near = t1.x.max(t1.y).max(t1.z);
        let t_far = t2.x.min(t2.y).min(t2.z);

        if t_near > 0.0 && t_near < t_far {
            let point = ray_origin + ray_direction * t_near;
            let normal = self.get_normal(&point);
            let distance = t_near;
            return Intersect::new(point, normal, distance, self.material);
        }

        Intersect::empty()
    }
}

impl Block {
    fn get_normal(&self, point: &Vec3) -> Vec3 {
        if (point.x - self.min_corner.x).abs() < 0.001 {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max_corner.x).abs() < 0.001 {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min_corner.y).abs() < 0.001 {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max_corner.y).abs() < 0.001 {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min_corner.z).abs() < 0.001 {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}