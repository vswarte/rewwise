use yew::prelude::*;
use web_sys::HtmlInputElement;
use gloo_file::{FileList, callbacks::FileReader};

pub struct OpenedFile {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Properties, PartialEq)]
pub struct FileSelectorProperties {
    pub id: String,
    pub label: String,
    pub onselectedfile: Callback<OpenedFile>,

    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub accept: Vec<String>,
}

#[function_component(FileSelector)]
pub fn file_selector(props: &FileSelectorProperties) -> Html {
    let state = use_state(|| None as Option<FileReader>);

    let state_onchange = state.clone();
    let onselectedfile = props.onselectedfile.clone();

    let onchange = Callback::from(move |e: Event| {
        let cb = onselectedfile.clone();
        let input_event: HtmlInputElement = e.target_unchecked_into();
        let files = input_event.files().map(FileList::from);

        let reader = files.map(|x| x.to_vec().remove(0))
            .map(|f| {
                let name = f.name().clone();

                gloo_file::callbacks::read_as_bytes(&f, move |r| {
                    cb.emit(OpenedFile {
                        name,
                        data: r.expect("Could not read data")
                    });
                })
            });

        state_onchange.set(reader);
    });

    html! {
        <div class="mt-2">
            <label for="file" class="block mb-2 text-sm font-semibold text-gray-900 dark:text-white">
                {&props.label}
            </label>
            <input type="file" name="file" id="file" class="bg-gray-50 text-gray-900 sm:text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-700 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" required={true} {onchange} />
        </div>
    }
}
