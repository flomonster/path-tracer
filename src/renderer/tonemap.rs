use cgmath::*;
use serde::Deserialize;

#[derive(Copy, Debug, Clone, Deserialize)]
pub enum TonemapType {
    #[serde(rename = "REINHARD")]
    Reinhard,
    #[serde(rename = "FILMIC")]
    Filmic,
    #[serde(rename = "ACES")]
    Aces,
}

pub fn tonemap(tonemap_type: TonemapType, color: Vector3<f32>) -> Vector3<f32> {
    match tonemap_type {
        TonemapType::Reinhard => reinhard(color),
        TonemapType::Filmic => filmic(color),
        TonemapType::Aces => aces(color),
    }
}

fn reinhard(color: Vector3<f32>) -> Vector3<f32> {
    color.div_element_wise(color + Vector3::new(1., 1., 1.))
}

fn filmic(color: Vector3<f32>) -> Vector3<f32> {
    let a = Vector3::new(0.004, 0.004, 0.004);
    let color = color - a;
    let color = Vector3::new(color.x.max(0.), color.y.max(0.), color.z.max(0.));

    let b = Vector3::new(0.5, 0.5, 0.5);
    let c = Vector3::new(1.7, 1.7, 1.7);
    let d = Vector3::new(0.06, 0.06, 0.06);
    let num = color.mul_element_wise(6.2 * color + b);
    let denom = color.mul_element_wise(6.2 * color + c) + d;
    num.div_element_wise(denom)
}

fn aces(color: Vector3<f32>) -> Vector3<f32> {
    let a = 2.51;
    let b = Vector3::new(0.03, 0.03, 0.03);
    let c = 2.43;
    let d = Vector3::new(0.59, 0.59, 0.59);
    let e = Vector3::new(0.14, 0.14, 0.14);
    let num = color.mul_element_wise(a * color + b);
    let denom = color.mul_element_wise(c * color + d) + e;
    let res = num.div_element_wise(denom);
    Vector3::new(
        res.x.max(0.).min(1.),
        res.y.max(0.).min(1.),
        res.z.max(0.).min(1.),
    )
}
