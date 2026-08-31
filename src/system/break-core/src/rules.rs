use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub fn from_chrono(day: chrono::Weekday) -> Self {
        match day {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Thursday,
            chrono::Weekday::Fri => Weekday::Friday,
            chrono::Weekday::Sat => Weekday::Saturday,
            chrono::Weekday::Sun => Weekday::Sunday,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

impl TimeOfDay {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }

    pub fn minutes_from_midnight(&self) -> u16 {
        u16::from(self.hour) * 60 + u16::from(self.minute)
    }

    pub fn is_valid(&self) -> bool {
        self.hour < 24 && self.minute < 60
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindow {
    pub id: String,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub days: Vec<Weekday>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteTarget {
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTarget {
    pub bundle_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    #[serde(default)]
    pub windows: Vec<TimeWindow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    pub version: u32,
    #[serde(default)]
    pub sites: Vec<SiteTarget>,
    #[serde(default)]
    pub apps: Vec<AppTarget>,
    #[serde(default)]
    pub schedule: Schedule,
}

pub const RULES_VERSION: u32 = 1;

impl Default for Rules {
    fn default() -> Self {
        Self {
            version: RULES_VERSION,
            sites: Vec::new(),
            apps: Vec::new(),
            schedule: Schedule::default(),
        }
    }
}

pub fn normalize_host(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_scheme = match trimmed.find("://") {
        Some(index) => &trimmed[index + 3..],
        None => trimmed,
    };

    let authority = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let authority = authority.rsplit('@').next().unwrap_or_default();
    let host = authority.split(':').next().unwrap_or_default();

    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_string();

    if host.is_empty() || !host.contains('.') {
        return None;
    }
    if host
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
    {
        return None;
    }

    Some(host)
}

impl SiteTarget {
    pub fn from_input(input: &str) -> Option<Self> {
        normalize_host(input).map(|host| Self { host })
    }
}

impl Rules {
    pub fn add_site(&mut self, input: &str) -> Option<&SiteTarget> {
        let target = SiteTarget::from_input(input)?;
        if self.sites.iter().any(|site| site.host == target.host) {
            return None;
        }
        self.sites.push(target);
        self.sites.last()
    }

    pub fn remove_site(&mut self, host: &str) {
        self.sites.retain(|site| site.host != host);
    }
}
