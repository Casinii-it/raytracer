use std::vec;

use glam::{Vec2, Vec3};
use minifb::{Key, Window, WindowOptions};



const WIDTH: usize = 1600;
const HEIGHT: usize = 900;

#[derive(Debug)]
struct Camera {
    position: Vec3,
    velocity: Vec3,
    orientation: Vec3,

    fov: f32,
    resolution: Vec2
}


#[derive(Debug)]
struct Ray {
    position: Vec3,
    orientation: Vec3,
    colour: [u8;4]
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



fn render(camera: &Camera, buffer: &mut Vec<u32>) {
    let focal_length = find_focal_length(&camera);
    let normalized_orientation = camera.orientation.normalize_or(Vec3::new(1.,0.,0.));

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
            colour: [0,0,150,255]
        };
        
        raycast(&mut ray);

        *pixel = vec2int(ray.colour);

    }
}

fn raycast(ray: &mut Ray) {
    for i in 0..100 {
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
            ray.position += ray.orientation * 1.;
            //if ray.orientation.z < 0. {
            //    println!("{ray:?}");
            //}
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

    window.set_target_fps(60);



    let mut camera = Camera{
        position: Vec3::new(0.,0.,1.),
        velocity: Vec3::ZERO,
        orientation: Vec3::new(1.,0.,0.),

        fov: 150.,
        resolution: Vec2::new(WIDTH as f32,HEIGHT as f32)
    };








    
    while window.is_open() && !window.is_key_down(Key::Escape) {

        render(&camera, &mut buffer);
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }


}