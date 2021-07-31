
use rocketmap_entities::{Gym, GymDetails, Pokemon, Pokestop, Quest, Raid, Request};

use mysql_async::{params, prelude::Queryable};

use chrono::Utc;

use log::error;

use crate::db::get_conn;

async fn update_gym(gym: &Gym) -> Result<(), ()> {
    let mut conn = get_conn().await?;
    conn.exec_drop(
        format!(
            "INSERT INTO gym (id, updated, first_seen_timestamp, lat, lon, name, url, last_modified_timestamp, enabled, team_id, guarding_pokemon_id, availble_slots, raid_end_timestamp, ex_raid_eligible, sponsor_id, ar_scan_eligible)
            VALUES (:id, UNIX_TIMESTAMP(), UNIX_TIMESTAMP(), :lat, :lon, :name, :url, :timestamp, :enabled, :team_id, :guard_id, :slots, :raid_end, :ex, :sponsor, :ar)
            ON DUPLICATE KEY UPDATE updated = UNIX_TIMESTAMP(), lat = :lat, lon = :lon, name = :name, url = :url,{}{} team_id = :team_id,{} availble_slots = :slots,{}{} sponsor_id = :sponsor, ar_scan_eligible = :ar;",
            gym.last_modified.map(|_| " last_modified_timestamp = :timestamp,").unwrap_or_default(),
            gym.enabled.map(|_| " enabled = :enabled,").unwrap_or_default(),
            gym.guard_pokemon_id.map(|_| " guarding_pokemon_id = :guard_id,").unwrap_or_default(),
            gym.raid_active_until.map(|_| " raid_end_timestamp = :raid_end,").unwrap_or_default(),
            gym.ex_raid_eligible.map(|_| " ex_raid_eligible = :ex,").unwrap_or_default(),
        ),
        params! {
            "id" => gym.gym_id.as_str(),
            "lat" => gym.latitude,
            "lon" => gym.longitude,
            "name" => gym.gym_name.as_str(),
            "url" => gym.url.as_str(),
            "timestamp" => gym.last_modified,
            "enabled" => gym.enabled,
            "team_id" => gym.team_id.get_id(),
            "guard_id" => gym.guard_pokemon_id,
            "slots" => gym.slots_available,
            "raid_end" => gym.raid_active_until,
            "ex" => gym.ex_raid_eligible,
            "sponsor" => gym.sponsor_od,
            "ar" => gym.ar_scan_eligible,
        })
        .await
        .map_err(|e| error!("Mysql update gym error: {}\n{:?}", e, gym))?;
    Ok(())
}

async fn update_gym_details(gym: &GymDetails) -> Result<(), ()> {
    let mut conn = get_conn().await?;
    conn.exec_drop(
        "INSERT INTO gym (id, updated, first_seen_timestamp, lat, lon, name, url, team_id, availble_slots, ex_raid_eligible, in_battle, sponsor_id, ar_scan_eligible)
        VALUES (:id, UNIX_TIMESTAMP(), UNIX_TIMESTAMP(), :lat, :lon, :name, :url, :team_id, :slots, :ex, :in_battle, :sponsor, :ar)
        ON DUPLICATE KEY UPDATE updated = UNIX_TIMESTAMP(), lat = :lat, lon = :lon, name = :name, url = :url, team_id = :team_id, availble_slots = :slots, ex_raid_eligible = :ex, in_battle = :in_battle, sponsor_id = :sponsor, ar_scan_eligible = :ar;",
        params! {
            "id" => gym.id.as_str(),
            "lat" => gym.latitude,
            "lon" => gym.longitude,
            "name" => gym.name.as_str(),
            "url" => gym.url.as_str(),
            "team_id" => gym.team.get_id(),
            "slots" => gym.slots_available,
            "ex" => gym.ex_raid_eligible,
            "in_battle" => gym.in_battle,
            "sponsor" => gym.sponsor_od,
            "ar" => gym.ar_scan_eligible,
        })
        .await
        .map_err(|e| error!("Mysql update gym_details error: {}\n{:?}", e, gym))?;
    Ok(())
}

async fn update_pokestop(pokestop: &Pokestop) -> Result<(), ()> {
    let mut conn = get_conn().await?;
    conn.exec_drop(
        format!(
            "INSERT INTO pokestop (id, first_seen_timestamp, lat, lon, name, url, enabled, last_modified_timestamp, lure_expire_timestamp, pokestop_display, incident_expire_timestamp, updated, lure_id, grunt_type, ar_scan_eligible)
            VALUES (:id, UNIX_TIMESTAMP(), :lat, :lon, :name, :url, :enabled, :last, :lure_exp, :display, :incident_exp, :updated, :lure_id, :grunt_type, :ar)
            ON DUPLICATE KEY UPDATE lat = :lat, lon = :lon,{}{}{} last_modified_timestamp = :last, lure_expire_timestamp = :lure_exp,{} incident_expire_timestamp = :incident_exp, updated = :updated, lure_id = :lure_id, grunt_type = :grunt_type, ar_scan_eligible = :ar;",
            pokestop.name.as_ref().map(|_| " name = :name,").unwrap_or_default(),
            pokestop.url.as_ref().map(|_| " url = :url,").unwrap_or_default(),
            pokestop.enabled.map(|_| " enabled = :enabled,").unwrap_or_default(),
            pokestop.pokestop_display.map(|_| " pokestop_display = :display,").unwrap_or_default(),
        ),
        params! {
            "id" => pokestop.pokestop_id.as_str(),
            "lat" => pokestop.latitude,
            "lon" => pokestop.longitude,
            "name" => pokestop.name.as_deref(),
            "url" => pokestop.url.as_deref(),
            "enabled" => pokestop.enabled,
            "last" => pokestop.last_modified,
            "lure_exp" => pokestop.lure_expiration,
            "display" => pokestop.pokestop_display,
            "incident_exp" => pokestop.incident_expire_timestamp,
            "updated" => pokestop.updated,
            "lure_id" => pokestop.lure_id,
            "grunt_type" => pokestop.grunt_type,
            "ar" => pokestop.ar_scan_eligible,
        })
        .await
        .map_err(|e| error!("Mysql update pokestop error: {}\n{:?}", e, pokestop))?;
    Ok(())
}

async fn update_pokemon(pokemon: &Pokemon) -> Result<(), ()> {
    let mut conn = get_conn().await?;
    conn.exec_drop(
        format!(
            "INSERT INTO pokemon (id, pokemon_id, pokestop_id, lat, lon, expire_timestamp, expire_timestamp_verified, updated, first_seen_timestamp, gender, cp, form, costume, atk_iv, def_iv, sta_iv, move_1, move_2, weight, size, capture_1, capture_2, capture_3, weather, level, cell_id, username, shiny, display_pokemon_id, is_event, pvp_rankings_great_league, pvp_rankings_ultra_league)
            VALUES (:id, :pokemon, :pokestop, :lat, :lon, :exp, :exp_ver, :update, :first_seen, :gender, :cp, :form, :costume, :atk_iv, :def_iv, :sta_iv, :move1, :move2, :weight, :size, :capture1, :capture2, :capture3, :weather, :level, :cell, :username, :shiny, :display, :event, :great, :ultra)
            ON DUPLICATE KEY UPDATE pokemon_id = :pokemon, pokestop_id = :pokestop, lat = :lat, lon = :lon, expire_timestamp = :exp, expire_timestamp_verified = :exp_ver,{}{} gender = :gender, cp = :cp, form = :form, costume = :costume, atk_iv = :atk_iv, def_iv = :def_iv, sta_iv = :sta_iv, move_1 = :move1, move_2 = :move2, weight = :weight, size = :size, capture_1 = :capture1, capture_2 = :capture2, capture_3 = :capture3, weather = :weather, level = :level, cell_id = :cell, username = :username, shiny = :shiny, display_pokemon_id = :display, is_event = :event, pvp_rankings_great_league = :great, pvp_rankings_ultra_league = :ultra;",
            pokemon.last_modified_time.map(|_| " updated = :update,").unwrap_or_default(),
            pokemon.first_seen.map(|_| " first_seen_timestamp = :first_seen,").unwrap_or_default(),
        ),
        params! {
            "id" => pokemon.encounter_id.as_str(),
            "pokemon" => pokemon.pokemon_id,
            "pokestop" => pokemon.pokestop_id.as_ref().and_then(|id| if id == "None" { None } else { Some(id) }),
            // "spawn" => pokemon.spawnpoint_id,
            "lat" => pokemon.latitude,
            "lon" => pokemon.longitude,
            "exp" => pokemon.disappear_time,
            "exp_ver" => pokemon.disappear_time_verified,
            "update" => pokemon.last_modified_time,
            "first_seen" => pokemon.first_seen.unwrap_or_else(|| Utc::now().timestamp()),
            "gender" => pokemon.gender.get_id(),
            "cp" => pokemon.cp,
            "form" => pokemon.form,
            "costume" => pokemon.costume,
            "atk_iv" => pokemon.individual_attack,
            "def_iv" => pokemon.individual_defense,
            "sta_iv" => pokemon.individual_stamina,
            "move1" => pokemon.move_1,
            "move2" => pokemon.move_2,
            "weight" => pokemon.weight,
            "size" => pokemon.height,
            "capture1" => pokemon.capture_1,
            "capture2" => pokemon.capture_2,
            "capture3" => pokemon.capture_3,
            "weather" => pokemon.weather,
            "level" => pokemon.pokemon_level,
            "cell" => pokemon.s2_cell_id,
            "username" => pokemon.username.as_ref(),
            "shiny" => pokemon.shiny,
            "display" => pokemon.display_pokemon_id,
            "event" => pokemon.is_event.unwrap_or_default(),
            "great" => pokemon.pvp_rankings_great_league.as_ref().and_then(|pvp| serde_json::to_string(pvp).ok()),
            "ultra" => pokemon.pvp_rankings_ultra_league.as_ref().and_then(|pvp| serde_json::to_string(pvp).ok()),
        })
        .await
        .map_err(|e| error!("Mysql update pokemon error: {}\n{:?}", e, pokemon))?;
    Ok(())
}

async fn update_quest(quest: &Quest) -> Result<(), ()> {
    let mut conn = get_conn().await?;
    conn.exec_drop(
        "INSERT INTO pokestop (id, first_seen_timestamp, lat, lon, name, url, quest_type, quest_target, quest_template, quest_rewards, updated, quest_conditions, quest_timestamp, ar_scan_eligible)
        VALUES (:id, UNIX_TIMESTAMP(), :lat, :lon, :name, :url, :type, :target, :template, :rewards, :updated, :conditions, UNIX_TIMESTAMP(), :ar)
        ON DUPLICATE KEY UPDATE lat = :lat, lon = :lon, name = :name, url = :url, quest_type = :type, quest_target = :target, quest_template = :template, quest_rewards = :rewards, updated = :updated, quest_conditions = :conditions, quest_timestamp = UNIX_TIMESTAMP(), ar_scan_eligible = :ar;",
        params! {
            "id" => quest.pokestop_id.as_str(),
            "lat" => quest.latitude,
            "lon" => quest.longitude,
            "name" => quest.pokestop_name.as_str(),
            "url" => quest.pokestop_url.as_str(),
            "type" => quest.quest_type,
            "target" => quest.target,
            "template" => quest.template.as_str(),
            "rewards" => serde_json::to_string(&quest.rewards).ok(),
            "updated" => quest.updated,
            "conditions" => serde_json::to_string(&quest.conditions).ok(),
            "ar" => quest.ar_scan_eligible,
        })
        .await
        .map_err(|e| error!("Mysql update quest error: {}\n{:?}", e, quest))?;
    Ok(())
}

async fn update_raid(raid: &Raid) -> Result<(), ()> {
    let mut conn = get_conn().await?;
    conn.exec_drop(
        "INSERT INTO gym (id, updated, first_seen_timestamp, lat, lon, name, url, team_id, raid_spawn_timestamp, raid_battle_timestamp, raid_end_timestamp, raid_level, raid_pokemon_id, raid_pokemon_cp, raid_pokemon_move_1, raid_pokemon_move_2, ex_raid_eligible, raid_pokemon_form, raid_is_exclusive, raid_pokemon_gender, sponsor_id, raid_pokemon_evolution, ar_scan_eligible)
        VALUES (:id, UNIX_TIMESTAMP(), UNIX_TIMESTAMP(), :lat, :lon, :name, :url, :team, :spawn, :start, :end, :level, :pokemon, :cp, :move1, :move2, :ex, :form, :is_ex, :gender, :sponsor, :evo, :ar)
        ON DUPLICATE KEY UPDATE updated = UNIX_TIMESTAMP(), lat = :lat, lon = :lon, name = :name, url = :url, team_id = :team, raid_spawn_timestamp = :spawn, raid_battle_timestamp = :start, raid_end_timestamp = :end, raid_level = :level, raid_pokemon_id = :pokemon, raid_pokemon_cp = :cp, raid_pokemon_move_1 = :move1, raid_pokemon_move_2 = :move2, ex_raid_eligible = :ex, raid_pokemon_form = :form, raid_is_exclusive = :is_ex, raid_pokemon_gender = :gender, sponsor_id = :sponsor, raid_pokemon_evolution = :evo, ar_scan_eligible = :ar;",
        params! {
            "id" => raid.gym_id.as_str(),
            "lat" => raid.latitude,
            "lon" => raid.longitude,
            "name" => raid.gym_name.as_str(),
            "url" => raid.gym_url.as_str(),
            "team" => raid.team_id.get_id(),
            "spawn" => raid.spawn,
            "start" => raid.start,
            "end" => raid.end,
            "level" => raid.level,
            "pokemon" => raid.pokemon_id,
            "cp" => raid.cp,
            "move1" => raid.move_1,
            "move2" => raid.move_2,
            "ex" => raid.ex_raid_eligible,
            "form" => raid.form,
            "is_ex" => raid.is_exclusive,
            "gender" => raid.gender.as_ref().map(|g| g.get_id()),
            "sponsor" => raid.sponsor_od,
            "evo" => raid.evolution,
            "ar" => raid.ar_scan_eligible,
        })
        .await
        .map_err(|e| error!("Mysql update raid error: {}\n{:?}", e, raid))?;
    Ok(())
}

pub async fn submit<T: Iterator<Item=Request>>(iter: T) {
    for request in iter {
        tokio::spawn(async move {
            match request {
                Request::Gym(g) => { update_gym(&g).await.ok(); },
                Request::GymDetails(g) => { update_gym_details(&g).await.ok(); },
                Request::Invasion(i) => { update_pokestop(&i).await.ok(); },
                Request::Pokestop(p) => { update_pokestop(&p).await.ok(); },
                Request::Pokemon(p) => { update_pokemon(&p).await.ok(); },
                Request::Quest(q) => { update_quest(&q).await.ok(); },
                Request::Raid(r) => { update_raid(&r).await.ok(); },
                _ => {},
            }
        });
    }
}
