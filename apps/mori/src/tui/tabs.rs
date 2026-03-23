#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Plans,
    Agents,
    Git,
    Logs,
    Config,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Dashboard,
        Tab::Plans,
        Tab::Agents,
        Tab::Git,
        Tab::Logs,
        Tab::Config,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Plans => "Plans",
            Self::Agents => "Agents",
            Self::Git => "Git",
            Self::Logs => "Logs",
            Self::Config => "Config",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Dashboard => 0,
            Self::Plans => 1,
            Self::Agents => 2,
            Self::Git => 3,
            Self::Logs => 4,
            Self::Config => 5,
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        Self::ALL.get(idx).copied()
    }

    pub fn fkey_label(&self) -> &'static str {
        match self {
            Self::Dashboard => "F1",
            Self::Plans => "F2",
            Self::Agents => "F3",
            Self::Git => "F4",
            Self::Logs => "F5",
            Self::Config => "F6",
        }
    }
}
