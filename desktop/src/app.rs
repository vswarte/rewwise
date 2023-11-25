use yew::prelude::*;
use yew_router::prelude::*;
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
enum Route {
    #[at("/")]
    SoundbankSelector,
    #[at("/editor")]
    Editor,
    #[at("/player")]
    Player,
}

fn switch(route: Route) -> Html {
    match route {
        Route::SoundbankSelector => html!{ <SoundbankSelector /> },
        Route::Editor => todo!(),
        Route::Player => player(),
    }
}

fn player() -> Html {
    html! {
        <div>
            <div class="bg-white border-slate-100 dark:bg-slate-800 dark:border-slate-500 border-b p-4 pb-6 sm:p-10 sm:pb-8 lg:p-6 xl:p-10 xl:pb-8 space-y-6 sm:space-y-8 lg:space-y-6 xl:space-y-8">
              <div class="flex items-center space-x-4">
                <img src="https://i1.sndcdn.com/artworks-C94lW33b2UYMVKaQ-SWQ0jA-t500x500.jpg" alt="" width="88" height="88" class="flex-none rounded-lg bg-slate-100" loading="lazy" />
                <div class="min-w-0 flex-auto space-y-1 font-semibold">
                  <p class="text-cyan-500 dark:text-cyan-400 text-sm leading-6">
                    <abbr title="Episode">{"Ep."}</abbr> {"128"}
                  </p>
                  <h2 class="text-slate-500 dark:text-slate-400 text-sm leading-6 truncate">
                      {"Cowboy Brodown"}
                  </h2>
                  <p class="text-slate-900 dark:text-slate-50 text-lg">
                      {"Big Gay"}
                  </p>
                </div>
              </div>
              <div class="space-y-2">
                <div class="relative">
                  <div class="bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden">
                    <div class="bg-cyan-500 dark:bg-cyan-400 w-1/2 h-2" role="progressbar" aria-label="music progress" aria-valuenow="1456" aria-valuemin="0" aria-valuemax="4550"></div>
                  </div>
                  <div class="ring-cyan-500 dark:ring-cyan-400 ring-2 absolute left-1/2 top-1/2 w-4 h-4 -mt-2 -ml-2 flex items-center justify-center bg-white rounded-full shadow">
                    <div class="w-1.5 h-1.5 bg-cyan-500 dark:bg-cyan-400 rounded-full ring-1 ring-inset ring-slate-900/5"></div>
                  </div>
                </div>
                <div class="flex justify-between text-sm leading-6 font-medium tabular-nums">
                  <div class="text-cyan-500 dark:text-slate-100">{"24:16"}</div>
                  <div class="text-slate-500 dark:text-slate-400">{"75:50"}</div>
                </div>
              </div>
            </div>
            <div class="bg-slate-50 text-slate-500 dark:bg-slate-600 dark:text-slate-200 flex items-center">
              <button type="button" class="bg-white text-slate-900 dark:bg-slate-100 dark:text-slate-700 flex-none -my-2 mx-auto w-20 h-20 rounded-full ring-1 ring-slate-900/5 shadow-md flex items-center justify-center" aria-label="Pause">
                <svg width="30" height="32" fill="currentColor">
                  <rect x="6" y="4" width="4" height="24" rx="2" />
                  <rect x="20" y="4" width="4" height="24" rx="2" />
                </svg>
              </button>
            </div>
        </div>
    }
}
