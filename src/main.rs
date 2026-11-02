use std::{f32::INFINITY};
use glam::{Vec2, Vec3};
use minifb::{Key::self, Window, WindowOptions};

const WIDTH: usize = 1600;
const HEIGHT: usize = 900;

#[derive(Default)]
struct World {
    floor: Floor,
    sky: Sky
}


#[derive(Default)]
struct Floor {
    height: f32,
    colour: [u8;4]
}

#[derive(Default)]
struct Sky {
    colour: [u8;4],
    tint_per_unit: f32
}


#[derive(Debug)]
struct Camera {
    position: Vec3,
    velocity: Vec3,
    orientation: Vec3,

    fov: f32,
    resolution: Vec2
}


#[derive(Debug, Default)]
struct Ray {
    position: Vec3,
    orientation: Vec3,
    colour: [u8;4],
    reflectiveness: f32,
    last_collision: Vec3,
    normal: Vec3
}


#[derive(Default)]
struct Sphere {
    position: Vec3,
    radius: f32,
    colour: [u8;4],
    reflectiveness: f32,
    velocity: Vec3
}

#[derive(Default)]
struct Light {
    position: Vec3,
    brightness: f32,
    //radius: f32,
    colour: [u8;4],
    velocity: Vec3
}

struct Triangle {
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    colour: [u8;4]
} 


#[derive(Default)]
struct Mesh<'a> {
    triangles: &'a [Triangle],
    default_colour: [u8;4],
    
    position: Vec3,
    orientation: Vec3,
    scale: f32
}


fn mix_colours(c1: [u8;4], c2: [u8;4], weight: f32) -> [u8;4] {
    let mut average = [8;4];
    for i in 1..4 {
        average[i] = (c1[i] as f32 * (1.-weight).clamp(0.,1.)   +   c2[i] as f32 * weight) as u8; 
    }
    return average
}

fn vec2int(slice: [u8;4]) -> u32 {
    return u32::from_be_bytes(slice);
}

fn find_focal_length(camera: &Camera) -> f32{
    let max_length = (camera.resolution.x).max(camera.resolution.y);
    let numerator = (max_length-1.)/2.;
    let denominator = ((camera.fov.to_radians())/2.).tan();
    let result = numerator/denominator;

    return result

}


enum VecAxis {X, Y, Z}

fn scale_vector_to_value(pos_vec: &Vec3, dir_vec: &Vec3, axis: VecAxis, value: f32) -> Vec3 {
    let mult = match axis {
        VecAxis::X => (value-pos_vec.x)/dir_vec.x,
        VecAxis::Y => (value-pos_vec.y)/dir_vec.y,
        VecAxis::Z => (value-pos_vec.z)/dir_vec.z,
    };
    
    let result = pos_vec + dir_vec * mult;
    
    return result
}



fn sphere_collision(ray: &Ray, sphere: &Sphere) -> Option<Vec3> {
    /* 
    Source: https://gamedev.stackexchange.com/questions/20815/most-efficient-bounding-sphere-vs-ray-collision-algorithms/20831#20831
    Original pseudocode by https://gamedev.stackexchange.com/users/3505
    Ported to rust by Casinii
    */

    let position = sphere.position;
    let radius = sphere.radius;
    let vec_start = ray.position;
    let vec_dir = ray.orientation;

    let sphere_to_start = vec_start - position;
    let radius_squared = radius.powi(2);

    if sphere_to_start.dot(sphere_to_start) <= radius_squared {
        return Some(vec_start)
    }
    else if sphere_to_start.dot(vec_dir) <= 0. {
        let v = sphere_to_start - sphere_to_start.project_onto(vec_dir);
        let v_squared = v.dot(v);
        if v_squared <= radius_squared {
            return Some(position + v - (vec_dir.normalize() * (radius_squared - v_squared).sqrt()))
        }
        else {
            return None
        }
    }
    else {
        return None
    }
}

fn physics_step(objects: &mut [Sphere], camera: &mut Camera, frame: f32) {
    for object in objects {
        if object.radius != 1. {
            return
        }
        object.position.x += (frame/30.).sin()/20.;
        object.position += (frame/120.).sin()/100.;
    }
    //camera.position += camera.velocity/60.;
}

fn light_calc(ray: &mut Ray, spheres: &[Sphere], lights: &[Light]) -> f32{
    let mut light_scale = 0.;
    for light in lights {
        let mut collision = false;

        ray.orientation = (light.position - ray.position).normalize();

        for sphere in spheres {
            if sphere_collision(ray, sphere).is_some() {
                collision = true;
                
            }
        }
    
        if !collision {
            light_scale += light.brightness * ray.normal.normalize().dot((light.position - ray.position).normalize()) / ray.position.distance_squared(light.position);
        }
    }

    return light_scale
}

fn render(camera: &Camera, buffer: &mut Vec<u32>, objects: &[Sphere], lights: &[Light], world: &World, depth: u32) {
    let focal_length = find_focal_length(&camera);
    let _normalized_orientation = camera.orientation.normalize_or(Vec3::new(1.,0.,0.));

    for (idx, pixel) in buffer.iter_mut().enumerate() {
        let pixel_y = idx as u32 / camera.resolution.x as u32;
        let pixel_x = idx as u32 % camera.resolution.x as u32;

        let ray_vector = Vec3::new(focal_length,
            pixel_x as f32 + 0.5 - ((camera.resolution.x as u32)/2) as f32,
            (pixel_y as f32 + 0.5 - ((camera.resolution.y as u32)/2) as f32) * -1.
        ).normalize();

        let mut ray = Ray{
            position: camera.position,
            orientation: ray_vector,
            colour: [0,0,150,255],
            last_collision: Vec3::new(INFINITY, INFINITY, INFINITY),
            ..Default::default()
        };
        
        raycast(&mut ray, &objects, &lights, &world, depth);

        *pixel = vec2int(ray.colour);
    }
}

fn raycast(ray: &mut Ray, spheres: &[Sphere], lights: &[Light], world: &World, depth: u32) {
    let mut is_world = true;
    for sphere in spheres {
        if let Some(collision_point) = sphere_collision(ray, &sphere) {
            is_world = false;
            if ray.position.distance_squared(collision_point) < ray.position.distance_squared(ray.last_collision) {
                ray.last_collision = collision_point;
                ray.position = ray.last_collision;
                ray.normal = (ray.position - sphere.position).normalize(); 
                ray.colour = sphere.colour;
                ray.reflectiveness = sphere.reflectiveness;
            }
        }
    }

    if !is_world {
        ray.position = ray.last_collision + 0.00001 * ray.normal;
        if depth <= 0 {
            ray.colour = mix_colours([0;4], ray.colour, light_calc(ray, &spheres, &lights));
        }
        else {
            
            
            if ray.reflectiveness == 0. {
                ray.colour = mix_colours([0;4], ray.colour, light_calc(ray, &spheres, &lights));
            }
            else {     
                let new_dir = ray.orientation.reflect(ray.normal);
                
                ray.orientation = new_dir;
                ray.last_collision = Vec3::new(INFINITY, INFINITY, INFINITY);
                let base_reflectivity = ray.reflectiveness;
                let base_colour = ray.colour;
                raycast(&mut *ray, &spheres, &lights, &world, depth-1);
                
                if base_reflectivity != 1. {
                    
                    ray.colour = mix_colours(base_colour, ray.colour, base_reflectivity);
                }
            }
            
        }
    }
    else {
        if ray.orientation.z < 0. {
            ray.colour = world.floor.colour;
            ray.position = scale_vector_to_value(&ray.position, &ray.orientation, VecAxis::Z, world.floor.height);
            ray.normal = Vec3::new(0.,0.,1.);
            ray.colour = [(light_calc(ray, &spheres, &lights)*255.) as u8; 4];
        }
        else if ray.orientation.z >= 0. {
            ray.colour = world.sky.colour;
            ray.colour = [0, ((ray.orientation.x * 500.)%255.) as u8, ((ray.orientation.y * 500.)%255.) as u8, ((ray.orientation.z * 500.)%255.) as u8];
            return
        }
    }

    return
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "mwehehehe",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);



    let mut camera = Camera{
        position: Vec3::new(0.,0.,1.),
        velocity: Vec3::new(-1.,0.,0.),
        orientation: Vec3::new(1.,0.,0.),

        fov: 120.,
        resolution: Vec2::new(WIDTH as f32,HEIGHT as f32)
    };



    let mut objects = [
        Sphere {
        position: Vec3::new(3., -2., 1.),
        radius: 1.,
        colour: [0,0,150,255],
        reflectiveness: 0.,
        velocity: Vec3::new(0.,0.,0.),
        ..Default::default()
        },
        Sphere {
        position: Vec3::new(4., 4., 2.),
        radius: 2.,
        colour: [0,255,0,0],
        reflectiveness: 1.,
        velocity: Vec3::new(0.,0.,0.),
        ..Default::default()
        }
    ];

    let lights = [
        Light {
            position: Vec3::new(1.,0.,10.),
            brightness: 100.,
            ..Default::default()
        }
    ];

    let world = World{
        floor: Floor{
            height: 0.,
            colour: [0,150,150,150]
        },
        sky: Sky{
            colour: [0,0,50,255],
            tint_per_unit: 0.
        }
    };


    let mut counter = 0.;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        counter += 0.2;
        physics_step(&mut objects, &mut camera, counter);
        println!("{}", window.get_fps());
        render(&camera, &mut buffer, &objects, &lights, &world, 1);
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }


}