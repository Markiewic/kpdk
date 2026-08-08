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

const SUPPORTED_DEVICES: &[(&str, Architecture)] = &[
    ("PMS150C", Architecture::Pdk13),
    ("PMS15A", Architecture::Pdk13),
    ("PFS154", Architecture::Pdk14),
    ("PFS172", Architecture::Pdk14),
    ("PMS152", Architecture::Pdk14),
    ("PMS154C", Architecture::Pdk14),
    ("PMS171B", Architecture::Pdk14),
    ("PFS173", Architecture::Pdk15),
];

pub fn supported_devices() -> impl Iterator<Item = (&'static str, Architecture)> {
    SUPPORTED_DEVICES.iter().copied()
}

pub fn architecture(device: &str) -> Result<Architecture> {
    let normalized = device.to_ascii_uppercase();
    SUPPORTED_DEVICES
        .iter()
        .find(|(name, _)| *name == normalized)
        .map(|(_, architecture)| *architecture)
        .ok_or_else(|| {
            Error::Message(format!(
                "unsupported device `{normalized}`; the initial device table must be extended before building it"
            ))
        })
}

pub fn is_otp(device: &str) -> bool {
    device.to_ascii_uppercase().starts_with("PMS")
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

    #[test]
    fn supported_device_table_matches_lookup() {
        for (name, expected) in supported_devices() {
            assert_eq!(architecture(name).unwrap(), expected);
        }
    }
}
