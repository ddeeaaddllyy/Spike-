use esp_bootloader_esp_idf::partitions::{PARTITION_TABLE_MAX_LEN, read_partition_table};
use esp_storage::FlashStorage;

#[derive(Clone, Copy, PartialEq)]
pub struct Preferences {
    pub timeout: usize,
    // One step is ten percent. Never let the display become unreadably dark.
    pub brightness: u8,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            timeout: 1,
            brightness: 5,
        }
    }
}

impl Preferences {
    pub fn panel_level(self) -> u8 {
        (u16::from(self.brightness) * 255 / 10) as u8
    }
}

const SECTOR: u32 = 4096;

pub struct Store {
    flash: FlashStorage<'static>,
    base: Option<u32>,
    active: Option<(u32, u32)>,
}

impl Store {
    pub fn load(mut flash: FlashStorage<'static>) -> (Self, Preferences) {
        let mut table = [0; PARTITION_TABLE_MAX_LEN];
        let base = read_partition_table(&mut flash, &mut table)
            .ok()
            .and_then(|table| {
                table
                    .iter()
                    .find(|p| {
                        p.label_as_str() == "spikecfg"
                            && p.raw_type() == 1
                            && p.raw_subtype() == 0x06
                            && p.len() == 2 * SECTOR
                            && !p.is_read_only()
                            && !p.is_encrypted()
                    })
                    .map(|p| p.offset())
            });
        let mut store = Self {
            flash,
            base,
            active: None,
        };
        let mut prefs = Preferences::default();
        if let Some(base) = base {
            for slot in 0..2 {
                let mut record = [0; 16];
                if store
                    .flash
                    .read_nor(base + slot * SECTOR, &mut record)
                    .is_err()
                {
                    store.base = None;
                    break;
                }
                if let Some((seq, value)) = decode(&record)
                    && store
                        .active
                        .is_none_or(|(_, old)| (seq.wrapping_sub(old) as i32) > 0)
                {
                    store.active = Some((slot, seq));
                    prefs = value;
                }
            }
        }
        (store, prefs)
    }

    pub fn save(&mut self, prefs: Preferences) -> bool {
        let Some(base) = self.base else { return false };
        // Leave the previous record intact until the replacement has been verified.
        let (slot, seq) = self
            .active
            .map_or((0, 0), |(n, seq)| (1 - n, seq.wrapping_add(1)));
        let address = base + slot * SECTOR;
        let mut record = [0; 16];
        record[..4].copy_from_slice(b"SPK1");
        record[4..8].copy_from_slice(&seq.to_le_bytes());
        record[8..12].copy_from_slice(&(!seq).to_le_bytes());
        let t = prefs.timeout as u8;
        let b = prefs.brightness;
        record[12..].copy_from_slice(&[t, b, !t, !b]);
        let mut check = [0; 16];
        if self.flash.erase(address, address + SECTOR).is_err()
            || self.flash.write_nor(address, &record).is_err()
            || self.flash.read_nor(address, &mut check).is_err()
            || check != record
        {
            return false;
        }
        self.active = Some((slot, seq));
        true
    }
}

fn decode(record: &[u8; 16]) -> Option<(u32, Preferences)> {
    let seq = u32::from_le_bytes(record[4..8].try_into().unwrap());
    let inv = u32::from_le_bytes(record[8..12].try_into().unwrap());
    let [t, b, ti, bi] = record[12..].try_into().unwrap();
    (record[..4] == *b"SPK1"
        && seq == !inv
        && t == !ti
        && b == !bi
        && t < 3
        && (1..=10).contains(&b))
    .then_some((
        seq,
        Preferences {
            timeout: usize::from(t),
            brightness: b,
        },
    ))
}
