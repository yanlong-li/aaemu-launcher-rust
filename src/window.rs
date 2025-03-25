use std::sync::atomic::Ordering;
use slint::ComponentHandle;
use crate::{business_logic, download, protocol, web_site, MainWindow, PAUSE_UPGRADE, WEBSITE_URL};
use tracing::{debug, info};

pub(crate) fn create() -> Result<MainWindow, Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;

    main_window.on_exit(|| {
        std::process::exit(0);
    });

    main_window.on_start_game(|| {
        info!("开始游戏");

        slint::spawn_local(async move {
            let auth_token = protocol::handle().await.unwrap();
            business_logic::handle_launch(&auth_token).await;
        })
        .expect("TODO: panic message");
    });
    main_window.on_open_website(|| {
        web_site::open_website(WEBSITE_URL).expect("TODO: panic message");
    });
    main_window.on_upgrade(|| {
        debug!("开始更新DB");
        slint::spawn_local(async move {
            debug!("开始更新DB2");

            download::start_download_db().await.unwrap();
        })
        .expect("TODO: panic message");
    });

    main_window.on_pause_upgrade(|v: bool| {
        slint::spawn_local(async move {
            if let Some(ac) = PAUSE_UPGRADE.get() {
                ac.store(v, Ordering::Relaxed);
            }
        })
            .expect("TODO: panic message");
    });

    main_window.show().unwrap();

    Ok(main_window)
}
