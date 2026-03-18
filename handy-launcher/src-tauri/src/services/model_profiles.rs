pub struct ModelProfile {
    pub name: &'static str,
    pub ram: u32,
    pub size: &'static str,
}

pub fn default_profiles() -> [ModelProfile; 3] {
    [
        ModelProfile {
            name: "fast",
            ram: 2,
            size: "1.3 GB",
        },
        ModelProfile {
            name: "recommended",
            ram: 6,
            size: "3.8 GB",
        },
        ModelProfile {
            name: "accurate",
            ram: 8,
            size: "7 GB",
        },
    ]
}
