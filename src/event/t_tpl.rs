use crate::event::EventHandler;
use log::*;
use neovim_lib::Value;
use std::slice::Iter;

pub trait TTpl {
    fn event_tpl(&mut self, values: Iter<Value>) -> Result<(), ()>;
}

impl TTpl for EventHandler {
    fn event_tpl(&mut self, mut values: Iter<Value>) -> Result<(), ()> {
        let serialized_value: Value;
        match values.next() {
            Some(s) => serialized_value = s.clone(),
            None => {
                error!("serialized_value None");
                return Err(());
            },
        }
        let serialized: String;
        match serialized_value.as_str() {
            Some(ok) => serialized = ok.to_string(),
            None => {
                error!("serialized: String conversion None");
                return Err(());
            },
        }
        self.cobol.write_template_to_cbl(serialized)?;
        Ok(())
    }
}
