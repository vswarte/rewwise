use yew::prelude::*;
use wwise_format::HIRCObjectBody;

#[function_component(SoundbankEditor)]
pub fn soundbank_editor() -> Html {
    let lock = crate::soundbank::PRIMARY_SOUNDBANK.get().unwrap().read().unwrap();
    let bnk = lock.as_ref().unwrap();
    let hirc = crate::soundbank::hirc(bnk).unwrap();

    let rows = hirc.objects.iter()
        .filter_map(|o| {
            Some(match &o.body {
                HIRCObjectBody::Event(e) => html! {
                    <tr>
                        <td class="text-left">{o.id}</td>
                    </tr>
                },
                _ => return None,
            })
        })
        .collect::<Vec<Html>>();

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
    }
}

