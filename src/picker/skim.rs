use crate::error;
use crate::models;
use crate::picker;
use error::PickerError;
use models::ResultWithDefaultError;
use picker::{ItemPicker, PickableItem, PickableItemKey};
use skim::prelude::*;

pub struct SkimPicker;

fn get_skim_configuration() -> SkimOptions {
    SkimOptionsBuilder::default()
        // Set viewport to take entire screen
        .height(String::from("100%"))
        // Disable multiselect
        .multi(false)
        .build()
        .unwrap()
}

impl SkimItem for PickableItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::from(self.formatted.as_str())
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::from(self.key.to_string())
    }
}

fn generic_picker_error() -> Box<dyn std::error::Error + Send> {
    Box::new(PickerError::Generic)
}

impl ItemPicker for SkimPicker {
    fn pick(&self, items: Vec<PickableItem>) -> ResultWithDefaultError<PickableItemKey> {
        // `run_items` feeds the items to skim itself; skim owns the channel and
        // the batching, so the picker only has to describe the viewport.
        let output =
            Skim::run_items(get_skim_configuration(), items).map_err(|_| generic_picker_error())?;

        if output.is_abort {
            return Err(Box::new(PickerError::Cancelled));
        }

        output
            .selected_items
            .first()
            .and_then(|selected| selected.output().parse::<PickableItemKey>().ok())
            .ok_or_else(generic_picker_error)
    }
}
