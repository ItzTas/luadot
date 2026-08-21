use std::collections::HashMap;
use std::process::Command;
use std::sync::OnceLock;

use super::constants::{LSPCI, LSPCI_ARGS};

pub type Models = HashMap<String, String>;

pub fn models() -> &'static Models {
    static MODELS: OnceLock<Models> = OnceLock::new();

    MODELS.get_or_init(|| parse(&lspci().unwrap_or_default()))
}

fn lspci() -> Option<String> {
    let output = Command::new(LSPCI).args(LSPCI_ARGS).output().ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
}

fn parse(output: &str) -> Models {
    output.lines().filter_map(entry).collect()
}

fn entry(line: &str) -> Option<(String, String)> {
    let (slot, rest) = line.split_once(' ')?;
    let mut quoted = rest.split('"').skip(1).step_by(2);

    let _class = quoted.next()?;
    let _vendor = quoted.next()?;
    let device = quoted.next()?;

    Some((slot.to_string(), device.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUT: &str = concat!(
        r#"0000:00:02.0 "VGA compatible controller" "Intel Corporation" "Raptor Lake-P [Iris Xe]" -ra4 "Lenovo" "Device 22e8""#,
        "\n",
        r#"0000:01:00.0 "3D controller" "NVIDIA Corporation" "AD107M [GeForce RTX 4060]" -ra1 "Lenovo" "Device 3a41""#,
        "\n",
    );

    #[test]
    fn maps_every_slot_to_its_device_name() {
        let models = parse(OUTPUT);

        assert_eq!(models["0000:00:02.0"], "Raptor Lake-P [Iris Xe]");
        assert_eq!(models["0000:01:00.0"], "AD107M [GeForce RTX 4060]");
    }
}
