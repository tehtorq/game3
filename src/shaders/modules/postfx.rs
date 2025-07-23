pub mod motion_blur {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/motion_blur.frag");
}

pub mod chromatic_aberration {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/chromatic_aberration.frag");
}

pub mod bloom {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/bloom.frag");
}

pub mod damage_shield {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/damage_shield.frag");
}

pub mod speed_lines {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/speed_lines.frag");
}

pub mod crt {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/crt.frag");
}

pub mod vignette {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/vignette.frag");
}

pub mod combined {
    pub const VERTEX: &str = include_str!("../glsl/postfx/base.vert");
    pub const FRAGMENT: &str = include_str!("../glsl/postfx/combined.frag");
}