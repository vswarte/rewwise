use yew::prelude::*;
use yew_router::prelude::*;
use wwise_format::{HIRCSection, HIRCObject, HIRCObjectBody, SoundbankHelper};

use crate::soundbank;
use crate::app::Route;

#[derive(Properties, PartialEq)]
pub struct HircEditorProperties {
    pub object: Option<u32>,
}

#[function_component(HircEditor)]
pub fn hirc_editor(properties: &HircEditorProperties) -> Html {
    html! {
        <>
            <div class="relative flex w-96">
                <div class="flex grow flex-col gap-y-5 overflow-y-auto bg-gray-900 px-6 ring-1 ring-white/10">
                    <div class="flex h-16 shrink-0 items-center">
                        <img class="h-8 w-auto" src="https://tailwindui.com/img/logos/mark.svg?color=indigo&shade=500" alt="Your Company" />
                    </div>

                    <form action="#" method="GET">
                        <label for="search-field" class="sr-only">{"Search"}</label>
                        <div class="relative w-full">
                            <svg class="pointer-events-none absolute inset-y-0 left-0 h-full w-6 px-1 text-gray-500" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                <path fill-rule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clip-rule="evenodd" />
                            </svg>
                            <input id="search-field" class="block h-full w-full border-0 bg-transparent py-2 pl-8 pr-0 text-white focus:ring-0 sm:text-sm" placeholder="Search..." type="search" name="search" />
                        </div>
                    </form>

                    <nav class="flex flex-1 flex-col">
                        <ul role="list" class="flex flex-1 flex-col gap-y-7">
                            <li>
                                <ul role="list" class="-mx-2 space-y-1">
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <svg class="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
                                            </svg>
                                            {"Projects"}
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="bg-gray-800 text-white group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <svg class="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M21.75 17.25v-.228a4.5 4.5 0 00-.12-1.03l-2.268-9.64a3.375 3.375 0 00-3.285-2.602H7.923a3.375 3.375 0 00-3.285 2.602l-2.268 9.64a4.5 4.5 0 00-.12 1.03v.228m19.5 0a3 3 0 01-3 3H5.25a3 3 0 01-3-3m19.5 0a3 3 0 00-3-3H5.25a3 3 0 00-3 3m16.5 0h.008v.008h-.008v-.008zm-3 0h.008v.008h-.008v-.008z" />
                                            </svg>
                                            {"Deployments"}
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <svg class="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M9.348 14.651a3.75 3.75 0 010-5.303m5.304 0a3.75 3.75 0 010 5.303m-7.425 2.122a6.75 6.75 0 010-9.546m9.546 0a6.75 6.75 0 010 9.546M5.106 18.894c-3.808-3.808-3.808-9.98 0-13.789m13.788 0c3.808 3.808 3.808 9.981 0 13.79M12 12h.008v.007H12V12zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" />
                                            </svg>
                                            {"Activity"}
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <svg class="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418" />
                                            </svg>
                                            {"Domains"}
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <svg class="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 14.25v2.25m3-4.5v4.5m3-6.75v6.75m3-9v9M6 20.25h12A2.25 2.25 0 0020.25 18V6A2.25 2.25 0 0018 3.75H6A2.25 2.25 0 003.75 6v12A2.25 2.25 0 006 20.25z" />
                                            </svg>
                                            {"Usage"}
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <svg class="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                            </svg>
                                            {"Settings"}
                                        </a>
                                    </li>
                                </ul>
                            </li>
                            <li>
                                <div class="text-xs font-semibold leading-6 text-gray-400">{"Your teams"}</div>
                                <ul role="list" class="-mx-2 mt-2 space-y-1">
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-800 text-[0.625rem] font-medium text-gray-400 group-hover:text-white">{"P"}</span>
                                            <span class="truncate">{"Planetaria"}</span>
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-800 text-[0.625rem] font-medium text-gray-400 group-hover:text-white">{"P"}</span>
                                            <span class="truncate">{"Protocol"}</span>
                                        </a>
                                    </li>
                                    <li>
                                        <a href="#" class="text-gray-400 hover:text-white hover:bg-gray-800 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold">
                                            <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-800 text-[0.625rem] font-medium text-gray-400 group-hover:text-white">{"T"}</span>
                                            <span class="truncate">{"Tailwind Labs"}</span>
                                        </a>
                                    </li>
                                </ul>
                            </li>
                            <li class="-mx-6 mt-auto">
                                <a href="#" class="flex items-center gap-x-4 px-6 py-3 text-sm font-semibold leading-6 text-white">
                                    {"Player UI goes here"}
                                </a>
                            </li>
                        </ul>
                    </nav>
                </div>
            </div>

            <div class="w-full">
                <div class="sticky top-0 z-40 flex h-16 shrink-0 items-center gap-x-6 border-b border-white/5 bg-gray-900 px-4 shadow-sm sm:px-6 lg:px-8">
                    <div class="flex flex-1 gap-x-4 self-stretch lg:gap-x-6">
                        <form class="flex flex-1" action="#" method="GET"></form>
                    </div>
                </div>

                <main>
                    <header class="flex items-center justify-between border-b border-white/5 px-4 py-4 sm:px-6 sm:py-6 lg:px-8">
                        <h1 class="text-base font-semibold leading-7 text-white">{"HIRC Object"}</h1>

                        /*
                        <div class="relative">
                            <button type="button" class="flex items-center gap-x-1 text-sm font-medium leading-6 text-white" id="sort-menu-button" aria-expanded="false" aria-haspopup="true">
                                {"Sort by"}
                                <svg class="h-5 w-5 text-gray-500" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                    <path fill-rule="evenodd" d="M10 3a.75.75 0 01.55.24l3.25 3.5a.75.75 0 11-1.1 1.02L10 4.852 7.3 7.76a.75.75 0 01-1.1-1.02l3.25-3.5A.75.75 0 0110 3zm-3.76 9.2a.75.75 0 011.06.04l2.7 2.908 2.7-2.908a.75.75 0 111.1 1.02l-3.25 3.5a.75.75 0 01-1.1 0l-3.25-3.5a.75.75 0 01.04-1.06z" clip-rule="evenodd" />
                                </svg>
                            </button>

                            <div class="absolute right-0 z-10 mt-2.5 w-40 origin-top-right rounded-md bg-white py-2 shadow-lg ring-1 ring-gray-900/5 focus:outline-none" role="menu" aria-orientation="vertical" aria-labelledby="sort-menu-button" tabindex="-1">
                                <a href="#" class="block px-3 py-1 text-sm leading-6 text-gray-900" role="menuitem" tabindex="-1" id="sort-menu-item-0">{"Name"}</a>
                                <a href="#" class="block px-3 py-1 text-sm leading-6 text-gray-900" role="menuitem" tabindex="-1" id="sort-menu-item-1">{"Date updated"}</a>
                                <a href="#" class="block px-3 py-1 text-sm leading-6 text-gray-900" role="menuitem" tabindex="-1" id="sort-menu-item-2">{"Environment"}</a>
                            </div>
                        </div>
                        */
                    </header>

                    <ul role="list" class="divide-y divide-white/5">
                        <li class="relative flex items-center space-x-4 px-4 py-4 sm:px-6 lg:px-8">
                            <div class="min-w-0 flex-auto">
                                <div class="flex items-center gap-x-3">
                                    <div class="flex-none rounded-full p-1 text-gray-500 bg-gray-100/10">
                                        <div class="h-2 w-2 rounded-full bg-current"></div>
                                    </div>
                                    <h2 class="min-w-0 text-sm font-semibold leading-6 text-white">
                                        <a href="#" class="flex gap-x-2">
                                            <span class="truncate">{"Planetaria"}</span>
                                            <span class="text-gray-400">{"/"}</span>
                                            <span class="whitespace-nowrap">{"ios-app"}</span>
                                            <span class="absolute inset-0"></span>
                                        </a>
                                    </h2>
                                </div>
                                <div class="mt-3 flex items-center gap-x-2.5 text-xs leading-5 text-gray-400">
                                    <p class="truncate">{"Deploys from GitHub"}</p>
                                    <svg viewBox="0 0 2 2" class="h-0.5 w-0.5 flex-none fill-gray-300">
                                        <circle cx="1" cy="1" r="1" />
                                    </svg>
                                    <p class="whitespace-nowrap">{"Initiated 1m 32s ago"}</p>
                                </div>
                            </div>
                            <div class="rounded-full flex-none py-1 px-2 text-xs font-medium ring-1 ring-inset text-gray-400 bg-gray-400/10 ring-gray-400/20">{"Preview"}</div>
                            <svg class="h-5 w-5 flex-none text-gray-400" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
                            </svg>
                        </li>

                    </ul>
                </main>

            </div>
        </>
    }

    /*
    let bnk_lock = soundbank::use_soundbank();
    let bnk = bnk_lock.as_ref().unwrap();

    let hirc = crate::soundbank::hirc(bnk).unwrap();
    let main_content = match properties.object {
        Some(o) => main_state(bnk.hirc_object(&o)),
        None => empty_state(),
    };

    html! {
        <>
            <div class="flex-initial w-64">
                {hirc_table(hirc)}
            </div>
            <div class="flex-1">
                {main_content}
            </div>
        </>
    }
    */
}

fn hirc_table(hirc: &HIRCSection) -> Html {
    let rows = hirc.objects.iter()
        .filter_map(|o| {
            Some(match &o.body {
                HIRCObjectBody::Event(e) => html! {
                    <tr>
                        <td class="text-left font-mono">
                            <Link<Route> to={Route::HircRoot { object: o.id }}> 
                                {o.id}
                            </Link<Route>> 
                        </td>
                    </tr>
                },
                _ => return None,
            })
        })
        .collect::<Vec<Html>>();

    html! {
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
    }
}

fn empty_state() -> Html {
    html! {
        <p>{"Select your cum"}</p>
    }
}

fn main_state(object: Option<&HIRCObjectBody> ) -> Html {
    let object = match object {
        Some(o) => o,
        None => { return html! { <p>{"Could not find selected object"}</p> }; },
    };

    match object {
        HIRCObjectBody::Event(e) => e.form(),
        _ => { return html! { <p>{"Selected unknown object type"}</p> }; }
    }
}

trait HIRCObjectForm {
    fn form(&self) -> Html;
}


impl HIRCObjectForm for u32 {
    fn form(&self) -> Html {
        html! {
            <input 
                type="number"
                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
                value={format!("{}", self)}
            />
        }
    }
}

impl HIRCObjectForm for wwise_format::CAkEvent {
    fn form(&self) -> Html {
        let actions = self.actions.iter().enumerate()
            .map(|(i, a)| html! {
                <div>
                    <label class="block text-sm font-medium leading-6">{format!("#{}", i)}</label>
                    <div class="mt-0">
                        {a.form()}
                    </div>
                </div>
            })
            .collect::<Html>();

        html! {
            <>
                <p>{"Actions"}</p>
                {actions}
            </>
        }
    }
}
