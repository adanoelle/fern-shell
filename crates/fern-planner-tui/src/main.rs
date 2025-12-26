//! Fern Planner TUI - A minimalist planner with goals, intentions, and calendar.

mod app;
mod data;
mod model;
mod msg;
mod view;

use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    frond::run::<app::PlannerApp>().await.map_err(|e| miette::miette!("{}", e))
}
