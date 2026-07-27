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

pub fn architecture(device: &str) -> Result<Architecture> {
    match device.to_ascii_uppercase().as_str() {
        "PMS150C" | "PMS15A" => Ok(Architecture::Pdk13),
        "PFS154" | "PFS172" | "PMS152" | "PMS154C" | "PMS171B" => {
            Ok(Architecture::Pdk14)
        }
        "PFS173" => Ok(Architecture::Pdk15),
        other => Err(Error::Message(format!(
            "unsupported device `{other}`; the initial device table must be extended before building it"
        ))),
    }
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
}
