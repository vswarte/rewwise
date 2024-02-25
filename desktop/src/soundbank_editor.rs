use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;
use crate::hirc_editor::HircEditor;

#[function_component(SoundbankEditor)]
pub fn soundbank_editor() -> Html {
    let navigator = use_navigator().unwrap();
    ensure_has_soundbank(navigator);

    html! {
    }
}

fn ensure_has_soundbank(navigator: Navigator) {
    let lock = crate::soundbank::PRIMARY_SOUNDBANK.get()
        .expect("Could not acquire soundbank oncelock")
        .read()
        .expect("Could not acquire read lock on soundbank");

    if lock.as_ref().is_none() {
        navigator.push(&Route::SoundbankSelector);
    }
}
