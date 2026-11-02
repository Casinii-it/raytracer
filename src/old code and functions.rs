
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
