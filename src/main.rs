use anyhow::Result;
use mensa_upb_stats::{util, Canteen};
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let db = util::get_db()?;

    tracing_subscriber::fmt::init();

    sqlx::migrate!().run(&db).await?;

    tracing::info!("Starting to scrape menu");

    let canteens = Canteen::iter().collect::<Vec<_>>();
    util::async_for_each(&canteens, |(canteen, menu)| {
        let db = db.clone();
        async move {
            util::add_menu_to_db(&db, canteen, menu).await;
        }
    })
    .await;

    Ok(())
}
