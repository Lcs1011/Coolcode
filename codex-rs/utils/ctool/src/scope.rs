use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CToolScope {
    None,
    Workspace,
    SelectedOnly,
    TheEyeofProvidence,
}

impl CToolScope {
    pub fn as_str(self) -> &'static str {
        match self {
            CToolScope::None => "None",
            CToolScope::Workspace => "Workspace",
            CToolScope::SelectedOnly => "SelectedOnly",
            CToolScope::TheEyeofProvidence => "TheEyeofProvidence",
        }
    }
}

impl Default for CToolScope {
    fn default() -> Self {
        CToolScope::None
    }
}

impl fmt::Display for CToolScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
