use std::marker::PhantomData;

pub struct NoPermission;
pub struct Read;
pub struct ReadAll;
pub struct Create;
pub struct Update;
pub struct UpdateAll;
pub struct Delete;
pub struct DeleteAll;

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
