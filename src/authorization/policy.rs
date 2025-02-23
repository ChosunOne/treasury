use std::marker::PhantomData;

pub struct Policy<Resource, ActionSet, Role> {
    resource: PhantomData<Resource>,
    action: PhantomData<ActionSet>,
    role: PhantomData<Role>,
}
