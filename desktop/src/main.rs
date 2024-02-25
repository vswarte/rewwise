mod app;
mod settings;
mod soundbank;
mod components;
mod hirc_editor;
mod soundbank_editor;
mod soundbank_selector;

use app::App;

fn main() {
    crate::soundbank::init();

    yew::Renderer::<App>::new().render();
}
