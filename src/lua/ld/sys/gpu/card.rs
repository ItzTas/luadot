use std::fs;
use std::path::Path;

use super::constants::{CARD_PREFIX, DEVICE_LINK, DRIVER_LINK, DRM_DIR, VENDOR_FILE, VENDORS};
use super::model::Models;

pub struct Card {
    pub vendor: String,
    pub name: String,
    pub driver: String,
}

pub fn cards(models: &Models) -> Vec<Card> {
    cards_in(Path::new(DRM_DIR), models)
}

fn cards_in(drm: &Path, models: &Models) -> Vec<Card> {
    let mut names: Vec<String> = fs::read_dir(drm)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_card(name))
        .collect();
    names.sort();

    names
        .iter()
        .filter_map(|name| card(&drm.join(name), models))
        .collect()
}

fn is_card(name: &str) -> bool {
    let Some(index) = name.strip_prefix(CARD_PREFIX) else {
        return false;
    };

    !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())
}

fn card(path: &Path, models: &Models) -> Option<Card> {
    let vendor = vendor(&fs::read_to_string(path.join(VENDOR_FILE)).ok()?);
    let name = linked(&path.join(DEVICE_LINK))
        .and_then(|slot| models.get(&slot).cloned())
        .unwrap_or_default();

    Some(Card {
        vendor,
        name,
        driver: linked(&path.join(DRIVER_LINK)).unwrap_or_default(),
    })
}

fn vendor(raw: &str) -> String {
    let id = raw.trim().to_lowercase();

    VENDORS
        .iter()
        .find(|(known, _)| *known == id)
        .map_or(id.clone(), |(_, name)| (*name).to_string())
}

fn linked(path: &Path) -> Option<String> {
    let target = fs::read_link(path).ok()?;

    Some(target.file_name()?.to_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    fn drm(cards: &[(&str, &str, &str, &str)]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let drm = root.path().join("drm");
        let bus = root.path().join("bus");
        fs::create_dir_all(&drm).unwrap();
        fs::create_dir_all(&bus).unwrap();

        for (name, slot, vendor, driver) in cards {
            let device = bus.join(slot);
            fs::create_dir_all(&device).unwrap();
            fs::write(device.join("vendor"), format!("{vendor}\n")).unwrap();

            let module = bus.join(driver);
            fs::create_dir_all(&module).unwrap();
            symlink(&module, device.join("driver")).unwrap();

            let card = drm.join(name);
            fs::create_dir_all(&card).unwrap();
            symlink(&device, card.join(DEVICE_LINK)).unwrap();
        }

        root
    }

    #[test]
    fn reads_every_card_in_order() {
        let root = drm(&[
            ("card1", "0000:01:00.0", "0x10de", "nvidia"),
            ("card0", "0000:00:02.0", "0x8086", "i915"),
            ("card0-eDP-1", "0000:00:02.1", "0x8086", "i915"),
        ]);
        let models = Models::from([(
            "0000:01:00.0".to_string(),
            "AD107M [GeForce RTX 4060]".to_string(),
        )]);

        let cards = cards_in(&root.path().join("drm"), &models);

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].vendor, "intel");
        assert_eq!(cards[0].driver, "i915");
        assert_eq!(cards[0].name, "");
        assert_eq!(cards[1].vendor, "nvidia");
        assert_eq!(cards[1].driver, "nvidia");
        assert_eq!(cards[1].name, "AD107M [GeForce RTX 4060]");
    }

    #[test]
    fn a_missing_directory_has_no_cards() {
        assert!(cards_in(Path::new("/nonexistent/drm"), &Models::new()).is_empty());
    }

    #[test]
    fn an_unknown_identifier_stays_as_it_is() {
        assert_eq!(vendor("0x10DE\n"), "nvidia");
        assert_eq!(vendor("0xbeef\n"), "0xbeef");
    }

    #[test]
    fn a_connector_is_not_a_card() {
        assert!(is_card("card0"));
        assert!(is_card("card12"));
        assert!(!is_card("card0-eDP-1"));
        assert!(!is_card("card"));
        assert!(!is_card("renderD128"));
    }
}
