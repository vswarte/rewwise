mod app;
mod soundbank;
mod components;
mod soundbank_editor;
mod soundbank_selector;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
