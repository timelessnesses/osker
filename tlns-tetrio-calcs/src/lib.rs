use std::str::FromStr;

use reqwest::ClientBuilder;

const API: &'static str = "https://ch.tetr.io/api/";

pub struct ProfileStats {
    pub apm: f32,
    pub pps: f32,
    pub vs: f32,
    pub rank: Ranks,
    pub tr: f32,
    pub name: String,
    pub pfp: String,
    pub glicko: f64,
    pub rd: f64,
}

macro_rules! enum_from_string {
    ($name:ident { $($variant:ident),* }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),*
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.replace("-", "Minus").replace("+", "Plus").as_str() {
                    $(stringify!($variant) => Ok($name::$variant),)*
                    _ => Err(()),
                }
            }
        }
    };
}

enum_from_string!(Ranks {
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
    D
});

mod weights {
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

impl ProfileStats {
    pub async fn from_username<'b>(username: &'b str) -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new()
            .user_agent("tlns-tetrio-calcs")
            .build()?;
        let user_api = API.to_string() + "users/" + username;
        let response: serde_json::Value = client
            .get(user_api)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?["data"]["user"]
            .clone();
        Ok(Self {
            apm: response["league"]["apm"].as_f64().unwrap() as f32,
            pps: response["league"]["pps"].as_f64().unwrap() as f32,
            vs: response["league"]["vs"].as_f64().unwrap() as f32,
            rank: Ranks::from_str(&response["league"]["rank"].as_str().unwrap().to_uppercase())
                .unwrap(),
            tr: response["league"]["tr"].as_f64().unwrap() as f32,
            name: response["username"].as_str().unwrap().to_string(),
            pfp: "https://tetr.io/user-content/avatars/".to_string()
                + response["_id"].as_str().unwrap()
                + ".jpg?rv="
                + response["avatar_revision"].as_str().unwrap(),
            rd: response["league"]["rd"].as_f64().unwrap(),
            glicko: response["league"]["glicko"].as_f64().unwrap(),
        })
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
        self.apm as f64 * weights::APM_WEIGHT as f64
            + self.pps as f64 * weights::PPS_WEIGHT as f64
            + self.vs as f64 * weights::VS_WEIGHT as f64
            + self.app() * weights::APP_WEIGHT as f64
            + self.ds_seconds() * weights::DS_SECONDS_WEIGHT as f64
            + self.ds_pieces() * weights::DS_SECONDS_WEIGHT as f64
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
            return 0.001
        }
        x
    }

    #[inline(always)]
    pub fn weighted_app(&self) -> f64 {
        self.app() - 5.0 * ((self.cheese_index() / -30.0) + 1.0 * core::f64::consts::PI / 180.0).tan()
    }

    #[inline(always)]
    pub fn accuracy_tr(&self) -> f64 {
        self.estimated_tr() - self.tr as f64
    }

    #[inline(always)]
    pub fn opener(&self) -> f64 {
        todo!()
    }

    #[inline(always)]
    pub fn plonk(&self) -> f64 {
        todo!()
    }

    #[inline(always)]
    pub fn stride(&self) -> f64 {
        todo!()
    }

    #[inline(always)]
    pub fn infinite_downstack(&self) -> f64 {
        todo!()
    }

    #[inline(always)]
    // Est. TR:
    pub fn estimated_tr(&self) -> f64 {
        25000.0
            / (1.0
                + (10.0 as f64).powf(
                    ((1500.0 - self.estimated_glicko()) * core::f64::consts::PI)
                        / (((3.0 * core::f64::consts::LOG10_E).powi(2) * 60.0).powi(2)).sqrt()
                        + (2500.0 * (64.0 * core::f64::consts::PI).powi(2)
                            + (147.0 * core::f64::consts::LOG10_E).powi(2)),
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
