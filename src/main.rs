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
    last_collision: Vec3,
    normal: Vec3
}


#[derive(Default)]
struct Sphere {
    position: Vec3,
    radius: f32,
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
    points: &'a [Triangle],
    default_colour: [u8;4],
    
    position: Vec3,
    orientation: Vec3,
    scale: f32
}


fn average_colour(c1: [u8;4], c2: [u8;4]) -> [u8;4] {
    let mut average = [8;4];
    for i in 1..4 {
        average[i] = (c1[i] + c2[i])/2; 
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

fn render(camera: &Camera, buffer: &mut Vec<u32>, objects: &[Sphere], world: &World) {
    let focal_length = find_focal_length(&camera);
    let _normalized_orientation = camera.orientation.normalize_or(Vec3::new(1.,0.,0.));

    for (idx, pixel) in buffer.iter_mut().enumerate() {
        let pixel_y = idx as u32 / camera.resolution.x as u32;
        let pixel_x = idx as u32 % camera.resolution.x as u32;

        let ray_vector = Vec3::new(focal_length,
            pixel_x as f32 + 0.5 - ((camera.resolution.x as u32)/2) as f32,
            (pixel_y as f32 + 0.5 - ((camera.resolution.y as u32)/2) as f32) * -1.
        ).normalize();

        //println!("{:?}, {}", (pixel_x, pixel_y), pixel_y as f32 + 0.5 - ((camera.resolution.y as u32)/2) as f32);


        let mut ray = Ray{
            position: camera.position,
            orientation: ray_vector,
            colour: [0,0,150,255],
            last_collision: Vec3::new(INFINITY, INFINITY, INFINITY),
            ..Default::default()
        };
        
        raycast_new(&mut ray, &objects, &world, 2);

        *pixel = vec2int(ray.colour);
    }
}

fn raycast(ray: &mut Ray) {
    for _ in 0..100 {
        if ray.position.distance(Vec3::new(6., 3., 1.5)) < 3. {
            let dot = (-300. + 500. * ray.orientation.dot(Vec3::new(1.,0.,0.))) as u8;
            ray.colour = [0,dot,dot,dot];
            return
        }
        else if ray.position.z <= 0. {
            let distance = (5. * ray.position.distance(Vec3::ZERO).clamp(0.,255.)) as u8;
            ray.colour = [0, distance, distance, distance];
            return
        }
        else {
            ray.position += ray.orientation * 0.1;
            //if ray.orientation.z < 0. {
            //    println!("{ray:?}");
            //}
        }
    }
}

fn raycast_new(ray: &mut Ray, spheres: &[Sphere], world: &World, depth: u32) {
    let mut not_world = false;
    for sphere in spheres {
        if let Some(collision_point) = sphere_collision(ray, &sphere) {
            not_world = true;
            if ray.position.distance_squared(collision_point) < ray.position.distance_squared(ray.last_collision) {
                ray.last_collision = collision_point;
                ray.position = ray.last_collision;
                ray.normal = (ray.position - sphere.position).normalize(); 
                ray.colour = sphere.colour;
            }
        }
    }

    if not_world {
        if depth <= 0 {
            //let mult = ((normal.dot(Vec3::new(-1.,100.,1.).normalize())/2. +0.5) * 255.) as u8;
            //ray.colour = [0, mult, mult, mult];\
            return
        }   
        let new_dir = ray.orientation.reflect(ray.normal);
        ray.position = ray.last_collision + 0.1 * ray.normal;
        ray.orientation = new_dir;
        //println!("{}", depth);
        raycast_new(&mut *ray, &spheres, &world, depth-1);
        return
    }


    let mut is_sky = false;
    
    let light_pos = Vec3::new(1.,0.,10.);


    if !not_world {
        if ray.orientation.z < 0. {
            ray.colour = world.floor.colour;
            ray.position = scale_vector_to_value(&ray.position, &ray.orientation, VecAxis::Z, world.floor.height);
            ray.normal = Vec3::new(0.,0.,1.);
            ray.orientation = (light_pos - ray.position).normalize()
        }
        else if ray.orientation.z >= 0. {
            is_sky = true;
            ray.colour = world.sky.colour;
            ray.colour = [0, ((ray.orientation.x * 500.)%255.) as u8, ((ray.orientation.y * 500.)%255.) as u8, ((ray.orientation.z * 500.)%255.) as u8];

        }
    }

    if !is_sky {

        let mut light_scale = ray.normal.normalize().dot((light_pos - ray.position).normalize()) / ray.position.distance_squared(light_pos);
        
        for sphere in spheres {
            if sphere_collision(ray, sphere).is_some() {
                light_scale = 0.;
            }
        }

        return

        //println!("{light_scale}");
        for hex in ray.colour[1..4].iter_mut() {
            *hex = (*hex as f32 * light_scale * 100.) as u8;
            //*hex = (200./ray.position.distance_squared(light_pos)) as u8;
            //println!("{} {}",ray.position.distance_squared(light_pos), ray.position); ///// IT's THE RAY POSITION, THIS RETURNS 1.01 [-0, -0, 0] (println!("{} {}",ray.position.distance_squared(light_pos), ray.position);)
        }
    }
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

    //window.set_target_fps(60);



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
        velocity: Vec3::new(0.,0.,0.),
        ..Default::default()
        },
        Sphere {
        position: Vec3::new(4., 4., 2.),
        radius: 2.,
        colour: [0,0,150,255],
        velocity: Vec3::new(0.,0.,0.),
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
        render(&camera, &mut buffer, &objects, &world);
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }


}