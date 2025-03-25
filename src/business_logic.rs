use crate::protocol::AuthToken;
use crate::{db_check, protocol, regedit, site_link_url, system_config, trion_1_2, MainWindow, MessageActions, State, VERSION};

pub async fn handle(window: &MainWindow) {
    let _ = site_link_url::handle().await;
    if !regedit::detecting() {
        if !regedit::register() {
            window.invoke_message(
                "注册表".into(),
                "写入注册表失败".into(),
                MessageActions::Exit,
            );
            return;
        }
    }

    let res = protocol::handle().await;

    if res.is_err() {
        window.invoke_message(
            "启动器".into(),
            "请通过官网启动".into(),
            MessageActions::Exit,
        );
        return;
    }
    let auth_token = res.unwrap();

    if !handle_version(auth_token.with_launcher_version).await {
        window.invoke_message(
            "启动器".into(),
            "当前本本过低，请安装最新版本".into(),
            MessageActions::Exit,
        );
        return;
    }

    if !handle_db_check(&auth_token).await {
        // window.invoke_message(
        //     "DB文件校验".into(),
        //     "数值不正确".into(),
        //     MessageActions::Exit,
        // );
        window.invoke_changeState(State::Upgrade);
        return;
    }
    handle_conf().await;

    window.invoke_changeState(State::Ready);
    // web_site::open_website(WEBSITE_URL);

    return;
}

pub async fn handle_conf() {
    let _ = system_config::update().await;
}

pub async fn handle_db_check(auth_token: &AuthToken) -> bool {
    if db_check::detect_db(auth_token.db_hash.as_ref()).is_err() {
        return false;
    }
    true
}

pub async fn handle_version(with_launcher_version: u16) -> bool {
    if with_launcher_version > VERSION {
        false
    } else {
        true
    }
}

pub async fn handle_launch(auth_token: &AuthToken) {
    trion_1_2::launch(auth_token).await;
}
