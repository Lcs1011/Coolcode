use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CToolBaseScope {
    None,
    Workspace,
    SelectedOnly,
    TheEyeofProvidence,
}

impl CToolBaseScope {
    pub fn as_str(self) -> &'static str {
        match self {
            CToolBaseScope::None => "None",
            CToolBaseScope::Workspace => "Workspace",
            CToolBaseScope::SelectedOnly => "SelectedOnly",
            CToolBaseScope::TheEyeofProvidence => "TheEyeofProvidence",
        }
    }
}

impl Default for CToolBaseScope {
    fn default() -> Self {
        CToolBaseScope::None
    }
}

impl fmt::Display for CToolBaseScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
