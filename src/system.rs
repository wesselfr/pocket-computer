use crate::touch::TouchCalibration;

pub enum SystemCmd {
    StartCalibration,
    ApplyCalibration(TouchCalibration),
    SetBrightness(u8),
}
