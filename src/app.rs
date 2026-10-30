use embassy_executor::Spawner;

#[embassy_executor::task]
pub async fn app_task(spawner: Spawner) {}
