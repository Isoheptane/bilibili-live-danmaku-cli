use colored::{ColoredString, Colorize};

#[derive(Debug, Clone, Copy)]
pub enum GuardLevel {
    Captain     = 3,    // 艦長
    Commander   = 2,    // 提督
    Governor    = 1,    // 總督
}

impl TryFrom<u64> for GuardLevel {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(GuardLevel::Captain),
            2 => Ok(GuardLevel::Commander),
            1 => Ok(GuardLevel::Governor),
            _ => Err(())
        }
    }
}

impl GuardLevel {
    pub fn name(&self) -> &'static str {
        match self {
            GuardLevel::Captain => "艦長",
            GuardLevel::Commander => "提督",
            GuardLevel::Governor => "總督",
        }
    }

    pub fn colorize(&self, text: &str) -> ColoredString {
        match self {
            GuardLevel::Captain => text.bright_blue(),
            GuardLevel::Commander => text.bright_purple(),
            GuardLevel::Governor => text.bright_yellow(),
        }
    }
}