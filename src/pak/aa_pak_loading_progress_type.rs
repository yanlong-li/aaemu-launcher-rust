use crate::pak::aa_pak::AAPak;


// 定义 AAPakLoadingProgressType 枚举
#[derive(Debug, Clone, Copy)]
pub enum AAPakLoadingProgressType {
    OpeningFile,
    ReadingHeader,
    WritingHeader,
    ReadingFAT,
    WritingFAT,
    ClosingFile,
    GeneratingDirectories,
}


// 定义 AAPakNotify trait 作为事件处理
pub trait AAPakNotify {
    fn on_progress(&self, sender: &AAPak, progress_type: AAPakLoadingProgressType, step: i32, maximum: i32);
}

// 实现示例：为某个结构体实现 AAPakNotify trait
// pub struct ProgressHandler;
//
// impl AAPakNotify for ProgressHandler {
//     fn on_progress(&self, sender: &AAPak, progress_type: AAPakLoadingProgressType, step: i32, maximum: i32) {
//         // 实现具体的事件处理逻辑
//         println!(
//             "Sender: {:?}, ProgressType: {:?}, Step: {}, Maximum: {}",
//             sender, progress_type, step, maximum
//         );
//     }
// }