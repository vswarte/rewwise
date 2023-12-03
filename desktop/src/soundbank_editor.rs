use yew::prelude::*;
use yew_router::prelude::*;
use wwise_format::HIRCObjectBody;

use crate::app::Route;
use crate::components::Topbar;

#[function_component(SoundbankEditor)]
pub fn soundbank_editor() -> Html {
    let navigator = use_navigator().unwrap();

    let lock = crate::soundbank::PRIMARY_SOUNDBANK.get().unwrap().read().unwrap();
    let bnk = match lock.as_ref() {
        Some(b) => b,
        None => {
            // Redirect user if there is no soundbank loaded
            navigator.push(&Route::SoundbankSelector);
            return html! {};
        }
    };
    let hirc = crate::soundbank::hirc(bnk).unwrap();

    let rows = hirc.objects.iter()
        .filter_map(|o| {
            Some(match &o.body {
                HIRCObjectBody::Event(e) => html! {
                    <tr>
                        <td class="text-left font-mono">{o.id}</td>
                    </tr>
                },
                _ => return None,
            })
        })
        .collect::<Vec<Html>>();

    html! {
        <div class="text-white">
            <Topbar />

            <div class="border-b border-gray-200 dark:border-gray-700 pt-2 bg-slate-600">
                <nav class="flex space-x-1">
                    <a class="ms-2 -mb-px py-2 px-2 inline-flex items-center gap-2 bg-white text-sm font-medium text-center border border-b-0 text-blue-600 rounded-t-lg dark:bg-slate-900 dark:border-gray-700 dark:border-b-gray-800 dark:hover:text-gray-400 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600" href="#">
                        {"Events"}
                    </a>
                    <a class="-mb-px py-1 px-2 inline-flex items-center gap-2 bg-gray-50 text-sm font-medium text-center border text-gray-500 rounded-t-lg hover:text-gray-700 dark:bg-gray-700 dark:border-gray-700 dark:text-gray-400 dark:hover:text-gray-300 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600" href="#">
                        {"Media"}
                    </a>
                    <a class="-mb-px py-1 px-2 inline-flex items-center gap-2 bg-gray-50 text-sm font-medium text-center border text-gray-500 rounded-t-lg hover:text-gray-700 dark:bg-gray-700 dark:border-gray-700 dark:text-gray-400 dark:hover:text-gray-300 dark:focus:outline-none dark:focus:ring-1 dark:focus:ring-gray-600" href="#">
                        {"Routing"}
                    </a>
                </nav>
            </div>


            <div class="min-h-screen mt-4 px-2">
                <table class="table-auto w-full">
                    <thead>
                        <tr>
                            <th class="text-left">{"Event ID"}</th>  
                        </tr>
                    </thead>
                    <tbody>
                        {rows}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
