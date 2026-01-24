#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Core,
    Workspace,
    Src,
    Project,
    Archive,
    Scratch,
}

impl Zone {
    pub fn short_label(&self) -> &'static str {
        match self {
            Zone::Core      => "CORE",
            Zone::Workspace => "WORK",
            Zone::Src       => "SRC",
            Zone::Project   => "PROJ",
            Zone::Archive   => "ARCH",
            Zone::Scratch   => "SCR",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Zone::Core      => "🔒",
            Zone::Workspace => "🦀",
            Zone::Src       => "🛠",
            Zone::Project   => "💼",
            Zone::Archive   => "💎",
            Zone::Scratch   => "🧪",
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, Zone::Core | Zone::Workspace)
    }
}
