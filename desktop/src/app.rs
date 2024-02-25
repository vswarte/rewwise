use yew::prelude::*;
use yew_router::prelude::*;
use crate::hirc_editor::HircEditor;
use crate::soundbank_selector::SoundbankSelector;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main class="dark:bg-slate-900">
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </main>
    }
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    SoundbankSelector,

    #[at("/hirc")]
    HircEditor,
    #[at("/hirc/:object")]
    HircRoot { object: u32 },
}

fn switch(route: Route) -> Html {
    match route {
        Route::SoundbankSelector
            => html!{ <SoundbankSelector /> },

        Route::HircEditor
            => shell(html!{ <HircEditor /> }),
        Route::HircRoot { object }
            => shell(html!{ <HircEditor object={object} /> }),
    }
}

fn shell(input: Html) -> Html {
    html! {
        <div>
            <div class="relative z-50" role="dialog" aria-modal="true">
                <div class="fixed inset-0 bg-gray-900/80"></div>

                <div class="fixed inset-0 flex">
                    {input}
                </div>
            </div>
        </div>
    }



    /*
    html!{
        <div class="min-h-full">
          <nav class="bg-gray-800">
            <div class="mx-auto px-4 sm:px-6 lg:px-8">
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <img class="h-8 w-8" src="https://tailwindui.com/img/logos/mark.svg?color=red&shade=500" alt="Rewwise" />
                  </div>
                  <div class="hidden md:block">
                    <div class="ml-10 flex items-baseline space-x-4">
                      <a href="#" class="bg-gray-900 text-white px-3 py-3 text-sm font-medium" aria-current="page">{"HIRC"}</a>
                      <a href="#" class="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-3 text-sm font-medium">{"Media"}</a>
                    </div>
                  </div>
                </div>
                <div class="block">
                  <div class="ml-4 flex items-center md:ml-6">

                    <button type="button" class="relative rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800">
                      <span class="absolute -inset-1.5"></span>
                      <span class="sr-only">{"View notifications"}</span>
                      <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M342.6 150.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0L192 210.7 86.6 105.4c-12.5-12.5-32.8-12.5-45.3 0s-12.5 32.8 0 45.3L146.7 256 41.4 361.4c-12.5 12.5-12.5 32.8 0 45.3s32.8 12.5 45.3 0L192 301.3 297.4 406.6c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L237.3 256 342.6 150.6z" />
                      </svg>
                    </button>

                  </div>
                </div>
              </div>
            </div>
          </nav>

          <main>
            <div class="flex text-white px-4">
                {input}
            </div>
          </main>
        </div>
    }
    */
}
