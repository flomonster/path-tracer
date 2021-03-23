use super::Ray;
use cgmath::*;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::*;
use std::sync::Mutex;

static LOG_FILE: Lazy<Mutex<File>> = Lazy::new(|| Mutex::new(File::create("log.obj").unwrap()));
static LOG_INDEX: AtomicUsize = AtomicUsize::new(1);

pub fn line(id: &str, origin: &Vector3<f32>, dest: &Vector3<f32>) {
    let index = LOG_INDEX.fetch_add(2, Ordering::Relaxed);
    let mut file = LOG_FILE.lock().unwrap();

    write!(file, "o {}\n", id).unwrap();
    write!(file, "v {} {} {}\n", origin.x, origin.y, origin.z).unwrap();
    write!(file, "v {} {} {}\n", dest.x, dest.y, dest.z).unwrap();
    write!(file, "l {} {}\n\n", index, index + 1).unwrap();
}

pub fn ray(id: &str, ray: &Ray) {
    line(id, &ray.origin, &(ray.origin + ray.direction));
}
