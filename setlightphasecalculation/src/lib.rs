use hyper::body;
use hyper::{Body, Request, Response, StatusCode};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::result::Result;
use tokio::time::sleep;
use tokio::time::Duration;
use validator::Validate;

// {
//     "temperature_celsius": 25.4,
//     "humidity_percent": 70.0,
//     "wind_kph": 100.0,
//     "rain": false
//     }

#[derive(Serialize, Deserialize)]
enum LightType {
    GREEN,
    YELLOW,
    RED,
}
#[derive(Serialize, Deserialize, PartialEq)]
enum EmergencyType {
    NONE,
    LUNATIC,
}

#[derive(Deserialize, Validate)]
struct Condition {
    road_condition: u8,
}

#[derive(Deserialize, Serialize, Validate)]
struct Plan {
    plate: String,
    direction: u8,
    speed: f32,
}

#[derive(Deserialize, Validate)]
struct Emergency {
    active: bool,
    emergency_type: EmergencyType,
}

#[derive(Deserialize, Serialize, Validate)]
struct Light {
    blink: bool,
    color: LightType,
}

#[derive(Deserialize, Validate)]
struct Incoming {
    condition: Option<Condition>,
    plans: Option<Vec<Plan>>,
    emergency: Option<Emergency>,
}

fn push_emergency(con: &mut redis::Connection) -> Result<(), Box<dyn Error>> {
    let light = Light {
        blink: true,
        color: LightType::YELLOW,
    };
    con.set("light", serde_json::to_string(&light).unwrap())?;

    Ok(())
}

fn initial_db_update(
    con: &mut redis::Connection,
    condition: &Option<Condition>,
    emergency: &Option<Emergency>,
    plans: &Option<Vec<Plan>>,
) -> Result<(), Box<dyn Error>> {
    if let Some(condition) = condition {
        con.set("condition", condition.road_condition)?;
    }

    if let Some(plans) = plans {
        con.set("plans", serde_json::to_string(&plans).unwrap())?;
    }

    if let Some(emergency) = emergency {
        if emergency.active {
            push_emergency(con)?;
        }

        con.set(
            "emergency",
            serde_json::to_string(&emergency.emergency_type).unwrap(),
        )?;
    }

    Ok(())
}

fn check_and_lock(con: &mut redis::Connection) -> Result<bool, Box<dyn Error>> {
    let lock: bool = match con.get("lock") {
        Ok(lock) => lock,
        Err(_) => false,
    };

    if lock {
        return Ok(true);
    }
    con.set("lock", false)?;

    Ok(false)
}

fn check_and_unlock(con: &mut redis::Connection) -> Result<bool, Box<dyn Error>> {
    let lock: bool = match con.get("lock") {
        Ok(lock) => lock,
        Err(_) => false,
    };

    if !lock {
        return Ok(true);
    }
    con.set("lock", false)?;

    Ok(false)
}

async fn wait_appropriately(con: &mut redis::Connection) {
    let condition: u8 = match con.get("condition") {
        Ok(condition) => condition,
        Err(_) => 5,
    };

    let wait_time: u64 = (condition as u64 / 2 + 2) * 1000;
    sleep(Duration::from_millis(wait_time)).await;
}

async fn change_light(con: &mut redis::Connection) -> Result<(), Box<dyn Error>> {
    let raw: String = con.get("emergency")?;
    let emergency: EmergencyType = serde_json::from_str(&raw.as_str()).unwrap();
    if emergency != EmergencyType::NONE {
        push_emergency(con)?;
    }

    // If no movement plan or cars just say lights are red blink (so pedestrians are happy)
    let raw_plans: String = con.get("plans")?;
    let plans: Vec<Plan> = serde_json::from_str(&raw_plans.as_str()).unwrap();
    if plans.is_empty() || plans.iter().position(|x| x.speed > 50.0).is_some() {
        con.set(
            "light",
            serde_json::to_string(&Light {
                blink: false,
                color: LightType::YELLOW,
            })
            .unwrap(),
        )?;

        wait_appropriately(con).await;

        con.set(
            "light",
            serde_json::to_string(&Light {
                blink: false,
                color: LightType::RED,
            })
            .unwrap(),
        )?;
    } else {
        con.set(
            "light",
            serde_json::to_string(&Light {
                blink: false,
                color: LightType::GREEN,
            })
            .unwrap(),
        )?;
    }

    let raw: String = con.get("emergency")?;
    let emergency: EmergencyType = serde_json::from_str(&raw.as_str()).unwrap();
    if emergency != EmergencyType::NONE {
        push_emergency(con)?;
    }

    Ok(())
}

pub async fn handle(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
    let bytes = body::to_bytes(req.into_body()).await?;
    let body: Incoming = serde_json::from_slice(&bytes)?;
    body.validate()?;

    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;
    initial_db_update(&mut con, &body.condition, &body.emergency, &body.plans)?;

    if let Ok(true) = check_and_lock(&mut con) {
        let ret = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("locked"))?;
        return Ok(ret);
    }

    change_light(&mut con).await?;

    if let Ok(true) = check_and_unlock(&mut con) {
        let ret = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("unlocked"))?;
        return Ok(ret);
    }

    let ret = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(vec![]))?;
    Ok(ret)
}
