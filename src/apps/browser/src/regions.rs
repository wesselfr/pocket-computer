use crate::{apps::browser::src::parser::StrSlice, input::Rect};

pub const MAX_REGIONS: usize = 16;

#[derive(Debug)]
pub enum HitAction {
    Link { href: StrSlice },
}

pub struct HitRegion {
    pub rect: Rect,
    pub action: HitAction,
}
