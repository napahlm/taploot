use std::collections::HashMap;
use std::sync::LazyLock;

/// Curated OUI table of ICS/OT/SCADA device manufacturers.
/// Maps the first 3 bytes of a MAC address to vendor name.
static OUI_TABLE: LazyLock<HashMap<[u8; 3], &'static str>> = LazyLock::new(|| {
    let entries: &[([u8; 3], &str)] = &[
        // Siemens
        ([0x00, 0x0E, 0x8C], "Siemens"),
        ([0x00, 0x1B, 0x1B], "Siemens"),
        ([0x00, 0x1C, 0x06], "Siemens"),
        ([0x08, 0x00, 0x06], "Siemens"),
        ([0xA0, 0xB9, 0xED], "Siemens"),
        ([0x4C, 0xEB, 0xBD], "Siemens"),
        ([0xAC, 0x64, 0xDD], "Siemens"),
        ([0x64, 0x9D, 0x99], "Siemens"),
        ([0x18, 0x3D, 0x5E], "Siemens"),
        ([0x00, 0x11, 0x33], "Siemens"),
        // Rockwell Automation / Allen-Bradley
        ([0x00, 0x00, 0xBC], "Rockwell Automation"),
        ([0x00, 0x1D, 0x9C], "Rockwell Automation"),
        ([0x5C, 0x88, 0x16], "Rockwell Automation"),
        ([0x00, 0xFE, 0xC8], "Rockwell Automation"),
        ([0xB8, 0x2C, 0xA0], "Rockwell Automation"),
        ([0x40, 0x9C, 0x28], "Rockwell Automation"),
        // ABB
        ([0x00, 0x80, 0x25], "ABB"),
        ([0x00, 0x21, 0x99], "ABB"),
        ([0x00, 0x22, 0x57], "ABB"),
        ([0x00, 0x24, 0xD7], "ABB"),
        ([0x64, 0x3A, 0xAB], "ABB"),
        ([0x00, 0x13, 0xE6], "ABB"),
        // Schneider Electric
        ([0x00, 0x80, 0xF4], "Schneider Electric"),
        ([0x00, 0x00, 0x54], "Schneider Electric"),
        ([0x00, 0x0F, 0x23], "Schneider Electric"),
        ([0xEC, 0x8E, 0xB5], "Schneider Electric"),
        ([0xE0, 0xCB, 0xBC], "Schneider Electric"),
        ([0x00, 0x1C, 0xEA], "Schneider Electric"),
        // Honeywell
        ([0x00, 0xD0, 0x2D], "Honeywell"),
        ([0x00, 0x04, 0xA5], "Honeywell"),
        ([0x00, 0x20, 0x85], "Honeywell"),
        ([0x00, 0x40, 0x84], "Honeywell"),
        ([0x2C, 0x44, 0xFD], "Honeywell"),
        // Emerson / Fisher-Rosemount
        ([0x00, 0xA0, 0xF8], "Emerson"),
        ([0x00, 0xD0, 0x59], "Emerson"),
        ([0x00, 0x03, 0x35], "Emerson"),
        ([0x00, 0x60, 0x35], "Emerson"),
        ([0xFC, 0x8E, 0x5B], "Emerson"),
        // Yokogawa
        ([0x00, 0xA0, 0x73], "Yokogawa"),
        ([0x00, 0x40, 0xF4], "Yokogawa"),
        ([0x04, 0x7F, 0x0E], "Yokogawa"),
        // GE Intelligent Platforms / GE Automation
        ([0x00, 0x19, 0xA7], "GE Automation"),
        ([0x00, 0x12, 0xC0], "GE Automation"),
        ([0x3C, 0xE5, 0xA6], "GE Automation"),
        // Moxa
        ([0x00, 0x90, 0xE8], "Moxa"),
        ([0x00, 0x0F, 0xE2], "Moxa"),
        // Phoenix Contact
        ([0x00, 0xA0, 0x45], "Phoenix Contact"),
        ([0x00, 0x13, 0x0A], "Phoenix Contact"),
        ([0xF0, 0x45, 0xDA], "Phoenix Contact"),
        // Beckhoff
        ([0x00, 0x01, 0x05], "Beckhoff"),
        // Wago
        ([0x00, 0x30, 0xDE], "Wago"),
        ([0x00, 0x23, 0x84], "Wago"),
        // B&R Industrial Automation
        ([0x00, 0x60, 0x65], "B&R Automation"),
        ([0xF0, 0x3E, 0x90], "B&R Automation"),
        // Hirschmann / Belden
        ([0x00, 0x80, 0x63], "Hirschmann"),
        ([0xEC, 0xE5, 0x55], "Hirschmann"),
        // SEL (Schweitzer Engineering)
        ([0x00, 0x30, 0xA7], "SEL"),
        ([0x00, 0x1C, 0x7C], "SEL"),
        // WAGO Kontakttechnik
        ([0x00, 0x50, 0xC2], "Red Lion"),
        // Advantech
        ([0x00, 0xD0, 0xC9], "Advantech"),
        ([0xE0, 0xD5, 0x5E], "Advantech"),
        ([0x88, 0xA4, 0xC2], "Advantech"),
        // Mitsubishi Electric
        ([0x00, 0x00, 0x5E], "Mitsubishi Electric"),
        ([0x00, 0x08, 0xEE], "Mitsubishi Electric"),
        ([0x00, 0x0C, 0xC6], "Mitsubishi Electric"),
        // Omron
        ([0x00, 0x00, 0x35], "Omron"),
        ([0x00, 0x1A, 0xBB], "Omron"),
        // Endress+Hauser
        ([0x00, 0x0E, 0x26], "Endress+Hauser"),
        // SICK AG
        ([0x00, 0x06, 0xC5], "SICK"),
        // Pilz
        ([0x00, 0x0C, 0xF1], "Pilz"),
        // Turck
        ([0x00, 0x07, 0x83], "Turck"),
        // Pepperl+Fuchs / Comtrol
        ([0x00, 0xC0, 0x4E], "Pepperl+Fuchs"),
        // IFM Electronic
        ([0x00, 0x02, 0x01], "IFM Electronic"),
        // Lenze
        ([0x00, 0x0E, 0x91], "Lenze"),
        // Danfoss
        ([0x00, 0x04, 0xA1], "Danfoss"),
        // Eaton / Cutler-Hammer
        ([0x00, 0x01, 0xF4], "Eaton"),
        ([0xEC, 0xE5, 0x12], "Eaton"),
        // Belden / Tofino
        ([0x00, 0x1E, 0x7A], "Belden"),
        // WEIDMULLER
        ([0x00, 0xD0, 0x93], "Weidmuller"),
        // National Instruments
        ([0x00, 0x80, 0x2F], "National Instruments"),
        ([0x3C, 0x0E, 0x23], "National Instruments"),
        // Lantronix
        ([0x00, 0x80, 0xA3], "Lantronix"),
        ([0x00, 0x20, 0x4A], "Lantronix"),
        // Digi International
        ([0x00, 0x40, 0x9D], "Digi International"),
        ([0x00, 0x13, 0xA2], "Digi International"),
        // HMS Industrial Networks
        ([0x00, 0x30, 0x11], "HMS Industrial"),
        // ProSoft Technology
        ([0x00, 0x24, 0x96], "ProSoft Technology"),
        // Kepware / PTC
        ([0x00, 0x0B, 0x3C], "Cimetrics"),
        // Cisco (common in OT switches)
        ([0x00, 0x00, 0x0C], "Cisco"),
        ([0x00, 0x1A, 0xA1], "Cisco"),
        ([0x00, 0x1B, 0x54], "Cisco"),
        ([0x00, 0x25, 0x45], "Cisco"),
        ([0xF8, 0x72, 0xEA], "Cisco"),
        ([0x68, 0xBD, 0xAB], "Cisco"),
        // HP / HPE (common in OT infrastructure)
        ([0x00, 0x1E, 0x0B], "HP"),
        ([0x00, 0x25, 0xB3], "HP"),
        ([0x3C, 0xD9, 0x2B], "HP"),
        // Dell (common in OT infrastructure)
        ([0x00, 0x14, 0x22], "Dell"),
        ([0x18, 0x03, 0x73], "Dell"),
        // VMware (virtual OT environments)
        ([0x00, 0x50, 0x56], "VMware"),
        ([0x00, 0x0C, 0x29], "VMware"),
        // Ruggedcom / Siemens Ruggedcom
        ([0x00, 0x18, 0x82], "Ruggedcom"),
        ([0x00, 0x15, 0xAC], "Ruggedcom"),
        // Westermo
        ([0x00, 0x07, 0x7C], "Westermo"),
        // CODESYS / 3S-Smart Software
        ([0x00, 0x15, 0xF7], "CODESYS"),
        // NetModule
        ([0xF8, 0xDC, 0x7A], "NetModule"),
        // Hilscher
        ([0x00, 0x02, 0xA2], "Hilscher"),
        // KUKA Robotics
        ([0x00, 0x04, 0x0B], "KUKA"),
        // Fanuc
        ([0x00, 0xE0, 0x09], "Fanuc"),
        // Keyence
        ([0x00, 0x01, 0xA3], "Keyence"),
        // Yaskawa
        ([0xB8, 0xAC, 0x6F], "Yaskawa"),
        // Bosch Rexroth
        ([0x00, 0x05, 0x51], "Bosch Rexroth"),
        // Parker Hannifin
        ([0x00, 0x19, 0xB2], "Parker Hannifin"),
        // Festo
        ([0x00, 0x0E, 0xF6], "Festo"),
        // Balluff
        ([0xF4, 0x3D, 0x80], "Balluff"),
        // EtherCAT / Beckhoff (additional)
        ([0x00, 0x01, 0x4A], "Sony/EtherCAT"),
        // Stoneridge
        ([0xF8, 0xB4, 0x6A], "Hewlett Packard Enterprise"),
        // Intel (common ethernet controllers in ICS)
        ([0x00, 0x1B, 0x21], "Intel"),
        ([0x00, 0x1E, 0x67], "Intel"),
        ([0x3C, 0xFD, 0xFE], "Intel"),
        ([0xA4, 0xBF, 0x01], "Intel"),
        // Broadcom (common in ICS network cards)
        ([0x00, 0x10, 0x18], "Broadcom"),
        // Realtek (common embedded NIC)
        ([0x00, 0xE0, 0x4C], "Realtek"),
        ([0x52, 0x54, 0x00], "QEMU/KVM"),
    ];

    let mut map = HashMap::with_capacity(entries.len());
    for &(prefix, vendor) in entries {
        map.insert(prefix, vendor);
    }
    map
});

/// Look up the vendor name for a MAC address string (e.g., "00:0e:8c:12:34:56").
/// Returns `None` if the prefix is not in the curated ICS/OT table.
pub fn lookup_vendor(mac: &str) -> Option<&'static str> {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let prefix = [
        u8::from_str_radix(parts[0], 16).ok()?,
        u8::from_str_radix(parts[1], 16).ok()?,
        u8::from_str_radix(parts[2], 16).ok()?,
    ];
    OUI_TABLE.get(&prefix).copied()
}
