#[derive(Debug, Clone, Copy)]
pub(crate) enum RocConfiguration {
    Beans,
    BeansSpecialisation,
    Perceus,
    PerceusSpecialisation,
}

impl RocConfiguration {
    pub(crate) fn flags(&self) -> String {
        match self {
            RocConfiguration::Beans => "".to_string(),
            RocConfiguration::BeansSpecialisation => "--cfg DROP_SPECIALIZE".to_string(),
            RocConfiguration::Perceus => "--cfg PERCEUS_RC".to_string(),
            RocConfiguration::PerceusSpecialisation => {
                "--cfg PERCEUS_RC --cfg DROP_SPECIALIZE".to_string()
            }
        }
    }
}
