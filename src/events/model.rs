
pub enum ModelEvent {
    Quit,
    SearchFor(String),
    UserListSelected(String),
    UserListDoubleClicked(String, String),
}