use std::{collections::HashMap, sync::Arc};

use crate::{Command, ZenisClient};

pub struct Framework {
    pub client: Arc<ZenisClient>,
    pub commands: HashMap<String, Box<(dyn Command + Send + Sync)>>,
}
