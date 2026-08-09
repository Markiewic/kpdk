use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Architecture {
    Pdk13,
    Pdk14,
    Pdk15,
}

impl Architecture {
    pub fn sdcc_target(self) -> &'static str {
        match self {
            Self::Pdk13 => "pdk13",
            Self::Pdk14 => "pdk14",
            Self::Pdk15 => "pdk15",
        }
    }
}

pub struct Device {
    pub name: &'static str,
    pub architecture: Architecture,
    pub otp: bool,
}

const DEVICES: &[Device] = &[
    Device { name: "PMS150C", architecture: Architecture::Pdk13, otp: true },
    Device { name: "PMS15A", architecture: Architecture::Pdk13, otp: true },
    Device { name: "PFS154", architecture: Architecture::Pdk14, otp: false },
    Device { name: "PFS172", architecture: Architecture::Pdk14, otp: false },
    Device { name: "PMS152", architecture: Architecture::Pdk14, otp: true },
    Device { name: "PMS154C", architecture: Architecture::Pdk14, otp: true },
    Device { name: "PMS171B", architecture: Architecture::Pdk14, otp: true },
    Device { name: "PFS173", architecture: Architecture::Pdk15, otp: false },
];

pub fn supported_devices() -> &'static [Device] {
    DEVICES
}

pub fn architecture(device: &str) -> Result<Architecture> {
    let normalized = device.to_ascii_uppercase();
    DEVICES
        .iter()
        .find(|item| item.name == normalized)
        .map(|item| item.architecture)
        .ok_or_else(|| Error::Message(format!(
            "unsupported device `{normalized}`; the initial device table must be extended before building it"
        )))
}

pub fn is_otp(device: &str) -> bool {
    let normalized = device.to_ascii_uppercase();
    DEVICES.iter().find(|item| item.name == normalized).map(|item| item.otp).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_pfs154() {
        assert_eq!(architecture("PFS154").unwrap(), Architecture::Pdk14);
    }

    #[test]
    fn device_lookup_is_case_insensitive() {
        assert_eq!(architecture("pms150c").unwrap(), Architecture::Pdk13);
    }
}
