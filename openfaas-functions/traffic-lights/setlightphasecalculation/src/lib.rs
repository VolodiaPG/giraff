use std::error::Error;

use redis::Commands;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio::time::Duration;
use validator::Validate;
use warp::{
    filters::BoxedFilter,
    http::{Response, StatusCode},
    Filter, Reply,
};

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
enum Direction {
    NORTH,
    SOUTH,
    EAST,
    WEST,
}

#[derive(Deserialize, Serialize, Validate)]
struct Plan {
    plate: String,
    direction: Direction,
    speed: i16,
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

/// if the lock is set, then return true, otherwise set it to true and return false
fn check_and_lock(con: &mut redis::Connection) -> Result<bool, Box<dyn Error>> {
    if Ok(true) == con.get("lock") {
        return Ok(true);
    }

    // Whatever the results, we still try to set the lock
    con.set("lock", true)?;

    Ok(false)
}

/// if the lock is set, then return false, otherwise set it to false and return false
fn check_and_unlock(con: &mut redis::Connection) -> Result<bool, Box<dyn Error>> {
    if Ok(false) == con.get("lock") {
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

fn get_emergency(con: &mut redis::Connection) -> Result<EmergencyType, Box<dyn Error>> {
    let raw: String = con.get("emergency")?;
    let emergency: EmergencyType  = serde_json::from_str(&raw.as_str())?;

    Ok(emergency)
}

async fn change_light(con: &mut redis::Connection) -> Result<(), Box<dyn Error>> {
    let emergency = get_emergency(con).unwrap_or(EmergencyType::NONE);
    if emergency != EmergencyType::NONE {
        push_emergency(con)?;
    }

    // If no movement plan or cars just say lights are red blink (so pedestrians are happy)
    let raw_plans: String = con.get("plans").unwrap_or("[]".to_string());
    let plans: Vec<Plan> = serde_json::from_str(&raw_plans.as_str()).unwrap();
    if plans.is_empty() || plans.iter().position(|x| x.speed > 50).is_some() {
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

    let emergency = get_emergency(con).unwrap_or(EmergencyType::NONE);
    if emergency != EmergencyType::NONE {
        push_emergency(con)?;
    }

    Ok(())
}

async fn handle(body: Incoming) -> Result<Box<dyn Reply>, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;
    initial_db_update(&mut con, &body.condition, &body.emergency, &body.plans)?;

    if let Ok(true) = check_and_lock(&mut con) {
        return Ok(Box::new(StatusCode::OK));
    }

    change_light(&mut con).await?;

    if let Ok(true) = check_and_unlock(&mut con) {
        return Ok(Box::new(StatusCode::OK));
    }

    Ok(Box::new(StatusCode::OK))
}

async fn reply(body: Incoming) -> Result<Box<dyn Reply>, warp::Rejection> {
    match handle(body).await {
        Ok(reply) => Ok(reply),
        Err(e) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(e.to_string()),
        )),
    }
}

fn json_body() -> impl Filter<Extract = (Incoming,), Error = warp::Rejection> + Clone {
    warp::body::json()
}

pub fn main() -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::post()
        .and(json_body())
        // .and_then(handle)
        .and_then(reply)
        .boxed()
}
