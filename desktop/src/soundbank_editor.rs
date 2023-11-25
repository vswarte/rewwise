use yew::prelude::*;

#[function_component(SoundbankEditor)]
pub fn soundbank_editor() -> Html {

    html! {
        <div class="text-white">
            <div class="text-sm font-medium text-center text-gray-500 border-b border-gray-200 dark:text-white dark:border-gray-700">
                <ul class="flex flex-wrap -mb-px">
                    <li class="me-2">
                        <a href="#" class="inline-block p-4 text-blue-600 border-b-2 border-blue-600 rounded-t-lg active dark:text-blue-500 dark:border-blue-500" aria-current="page">{"Events"}</a>
                    </li>
                    <li class="me-2">
                        <a href="#" class="inline-block p-4 border-b-2 border-transparent rounded-t-lg hover:text-gray-600 hover:border-gray-300 dark:hover:text-gray-300">{"Media"}</a>
                    </li>
                </ul>
            </div>

            <table>
                <thead>
                    <tr>
                        <th>{"Event ID"}</th>  
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>{"Test"}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}

