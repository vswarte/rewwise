use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;

#[function_component(Topbar)]
pub fn topbar() -> Html {
    let navigator = use_navigator().unwrap();

    let onclose = {
        let navigator = navigator.clone();
        move |_| {
            crate::soundbank::clear();
            navigator.push(&Route::SoundbankSelector);
        }
    };

    html! {
        <header class="flex flex-wrap sm:justify-start sm:flex-nowrap z-50 w-full bg-white text-sm py-2 dark:bg-gray-800">
            <nav class="w-full mx-auto px-4 sm:flex sm:items-center sm:justify-between" aria-label="Global">
                <a class="flex-none text-xl font-semibold dark:text-white" href="#">{"Rewwise"}</a>
                <div class="flex flex-row items-center gap-5 mt-5 sm:justify-end sm:mt-0 sm:ps-5">
                    <a class="font-medium text-gray-600 hover:text-gray-400 dark:text-gray-400 dark:hover:text-gray-500 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600" onclick={onclose}>
                        <svg class="w-[20px] h-[20px] fill-[#ffffff]" viewBox="0 0 384 512" xmlns="http://www.w3.org/2000/svg">
                            <path d="M342.6 150.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0L192 210.7 86.6 105.4c-12.5-12.5-32.8-12.5-45.3 0s-12.5 32.8 0 45.3L146.7 256 41.4 361.4c-12.5 12.5-12.5 32.8 0 45.3s32.8 12.5 45.3 0L192 301.3 297.4 406.6c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L237.3 256 342.6 150.6z"></path>
                        </svg>
                    </a>
                </div>
            </nav>
        </header>
    }
}
