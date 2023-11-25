mod app;
mod components;
mod soundbank_selector;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
