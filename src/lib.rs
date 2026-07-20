pub struct Battery {
    pub name: String,
    pub status: BatteryStatus,
    pub capacity: u8,
    pub charge_full: Option<u32>,
    pub charge_full_design: Option<u32>,
    pub charge_now: Option<u32>,
    pub cycle_count: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

#[derive(Debug)]
pub enum Berror {
    NoBatteriesFound,
    Io { path: String, source: std::io::Error },
    Parse { field: &'static str, source: std::num::ParseIntError },
}

impl std::str::FromStr for BatteryStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Charging" => BatteryStatus::Charging,
            "Discharging" => BatteryStatus::Discharging,
            "Full" => BatteryStatus::Full,
            "Not charging" => BatteryStatus::NotCharging,
            _ => BatteryStatus::Unknown,
        })
    }
}

fn read_field<T: std::str::FromStr>(path: &str) -> Option<T> {
    let contents = std::fs::read_to_string(path).ok()?;
    contents.trim().parse::<T>().ok()
}

pub fn read_battery(path: &str) -> Result<Battery, Berror> {
    let name = path.rsplit('/').next().unwrap().to_string();

    let status_path = format!("{path}/status");
    let rstatus = std::fs::read_to_string(&status_path)
        .map_err(|e| Berror::Io { path: status_path.clone(), source: e })?;
    let status: BatteryStatus = rstatus.parse().unwrap(); // still safe: Infallible

    let capacity_path = format!("{path}/capacity");
    let rcapacity = std::fs::read_to_string(&capacity_path)
        .map_err(|e| Berror::Io { path: capacity_path.clone(), source: e })?;
    let capacity: u8 = rcapacity.trim().parse()
        .map_err(|e| Berror::Parse { field: "capacity", source: e })?;

    Ok(Battery {
        name,
        status,
        capacity,
        charge_full: read_field(&format!("{path}/charge_full")),
        charge_full_design: read_field(&format!("{path}/charge_full_design")),
        charge_now: read_field(&format!("{path}/charge_now")),
        cycle_count: read_field(&format!("{path}/cycle_count")),
    })
}

pub fn read_batteries() -> Result<Vec<Battery>, Berror> {
    let dir = "/sys/class/power_supply";
    let entries = std::fs::read_dir(dir)
        .map_err(|e| Berror::Io { path: dir.to_string(), source: e })?;

    let mut batteries = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| Berror::Io { path: dir.to_string(), source: e })?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        let is_battery = path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("BAT"))
            .unwrap_or(false);

        if is_battery {
            batteries.push(read_battery(&path_str)?);
        }
    }

    if batteries.is_empty() {
        return Err(Berror::NoBatteriesFound);
    }

    Ok(batteries)
}