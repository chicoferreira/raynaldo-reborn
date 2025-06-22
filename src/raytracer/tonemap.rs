use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tonemapper {
    None,
    Reinhard,
    ReinhardWithHKSaturation,
    Filmic,
}

impl Tonemapper {
    pub fn all() -> &'static [Tonemapper] {
        &[
            Tonemapper::None,
            Tonemapper::Reinhard,
            Tonemapper::ReinhardWithHKSaturation,
            Tonemapper::Filmic,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tonemapper::None => "None",
            Tonemapper::Reinhard => "Reinhard",
            Tonemapper::ReinhardWithHKSaturation => "Reinhard with H&K Saturation",
            Tonemapper::Filmic => "Filmic",
        }
    }

    pub fn apply(&self, color: Vec3) -> Vec3 {
        match self {
            Tonemapper::None => color,
            Tonemapper::Reinhard => {
                let x = color.x;
                let y = color.y;
                let z = color.z;
                Vec3::new(x / (1.0 + x), y / (1.0 + y), z / (1.0 + z))
            }
            Tonemapper::ReinhardWithHKSaturation => {
                let luminance = 0.2126 * color.x + 0.7152 * color.y + 0.0722 * color.z;
                let compressed_luminance = luminance / (1.0 + luminance);

                let saturation = if luminance > 0.0 {
                    let max_channel = color.x.max(color.y).max(color.z);
                    let min_channel = color.x.min(color.y).min(color.z);
                    (max_channel - min_channel) / max_channel
                } else {
                    0.0
                };

                let hk_boost = 1.0 + 0.2 * saturation;
                let adjusted_luminance = (compressed_luminance * hk_boost).clamp(0.0, 1.0);

                let scale = if luminance > 0.0 {
                    adjusted_luminance / luminance
                } else {
                    1.0
                };

                Vec3::new(color.x * scale, color.y * scale, color.z * scale)
            }
            Tonemapper::Filmic => {
                let a = 0.15;
                let b = 0.50;
                let c = 0.10;
                let d = 0.20;
                let e = 0.02;

                let x = color.x * (a * color.x + b) / (color.x * (c * color.x + d) + e);
                let y = color.y * (a * color.y + b) / (color.y * (c * color.y + d) + e);
                let z = color.z * (a * color.z + b) / (color.z * (c * color.z + d) + e);

                Vec3::new(x, y, z)
            }
        }
    }
}
