pub struct Admin;
pub struct User;
pub struct Any;

#[derive(Copy, Clone, Default, Debug)]
pub enum Role {
    #[default]
    None,
    User,
    Admin,
    Any,
}
