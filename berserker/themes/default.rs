/// Default Niflheim Theme for CVKG
pub struct Theme {
    pub primary: [f32; 4],
    pub background: [f32; 4],
}

pub const DEFAULT_THEME: Theme = Theme {
    primary: [0.0, 1.0, 1.0, 1.0],
    background: [0.05, 0.05, 0.1, 1.0],
};
