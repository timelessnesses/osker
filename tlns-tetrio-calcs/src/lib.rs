use std::str::FromStr;

use reqwest::ClientBuilder;

pub const API: &'static str = "https://ch.tetr.io/api/";

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileStats {
    pub apm: f32,
    pub pps: f32,
    pub vs: f32,
    pub rank: Option<Ranks>,
    pub tr: Option<f64>,
    pub name: Option<String>,
    pub pfp: Option<String>,
    pub glicko: Option<f64>,
    pub rd: Option<f64>,
    pub is_real: bool,
}

macro_rules! enum_from_string {
    ($name:ident { $($variant:ident),* }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub enum $name {
            $($variant),*
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().replace("-", "Minus").replace("+", "Plus").as_str() {
                    $(stringify!($variant) => Ok($name::$variant),)*
                    _ => Err(()),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( $name::$variant => write!(f, stringify!($variant)), )*
                }
            }
        }
    };
}

enum_from_string!(Ranks {
    ALL,
    XPlus,
    X,
    U,
    SS,
    SPlus,
    S,
    SMinus,
    APlus,
    A,
    AMinus,
    BPlus,
    B,
    BMinus,
    CPlus,
    C,
    CMinus,
    DPlus,
    D,
    // unranked
    Z
});

pub mod weights {
    // Weights for area stat
    pub const APM_WEIGHT: usize = 1;
    pub const PPS_WEIGHT: usize = 45;
    pub const VS_WEIGHT: f64 = 0.444;
    pub const APP_WEIGHT: usize = 185;
    pub const DS_SECONDS_WEIGHT: usize = 175;
    pub const DS_PIECES_WEIGHT: usize = 450;
    pub const DS_APP_WEIGHT: usize = 140;
    pub const VS_APM_WEIGHT: usize = 60;
    pub const CHEESE_INDEX_WEIGHT: f64 = 1.25;
    pub const GARBAGE_EFFICIENCY_WEIGHT: usize = 315;

    // Weights for stat ranks and estimate tr and glicko
    pub const APM_SRW: usize = 0;
    pub const PPS_SRW: usize = 135;
    pub const VS_SRW: usize = 0;
    pub const APP_SRW: usize = 290;
    pub const DS_SECONDS_SRW: usize = 0;
    pub const DS_PIECES_SRW: usize = 700;
    pub const GARBAGE_EFFICIENCY_SRW: usize = 0;
}

#[derive(Debug)]
pub enum Errors {
    FailedToSendRequest(reqwest::Error),
    UserNotFound,
}

impl std::error::Error for Errors {}
impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:#?}", self))
    }
}

impl ProfileStats {
    pub async fn from_username<'b>(username: &'b str) -> Result<Self, Errors> {
        let client = ClientBuilder::new()
            .user_agent("tlns-tetrio-calcs")
            .build()
            .map_err(Errors::FailedToSendRequest)?;
        let user_api = API.to_string() + "users/" + username;
        let rank_summary = user_api.clone() + "/summaries/league";
        let user_info: serde_json::Value = client
            .get(user_api)
            .send()
            .await
            .map_err(Errors::FailedToSendRequest)?
            .error_for_status()
            .map_err(Errors::FailedToSendRequest)?
            .json::<serde_json::Value>()
            .await
            .map_err(Errors::FailedToSendRequest)?;
        let rank_info: serde_json::Value = client
            .get(rank_summary)
            .send()
            .await
            .map_err(Errors::FailedToSendRequest)?
            .error_for_status()
            .map_err(Errors::FailedToSendRequest)?
            .json::<serde_json::Value>()
            .await
            .map_err(Errors::FailedToSendRequest)?;
        if user_info["success"].as_bool().unwrap_or(false) == false {
            return Err(Errors::UserNotFound);
        }
        let user_info = user_info["data"].clone();
        let rank_info = rank_info["data"].clone();
        Ok(Self {
            apm: rank_info["apm"].as_f64().unwrap_or(0.0) as f32,
            pps: rank_info["pps"].as_f64().unwrap_or(0.0) as f32,
            vs: rank_info["vs"].as_f64().unwrap_or(0.0) as f32,
            rank: Some(
                Ranks::from_str(&rank_info["rank"].as_str().unwrap_or("Z").to_uppercase()).unwrap(),
            ),
            tr: Some(rank_info["tr"].as_f64().unwrap_or(0.0)),
            name: Some(user_info["username"].as_str().unwrap().to_string()),
            pfp: Some(
                "https://tetr.io/user-content/avatars/".to_string()
                    + user_info["_id"].as_str().unwrap()
                    + ".jpg?rv="
                    + user_info["avatar_revision"]
                        .as_u64()
                        .unwrap_or(0)
                        .to_string()
                        .as_str(),
            ),
            rd: Some(rank_info["rd"].as_f64().unwrap_or(0.0)),
            glicko: Some(rank_info["glicko"].as_f64().unwrap_or(0.0)),
            is_real: true,
        })
    }

    pub fn from_stat(apm: f32, pps: f32, vs: f32) -> Self {
        Self {
            apm,
            pps,
            vs,
            rank: None,
            tr: None,
            name: None,
            pfp: None,
            glicko: Some(60.0), // statPlayer = new Player("EXAMPLE", name[0], name[1], name[2], 0, 0, 60, null) but why?
            rd: None,
            is_real: false,
        }
    }

    #[inline(always)]
    pub fn app(&self) -> f64 {
        self.apm as f64 / (self.pps as f64 * 60.0)
    }

    #[inline(always)]
    pub fn ds_seconds(&self) -> f64 {
        (self.vs as f64 / 100.0) - (self.apm as f64 / 60.0)
    }

    #[inline(always)]
    pub fn ds_pieces(&self) -> f64 {
        (self.ds_seconds()) / self.pps as f64
    }

    #[inline(always)]
    pub fn app_ds_per_pieces(&self) -> f64 {
        self.ds_pieces() + self.app()
    }

    #[inline(always)]
    pub fn cheese_index(&self) -> f64 {
        (self.ds_pieces() * 150.0)
            + (((self.vs as f64 / self.apm as f64) - 2.0) * 50.0)
            + ((0.6 - self.app()) * 125.0)
    }

    #[inline(always)]
    pub fn garbage_efficiency(&self) -> f64 {
        ((self.app() * self.ds_seconds()) / self.pps as f64) * 2.0
    }

    #[inline(always)]
    pub fn area(&self) -> f64 {
        self.apm as f64
            + self.pps as f64 * weights::PPS_WEIGHT as f64
            + self.vs as f64 * weights::VS_WEIGHT as f64
            + self.app() * weights::APP_WEIGHT as f64
            + self.ds_seconds() * weights::DS_SECONDS_WEIGHT as f64
            + self.ds_pieces() * weights::DS_PIECES_WEIGHT as f64
            + self.garbage_efficiency() * weights::GARBAGE_EFFICIENCY_WEIGHT as f64
    }

    #[inline(always)]
    pub fn sr_area(&self) -> f64 {
        (self.apm as f64 * weights::APM_SRW as f64)
            + (self.pps as f64 * weights::PPS_SRW as f64)
            + (self.vs as f64 * weights::VS_SRW as f64)
            + (self.app() * weights::APP_SRW as f64)
            + (self.ds_seconds() * weights::DS_SECONDS_SRW as f64)
            + (self.ds_pieces() * weights::DS_PIECES_SRW as f64)
            + (self.garbage_efficiency() * weights::GARBAGE_EFFICIENCY_SRW as f64)
    }

    #[inline(always)]
    pub fn sr(&self) -> f64 {
        let x = (11.2 * ((self.sr_area() - 93.0) / 130.0).atan()) + 1.0;
        if x <= 0.0 {
            return 0.001;
        }
        x
    }

    #[inline(always)]
    pub fn weighted_app(&self) -> f64 {
        self.app()
            - 5.0 * (((self.cheese_index() / -30.0) + 1.0) * core::f64::consts::PI / 180.0).tan()
    }

    #[inline(always)]
    pub fn accuracy_tr(&self) -> f64 {
        match self.tr {
            Some(i) => self.estimated_tr() - i,
            None => 0.0,
        }
    }

    #[inline(always)]
    pub fn opener(&self) -> f64 {
        ((((self.apm as f64 / self.sr_area())
            / ((0.069 * 1.0017_f64.powf((self.sr().powi(5)) / 4700.0)) + self.sr() / 360.0)
            - 1.0)
            + (((self.pps as f64 / self.sr_area())
                / (0.0084264 * (2.14_f64.powf(-2.0 * (self.sr() / 2.7 + 1.03)))
                    - self.sr() / 5750.0
                    + 0.0067)
                - 1.0)
                * 0.75)
            + ((self.vs_apm() / (-(((self.sr() - 16.0) / 36.0).powi(2)) + 2.133) - 1.0) * -10.0)
            + ((self.app()
                / (0.1368803292 * 1.0024_f64.powf((self.sr().powi(5)) / 2800.0)
                    + self.sr() / 54.0)
                - 1.0)
                * 0.75)
            + ((self.ds_pieces()
                / (0.02136327583 * (14.0_f64.powf((self.sr() - 14.75) / 3.9))
                    + self.sr() / 152.0
                    + 0.022)
                - 1.0)
                * -0.25))
            / 3.5)
            + 0.5
    }

    #[inline(always)]
    pub fn plonk(&self) -> f64 {
        let x = (((self.garbage_efficiency()
            / (self.sr() / 350.0
                + 0.005948424455 * 3.8_f64.powf((self.sr() - 6.1) / 4.0)
                + 0.006)
            - 1.0)
            + (self.app()
                / (0.1368803292 * 1.0024_f64.powf((self.sr().powi(5)) / 2800.0)
                    + self.sr() / 54.0)
                - 1.0)
            + ((self.ds_pieces()
                / (0.02136327583 * (14.0_f64.powf((self.sr() - 14.75) / 3.9))
                    + self.sr() / 152.0
                    + 0.022)
                - 1.0)
                * 0.75)
            + (((self.pps as f64 / self.sr_area())
                / (0.0084264 * (2.14_f64.powf(-2.0 * (self.sr() / 2.7 + 1.03)))
                    - self.sr() / 5750.0
                    + 0.0067)
                - 1.0)
                * -1.0))
            / 2.73)
            + 0.5;
        truncate(x, 4)
    }

    #[inline(always)]
    pub fn stride(&self) -> f64 {
        let x = (((((self.apm as f64 / self.sr_area())
            / ((0.069 * 1.0017_f64.powf((self.sr().powi(5)) / 4700.0)) + self.sr() / 360.0)
            - 1.0)
            * -0.25)
            + ((self.pps as f64 / self.sr_area())
                / (0.0084264 * (2.14_f64.powf(-2.0 * (self.sr() / 2.7 + 1.03)))
                    - self.sr() / 5750.0
                    + 0.0067)
                - 1.0)
            + ((self.app()
                / (0.1368803292 * 1.0024_f64.powf((self.sr().powi(5)) / 2800.0)
                    + self.sr() / 54.0)
                - 1.0)
                * -2.0)
            + ((self.ds_pieces()
                / (0.02136327583 * (14.0_f64.powf((self.sr() - 14.75) / 3.9))
                    + self.sr() / 152.0
                    + 0.022)
                - 1.0)
                * -0.5))
            * 0.79)
            + 0.5;
        truncate(x, 4)
    }

    #[inline(always)]
    pub fn infinite_downstack(&self) -> f64 {
        let x = (((self.ds_pieces()
            / (0.02136327583 * (14.0_f64.powf((self.sr() - 14.75) / 3.9))
                + self.sr() / 152.0
                + 0.022)
            - 1.0)
            + ((self.app()
                / (0.1368803292 * 1.0024_f64.powf((self.sr().powi(5)) / 2800.0)
                    + self.sr() / 54.0)
                - 1.0)
                * -0.75)
            + (((self.apm as f64 / self.sr_area())
                / ((0.069 * 1.0017_f64.powf((self.sr().powi(5)) / 4700.0)) + self.sr() / 360.0)
                - 1.0)
                * 0.5)
            + ((self.vs_apm() / (-(((self.sr() - 16.0) / 36.0).powi(2)) + 2.133) - 1.0) * 1.5)
            + (((self.pps as f64 / self.sr_area())
                / (0.0084264 * (2.14_f64.powf(-2.0 * (self.sr() / 2.7 + 1.03)))
                    - self.sr() / 5750.0
                    + 0.0067)
                - 1.0)
                * 0.5))
            * 0.9)
            + 0.5;

        truncate(x, 4)
    }

    #[inline(always)]
    pub fn estimated_tr(&self) -> f64 {
        25000.0
            / (1.0
                + 10.0_f64.powf(
                    ((1500.0 - self.estimated_glicko()) * core::f64::consts::PI)
                        / ((((3.0 * 10.0_f64.ln().powi(2)) * 60.0_f64.powi(2))
                            + (2500.0
                                * ((64.0 * core::f64::consts::PI.powi(2))
                                    + (147.0 * 10.0_f64.ln().powi(2)))))
                        .sqrt()),
                ))
    }

    #[inline(always)]
    pub fn estimated_glicko(&self) -> f64 {
        0.000013
            * ((self.pps as f64 * (150.0 + ((self.vs_apm() - 1.66) * 35.0))
                + self.app() * 290.0
                + self.ds_pieces() * 700.0)
                .powi(3))
            - 0.0196
                * ((self.pps as f64 * (150.0 + ((self.vs_apm() - 1.66) * 35.0))
                    + self.app() * 290.0
                    + self.ds_pieces() * 700.0)
                    .powi(2))
            + (12.645
                * (self.pps as f64 * (150.0 + ((self.vs_apm() - 1.66) * 35.0))
                    + self.app() * 290.0
                    + self.ds_pieces() * 700.0))
            - 1005.4
    }

    #[inline(always)]
    pub fn vs_apm(&self) -> f64 {
        self.vs as f64 / self.apm as f64
    }
}

#[cfg(all(test, feature = "test"))]
mod tests {
    use once_cell::sync::OnceCell;

    use super::*;

    static DATA: once_cell::sync::OnceCell<ProfileStats> = OnceCell::with_value(ProfileStats {
        apm: 66.09,
        pps: 2.07,
        vs: 135.65,
        rank: Some(Ranks::U),
        tr: Some(23684.48),
        name: None,
        pfp: None,
        glicko: Some(2257.86),
        rd: Some(66.04),
        is_real: true,
    });

    #[test]
    fn test_ds_pieces() {
        let a = DATA.wait();
        assert_eq!(truncate(a.ds_pieces(), 4), 0.1232);
    }

    #[test]
    fn test_app() {
        let a = DATA.wait();
        assert_eq!(truncate(a.app(), 4), 0.5321);
    }

    #[test]
    fn test_app_ds_per_pieces() {
        let a = DATA.wait();
        assert_eq!(truncate(a.app_ds_per_pieces(), 4), 0.6553);
    }

    #[test]
    fn test_ds_seconds() {
        let a = DATA.wait();
        assert_eq!(truncate(a.ds_seconds(), 4), 0.2550)
    }

    #[test]
    fn test_vs_apm() {
        let a = DATA.wait();
        assert_eq!(truncate(a.vs_apm(), 4), 2.0525)
    }

    #[test]
    fn test_garbage_eff() {
        let a = DATA.wait();
        assert_eq!(truncate(a.garbage_efficiency(), 4), 0.1311)
    }

    #[test]
    fn test_cheese() {
        let a = DATA.wait();
        assert_eq!(truncate(a.cheese_index(), 4), 29.5878)
    }

    #[test]
    fn test_weighted_app() {
        let a = DATA.wait();
        assert_eq!(truncate(a.weighted_app(), 4), 0.5309)
    }

    #[test]
    fn test_area() {
        let a = DATA.wait();
        assert_eq!(truncate(a.area(), 4), 459.2692)
    }

    #[test]
    fn test_est_tr() {
        let a = DATA.wait();
        assert_eq!(truncate(a.estimated_tr(), 2), 23747.64)
    }

    #[test]
    fn test_est_glicko() {
        let a = DATA.wait();
        assert_eq!(
            truncate(a.estimated_glicko(), 4),
            truncate(2270.1577831407817, 4)
        )
    }

    #[test]
    fn test_sr_area() {
        let a = DATA.wait();
        assert_eq!(truncate(a.sr_area(), 4), truncate(519.998309178744, 4))
    }

    #[test]
    fn test_sr() {
        let a = DATA.wait();
        assert_eq!(truncate(a.sr(), 4), truncate(15.282925422083864, 4))
    }

    #[test]
    fn test_opener() {
        let a = DATA.wait();
        assert_eq!(truncate(a.opener(), 4), 0.5883)
    }

    #[test]
    fn test_plonk() {
        let a = DATA.wait();
        assert_eq!(truncate(a.plonk(), 4), 0.3368)
    }

    #[test]
    fn test_stride() {
        let a = DATA.wait();
        assert_eq!(truncate(a.stride(), 4), 0.6631)
    }

    #[test]
    fn test_inf_ds() {
        let a = DATA.wait();
        assert_eq!(truncate(a.infinite_downstack(), 4), 0.2741)
    }

    #[tokio::test]
    async fn test_fetch_pfp() {
        let e = ProfileStats::from_username("timelessnesses")
            .await
            .expect("Failed to fetch profile");
        eprintln!("{:#?}", e);
    }
}

pub fn truncate(b: f64, precision: usize) -> f64 {
    let factor = 10f64.powi(precision as i32);
    (b * factor).round() / factor
}
