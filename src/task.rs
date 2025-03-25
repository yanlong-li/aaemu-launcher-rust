use crate::{MainWindow, State, Task};
use tokio::sync::mpsc::Receiver;
use tracing::{debug, warn};

pub async fn task(rx: &mut Receiver<Task>, app: MainWindow) {
    println!("等待接收任务");
    loop {
        match rx.recv().await {
            None => {
                warn!("想终止任务？");
            }
            Some(msg) => match msg {
                Task::Progress(percentage) => {
                    debug!("升级进度:{}", percentage);
                    app.invoke_changeProgres(percentage as f32);
                    if percentage >= 100f64 {
                        app.invoke_changeState(State::Ready);
                    }
                    debug!("进度条更新完成");
                }
                Task::Message(title, content, action) => {
                    debug!("发送消息通知");
                    app.invoke_message(title.into(), content.into(), action);
                }
            },
        }
    }
}
