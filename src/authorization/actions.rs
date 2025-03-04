use std::marker::PhantomData;

pub struct NoPermission;
pub struct Read;
pub struct ReadAll;
pub struct Create;
pub struct Update;
pub struct UpdateAll;
pub struct Delete;
pub struct DeleteAll;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadLevel {
    ReadAll,
    Read,
    #[default]
    NoPermission,
}

impl ReadLevel {
    pub fn levels() -> Vec<Self> {
        let mut l = vec![Self::ReadAll, Self::Read, Self::NoPermission];
        l.sort();
        l
    }
}

impl From<ReadLevel> for &str {
    fn from(value: ReadLevel) -> Self {
        match value {
            ReadLevel::ReadAll => "read_all",
            ReadLevel::Read => "read",
            ReadLevel::NoPermission => "none",
        }
    }
}

impl From<&str> for ReadLevel {
    fn from(value: &str) -> Self {
        match value {
            "read" => ReadLevel::Read,
            "read_all" => ReadLevel::ReadAll,
            _ => ReadLevel::default(),
        }
    }
}

impl From<String> for ReadLevel {
    fn from(value: String) -> Self {
        match value {
            val if val == *"read" => ReadLevel::Read,
            val if val == *"read_all" => ReadLevel::ReadAll,
            _ => ReadLevel::default(),
        }
    }
}

impl From<&String> for ReadLevel {
    fn from(value: &String) -> Self {
        match value {
            val if val == "read" => ReadLevel::Read,
            val if val == "read_all" => ReadLevel::ReadAll,
            _ => ReadLevel::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreateLevel {
    Create,
    #[default]
    NoPermission,
}

impl CreateLevel {
    pub fn levels() -> Vec<Self> {
        let mut l = vec![Self::Create, Self::NoPermission];
        l.sort();
        l
    }
}

impl From<CreateLevel> for &str {
    fn from(value: CreateLevel) -> Self {
        match value {
            CreateLevel::Create => "create",
            CreateLevel::NoPermission => "none",
        }
    }
}

impl From<&str> for CreateLevel {
    fn from(value: &str) -> Self {
        match value {
            "create" => CreateLevel::Create,
            _ => CreateLevel::default(),
        }
    }
}

impl From<String> for CreateLevel {
    fn from(value: String) -> Self {
        match value {
            val if val == *"create" => CreateLevel::Create,
            _ => CreateLevel::default(),
        }
    }
}

impl From<&String> for CreateLevel {
    fn from(value: &String) -> Self {
        match value {
            val if val == "create" => CreateLevel::Create,
            _ => CreateLevel::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdateLevel {
    UpdateAll,
    Update,
    #[default]
    NoPermission,
}

impl UpdateLevel {
    pub fn levels() -> Vec<Self> {
        let mut l = vec![Self::UpdateAll, Self::Update, Self::NoPermission];
        l.sort();
        l
    }
}

impl From<UpdateLevel> for &str {
    fn from(value: UpdateLevel) -> Self {
        match value {
            UpdateLevel::UpdateAll => "update_all",
            UpdateLevel::Update => "update",
            UpdateLevel::NoPermission => "none",
        }
    }
}

impl From<&str> for UpdateLevel {
    fn from(value: &str) -> Self {
        match value {
            "update" => UpdateLevel::Update,
            "update_all" => UpdateLevel::UpdateAll,
            _ => UpdateLevel::default(),
        }
    }
}

impl From<String> for UpdateLevel {
    fn from(value: String) -> Self {
        match value {
            val if val == *"update" => UpdateLevel::Update,
            val if val == *"update_all" => UpdateLevel::UpdateAll,
            _ => UpdateLevel::default(),
        }
    }
}

impl From<&String> for UpdateLevel {
    fn from(value: &String) -> Self {
        match value {
            val if val == "update" => UpdateLevel::Update,
            val if val == "update_all" => UpdateLevel::UpdateAll,
            _ => UpdateLevel::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeleteLevel {
    DeleteAll,
    Delete,
    #[default]
    NoPermission,
}

impl DeleteLevel {
    pub fn levels() -> Vec<Self> {
        let mut l = vec![Self::DeleteAll, Self::Delete, Self::NoPermission];
        l.sort();
        l
    }
}

impl From<DeleteLevel> for &str {
    fn from(value: DeleteLevel) -> Self {
        match value {
            DeleteLevel::DeleteAll => "delete_all",
            DeleteLevel::Delete => "delete",
            DeleteLevel::NoPermission => "none",
        }
    }
}

impl From<&str> for DeleteLevel {
    fn from(value: &str) -> Self {
        match value {
            "delete" => DeleteLevel::Delete,
            "delete_all" => DeleteLevel::DeleteAll,
            _ => DeleteLevel::default(),
        }
    }
}

impl From<String> for DeleteLevel {
    fn from(value: String) -> Self {
        match value {
            val if val == *"delete" => DeleteLevel::Delete,
            val if val == *"delete_all" => DeleteLevel::DeleteAll,
            _ => DeleteLevel::default(),
        }
    }
}

impl From<&String> for DeleteLevel {
    fn from(value: &String) -> Self {
        match value {
            val if val == "delete" => DeleteLevel::Delete,
            val if val == "delete_all" => DeleteLevel::DeleteAll,
            _ => DeleteLevel::default(),
        }
    }
}

pub struct ActionSet<
    Read = NoPermission,
    Create = NoPermission,
    Update = NoPermission,
    Delete = NoPermission,
> {
    read: PhantomData<Read>,
    create: PhantomData<Create>,
    update: PhantomData<Update>,
    delete: PhantomData<Delete>,
}
