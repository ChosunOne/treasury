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
    pub const fn levels() -> [ReadLevel; 3] {
        [Self::ReadAll, Self::Read, Self::NoPermission]
    }
}

impl From<ReadLevel> for &str {
    fn from(value: ReadLevel) -> Self {
        match value {
            ReadLevel::NoPermission => "none",
            ReadLevel::Read => "read",
            ReadLevel::ReadAll => "read_all",
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
    pub const fn levels() -> [CreateLevel; 2] {
        [Self::Create, Self::NoPermission]
    }
}

impl From<CreateLevel> for &str {
    fn from(value: CreateLevel) -> Self {
        match value {
            CreateLevel::NoPermission => "none",
            CreateLevel::Create => "create",
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
    pub const fn levels() -> [UpdateLevel; 3] {
        [Self::UpdateAll, Self::Update, Self::NoPermission]
    }
}

impl From<UpdateLevel> for &str {
    fn from(value: UpdateLevel) -> Self {
        match value {
            UpdateLevel::NoPermission => "none",
            UpdateLevel::Update => "update",
            UpdateLevel::UpdateAll => "update_all",
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
    pub const fn levels() -> [DeleteLevel; 3] {
        [Self::DeleteAll, Self::Delete, Self::NoPermission]
    }
}

impl From<DeleteLevel> for &str {
    fn from(value: DeleteLevel) -> Self {
        match value {
            DeleteLevel::NoPermission => "none",
            DeleteLevel::Delete => "delete",
            DeleteLevel::DeleteAll => "delete_all",
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
