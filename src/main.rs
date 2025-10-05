mod framebuffer;
mod ray_intersect;
mod block;
mod color;
mod camera;
mod light;
mod material;

use minifb::{ Window, WindowOptions, Key };
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::block::Block;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Block],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Block],
    light: &Light,
    depth: u32,
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }


    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }

    (diffuse + specular) * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Block], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Raytracing",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

        let rubber = Material::new(
        Color::new(80, 0, 0),
        1.0,
        [0.9, 0.1, 0.0, 0.0],
        0.0,
    );

    let earth = Material::new(
        Color::new(139, 69, 19),
        0.2,
        [0.9, 0.1, 0.0, 0.0],
        1.0,
    );

    let leaves = Material::new(
        Color::new(34, 139, 34),
        0.3,
        [0.8, 0.2, 0.1, 0.0],
        1.05,
    );

    let wood = Material::new(
        Color::new(101, 67, 33),
        0.1,
        [0.9, 0.1, 0.0, 0.0],
        1.0,
    );

    let awa = Material::new(
        Color::new(64, 164, 223),
        0.9,
        [0.1, 0.8, 0.1, 0.6],
        1.33,
    );


   let objects = [

    Block { min_corner: Vec3::new(-7.0, -1.0, -7.0), max_corner: Vec3::new( 7.0, -0.5, -2.0), material: earth },
    Block { min_corner: Vec3::new(-7.0, -1.0,  2.0), max_corner: Vec3::new( 7.0, -0.5,  7.0), material: earth },

    Block { min_corner: Vec3::new(-7.0, -1.0, -2.0), max_corner: Vec3::new( 7.0, -0.5, -1.0), material: awa },
    Block { min_corner: Vec3::new(-7.0, -1.0, -1.0), max_corner: Vec3::new( 7.0, -0.5,  0.0), material: awa },
    Block { min_corner: Vec3::new(-7.0, -1.0,  0.0), max_corner: Vec3::new( 7.0, -0.5,  1.0), material: awa },
    Block { min_corner: Vec3::new(-7.0, -1.0,  1.0), max_corner: Vec3::new( 7.0, -0.5,  2.0), material: awa },

    Block { min_corner: Vec3::new(-6.0, -0.5, -3.0), max_corner: Vec3::new(-5.5, 0.5, -2.5), material: wood },
    Block { min_corner: Vec3::new(-6.5, 0.5, -3.5), max_corner: Vec3::new(-5.0, 1.5, -2.0), material: leaves },

    Block { min_corner: Vec3::new( 4.0, -0.5, -5.0), max_corner: Vec3::new( 4.5, 0.5, -4.5), material: wood },
    Block { min_corner: Vec3::new( 3.5, 0.5, -5.5), max_corner: Vec3::new( 5.0, 1.5, -4.0), material: leaves },

    Block { min_corner: Vec3::new( 2.0, -0.5,  5.0), max_corner: Vec3::new( 2.5, 0.5,  5.5), material: wood },
    Block { min_corner: Vec3::new( 1.5, 0.5,  4.5), max_corner: Vec3::new( 3.0, 1.5,  6.0), material: leaves },

    Block { min_corner: Vec3::new(-2.0, -0.5,  4.0), max_corner: Vec3::new(-1.5, 0.5,  4.5), material: wood },
    Block { min_corner: Vec3::new(-2.5, 0.5,  3.5), max_corner: Vec3::new(-1.0, 1.5,  5.0), material: leaves },

    Block { min_corner: Vec3::new( 5.6, -0.5,  0.0), max_corner: Vec3::new( 6.1, 0.5,  0.5), material: wood },
    Block { min_corner: Vec3::new( 5.1, 0.5, -0.5), max_corner: Vec3::new( 6.6, 1.5,  1.0), material: leaves },

    Block { min_corner: Vec3::new(-4.0, -0.5,  6.0), max_corner: Vec3::new(-3.5, 0.5,  6.5), material: wood },
    Block { min_corner: Vec3::new(-4.5, 0.5,  5.5), max_corner: Vec3::new(-3.0, 1.5,  7.0), material: leaves },

    Block { min_corner: Vec3::new( 0.0, -0.5, -6.0), max_corner: Vec3::new( 0.5, 0.5, -5.5), material: wood },
    Block { min_corner: Vec3::new(-0.5, 0.5, -6.5), max_corner: Vec3::new( 1.0, 1.5, -5.0), material: leaves },


    Block { min_corner: Vec3::new(-5.5, -0.5,  4.5), max_corner: Vec3::new( 5.5,  0.5,  7.0), material: earth },
    Block { min_corner: Vec3::new(-4.0,  0.5,  5.0), max_corner: Vec3::new( 4.0,  1.5,  6.8), material: earth },
    Block { min_corner: Vec3::new(-2.5,  1.5,  5.5), max_corner: Vec3::new( 2.5,  2.5,  6.5), material: earth },
    Block { min_corner: Vec3::new(-1.8,  2.5,  5.8), max_corner: Vec3::new(-1.0,  3.5,  6.3), material: earth },
    Block { min_corner: Vec3::new( 0.2,  2.5,  5.9), max_corner: Vec3::new( 0.9,  3.5,  6.4), material: earth },
    Block { min_corner: Vec3::new( 1.4,  2.5,  5.7), max_corner: Vec3::new( 2.2,  3.2,  6.2), material: earth },


    Block { min_corner: Vec3::new(-6.5, -0.5,  1.0), max_corner: Vec3::new(-5.0,  0.5,  2.5), material: earth },
    Block { min_corner: Vec3::new(-6.2,  0.5,  1.3), max_corner: Vec3::new(-5.3,  1.5,  2.2), material: earth },
    Block { min_corner: Vec3::new(-6.0,  1.5,  1.6), max_corner: Vec3::new(-5.5,  2.2,  2.0), material: earth },

    Block { min_corner: Vec3::new( 3.0, -0.5, -4.5), max_corner: Vec3::new( 4.5,  0.5, -3.2), material: earth },
    Block { min_corner: Vec3::new( 3.3,  0.5, -4.2), max_corner: Vec3::new( 4.2,  1.3, -3.5), material: earth },
    Block { min_corner: Vec3::new( 5.0, -0.5, -3.0), max_corner: Vec3::new( 6.2,  0.5, -2.0), material: earth },
    Block { min_corner: Vec3::new( 5.3,  0.5, -2.7), max_corner: Vec3::new( 5.9,  1.2, -2.2), material: earth },

    Block { min_corner: Vec3::new(-4.5, -0.5, -6.5), max_corner: Vec3::new(-1.0,  0.5, -4.0), material: earth },
    Block { min_corner: Vec3::new(-4.0,  0.5, -6.0), max_corner: Vec3::new(-1.5,  1.5, -4.3), material: earth },
    Block { min_corner: Vec3::new(-3.6,  1.5, -5.6), max_corner: Vec3::new(-2.8,  2.4, -4.8), material: earth },
    Block { min_corner: Vec3::new(-2.3,  1.5, -5.4), max_corner: Vec3::new(-1.7,  2.6, -4.6), material: earth },

    Block { min_corner: Vec3::new(-3.2, -0.5, -1.5), max_corner: Vec3::new(-1.8,  0.5,  1.5), material: earth },
    Block { min_corner: Vec3::new(-2.9,  0.5, -1.0), max_corner: Vec3::new(-2.1,  1.3,  1.0), material: earth },
    Block { min_corner: Vec3::new(-2.6,  1.3, -0.5), max_corner: Vec3::new(-2.4,  2.0,  0.5), material: earth },

    Block { min_corner: Vec3::new(-2.2, 0.6, 4.3), max_corner: Vec3::new(-1.7, 1.1, 4.8), material: rubber },
    Block { min_corner: Vec3::new(-1.6, 1.0, 4.4), max_corner: Vec3::new(-1.1, 1.5, 4.9), material: rubber },
    Block { min_corner: Vec3::new(-1.1, 0.7, 4.2), max_corner: Vec3::new(-0.6, 1.2, 4.7), material: rubber },

    Block { min_corner: Vec3::new(-0.9, 0.6, -5.6), max_corner: Vec3::new(-0.4, 1.1, -5.1), material: rubber },
    Block { min_corner: Vec3::new(-1.2, 1.0, -5.3), max_corner: Vec3::new(-0.7, 1.5, -4.8), material: rubber },
    Block { min_corner: Vec3::new(-1.5, 0.7, -5.1), max_corner: Vec3::new(-1.0, 1.2, -4.6), material: rubber },

    Block { min_corner: Vec3::new(4.7, 0.6, -3.9), max_corner: Vec3::new(5.2, 1.1, -3.4), material: rubber },
    Block { min_corner: Vec3::new(5.1, 0.9, -3.6), max_corner: Vec3::new(5.6, 1.4, -3.1), material: rubber },

];






    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let light = Light::new(
        Vec3::new(1.0, 2.0, 5.0),
        Color::new(255, 255, 255),
        1.0
    );

        let rotation_speed = PI/10.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {

        // Rotación con flechas
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0); 
        }

        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }

        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        // Zoom con W y S
        if window.is_key_down(Key::W) {
            camera.zoom(0.1); // acercar
        }

        if window.is_key_down(Key::S) {
            camera.zoom(-0.1); // alejar
        }

        render(&mut framebuffer, &objects, &camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }

}   