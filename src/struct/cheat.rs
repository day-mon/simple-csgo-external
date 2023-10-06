use crate::r#struct::context::PlayerEntity;

pub struct GameCheat {
    is_enabled: bool,
    name: String,
    description: String,
}

pub trait Cheat {
    fn toggle(local_player: &PlayerEntity, entities: &[PlayerEntity]) ;
}
