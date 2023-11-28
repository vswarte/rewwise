use yew::prelude::*;
use gloo_worker::Spawnable;
use yew_router::prelude::*;
use rewwise_worker::ParseWorker;
use wasm_bindgen_futures::spawn_local;

use crate::app::Route;
use crate::components::{FileSelector, OpenedFile};

#[function_component(SoundbankSelector)]
pub fn soundbank_selector() -> Html {
    let navigator = use_navigator().unwrap();
    let is_loading = use_state(|| false);
    let error_message = use_state(String::new);

    let onselectedfile = {
        let error_message = error_message.clone();
        let is_loading = is_loading.clone();

        move |file: OpenedFile| {
            is_loading.set(true);

            let navigator = navigator.clone();
            let is_loading = is_loading.clone();
            let error_message = error_message.clone();
            spawn_local(async move {
                let mut parse = ParseWorker::spawner()
                    .spawn("parse-worker.js");

                match parse.run(file.data).await {
                    Ok(s) => {
                        is_loading.set(false);
                        error_message.set("".to_string());
                        crate::soundbank::set(s);
                        navigator.push(&Route::SoundbankEditor);
                    },
                    Err(e) => {
                        is_loading.set(false);
                        error_message.set(format!("{:#?}", e));
                    },
                }
            });
        }
    };

    let error = {
        if !error_message.is_empty() {
            html! {
                <div class="rounded-lg flex items-center px-6 pt-4 text-red-800 border-t-4 border-red-300 bg-red-50 dark:text-red-400 dark:bg-gray-800 dark:border-red-800" role="alert">
                    <svg class="flex-shrink-0 w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5ZM9.5 4a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3ZM12 15H8a1 1 0 0 1 0-2h1v-3H8a1 1 0 0 1 0-2h2a1 1 0 0 1 1 1v4h1a1 1 0 0 1 0 2Z"/>
                    </svg>
                    <div class="ms-3">
                        <span class="font-semibold">{"There was an issue parsing your soundbank:"}</span>
                        <p>{(*error_message).clone()}</p>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    };

    let mut blur_classes = Classes::from("transition ease-in-out duration-300");
    if *is_loading {
        blur_classes.push("blur");
    }

    html! {
        <div class="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0">
            if *is_loading {
                <div role="status" class="absolute -translate-x-1/2 -translate-y-1/2 top-2/4 left-1/2 z-10 text-center">
                    <h2 class="text-lg font-bold leading-tight tracking-tight text-gray-900 md:text-4xl dark:text-white mb-5 text-center">
                        {"Loading soundbank"}
                    </h2>
                    <svg aria-hidden="true" class="inline w-10 h-10text-gray-200 animate-spin dark:text-gray-600 fill-blue-600" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="currentColor"/>
                        <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentFill"/>
                    </svg>
                </div>
            }

            <div class={blur_classes}>
                <h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-4xl dark:text-white mb-5 text-center">
                    {"Rewwise"}
                </h1>

                <div class="shadow-xl dark:border md:mt-0 sm:max-w-md xl:p-0 dark:bg-gray-800 dark:border-gray-700 rounded-lg">
                    {error}

                    <div class="p-6 space-y-4 md:space-y-6 sm:p-8">
                        <h2 class="font-bold text-gray-900 md:text-lg dark:text-white pb-0">
                            {"Select your Wwise soundbank"}
                        </h2>
                        <FileSelector id="id" label="Soundbank" {onselectedfile} />
                    </div>
                </div>
            </div>
        </div> 
    }
}
