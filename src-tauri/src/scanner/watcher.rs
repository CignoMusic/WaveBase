use crate::error::AppError;

pub struct FileWatcher;

impl FileWatcher {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self)
    }

    pub fn watch(&self, _path: &std::path::Path) -> Result<(), AppError> {
        Ok(())
    }
}
