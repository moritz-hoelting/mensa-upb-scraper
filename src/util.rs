use std::{env, future::Future};

use anyhow::Result;
use chrono::Utc;
use futures::StreamExt as _;
use num_bigint::BigInt;
use sqlx::{postgres::PgPoolOptions, types::BigDecimal, PgPool};

use crate::{menu::scrape_menu, Canteen, Dish};

pub async fn async_for_each<F, Fut>(canteens: &[Canteen], f: F)
where
    F: FnMut((Canteen, Vec<Dish>)) -> Fut,
    Fut: Future<Output = ()>,
{
    futures::stream::iter(canteens)
        .then(|canteen| async move { (*canteen, scrape_menu(*canteen).await) })
        .filter_map(|(canteen, menu)| async move { menu.ok().map(|menu| (canteen, menu)) })
        .for_each(f)
        .await;
}

pub fn get_db() -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .connect_lazy(&env::var("DATABASE_URL").expect("missing DATABASE_URL env variable"))?)
}

pub async fn add_meal_to_db(db: &PgPool, canteen: Canteen, dish: &Dish) -> Result<()> {
    let today = Utc::now().date_naive();

    let vegan = dish.is_vegan();

    sqlx::query!(
        "INSERT INTO meals (date,canteen,name,dish_type,image_src,price_students,price_employees,price_guests,vegan,vegetarian)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        today, canteen.get_identifier(), dish.get_name(), 
        dish.get_type().to_string(), dish.get_image_src(),
        price_to_bigdecimal(dish.get_price_students()),
        price_to_bigdecimal(dish.get_price_employees()),
        price_to_bigdecimal(dish.get_price_guests()),
        vegan, vegan || dish.is_vegetarian()
    ).execute(db).await?;

    Ok(())
}

pub async fn add_menu_to_db(db: &PgPool, canteen: Canteen, menu: Vec<Dish>) {
    futures::stream::iter(menu)
        .for_each(|dish| async move {
            if !dish.get_name().is_empty() {
                add_meal_to_db(db, canteen, &dish).await.ok();
            }
        })
        .await;
}

pub fn price_to_bigdecimal(s: Option<&str>) -> BigDecimal {
    s.and_then(|p| p.trim_end_matches(" €").replace(',', ".").parse().ok())
        .unwrap_or_else(|| BigDecimal::new(BigInt::from(99999), 2))
}
