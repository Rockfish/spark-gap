use glam::Vec2;
use rand::{thread_rng, Rng};
// use rand_distr::{Distribution, Normal};
use crate::error::Error;
use crate::error::Error::PathError;
use std::cmp::Ordering;
use std::ops::Mul;
use std::path::{Path, PathBuf};

#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

pub fn f32_max(a: f32, b: f32) -> f32 {
    match a.partial_cmp(&b).unwrap() {
        Ordering::Less => b,
        Ordering::Equal => b,
        Ordering::Greater => a,
    }
}

pub fn rand_int(x: i32, y: i32) -> i32 {
    thread_rng().gen_range(x..=y)
}

pub fn rand_float() -> f32 {
    thread_rng().gen_range(0.0..=1.0)
}

pub fn rand_in_range(x: f32, y: f32) -> f32 {
    thread_rng().gen_range(x..=y)
}

pub fn rand_bool() -> bool {
    thread_rng().gen_range(0.0..=1.0) > 0.5
}

//returns a random float in the range -1 < n < 1
pub fn random_clamped() -> f32 {
    thread_rng().gen_range(-1.0..=1.0)
}

// pub fn rand_normal_distribution() -> f32 {
//     let normal: Normal<f32> = Normal::new(2.0, 0.2).unwrap();
//     let v = normal.sample(&mut thread_rng());
//     v
// }

pub fn truncate(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        let v = v.normalize_or_zero();
        return v.mul(max);
    }
    v
}

pub fn wrap_around(pos: &mut Vec2, max_x: i32, max_y: i32) {
    if pos.x > max_x as f32 {
        pos.x -= max_x as f32;
    }

    if pos.x < 0.0 {
        pos.x += max_x as f32;
    }

    if pos.y < 0.0 {
        pos.y += max_y as f32;
    }

    if pos.y > max_y as f32 {
        pos.y -= max_y as f32
    }
}

pub fn get_exists_filename(directory: &Path, filename: &str) -> Result<PathBuf, Error> {
    let path = directory.join(filename);
    if path.is_file() {
        return Ok(path);
    }
    let filepath = PathBuf::from(filename.replace('\\', "/"));
    let filename = filepath.file_name().unwrap();
    let path = directory.join(filename);
    if path.is_file() {
        return Ok(path);
    }
    Err(PathError(format!("filename not found: {:?}", filename.to_os_string())))
}

#[cfg(test)]
mod tests {
    // use crate::utils::{rand_normal_distribution, truncate, wrap_around};
    use crate::utils::{truncate, wrap_around};
    use glam::vec2;

    #[test]
    pub fn test_truncate() {
        let mut v = vec2(100.0, 100.0);
        println!("length: {}", v.length());

        v = truncate(v, 5.0);
        println!("vec: {:?}  length: {}", v, v.length());
    }

    #[test]
    pub fn test_wraparound() {
        let mut v = vec2(10.0, 10.0);
        wrap_around(&mut v, 8, 8);
        println!("{:?}", v);
        let mut v = vec2(10.0, 10.0);
        wrap_around(&mut v, 10, 11);
        println!("{:?}", v);
    }

    // #[test]
    // pub fn test_distribution() {
    //     for i in 0..1000 {
    //         let x = rand_normal_distribution() - 2.0;
    //         debug!("{x}");
    //     }
    // }

    #[test]
    pub fn test_projection() {
        let wave_vec = vec2(10.0, 10.0);
        let spot_vec = vec2(9.0, 4.0);

        let proj_vec = spot_vec.project_onto(wave_vec);

        println!("{:?}", proj_vec);
    }
}
