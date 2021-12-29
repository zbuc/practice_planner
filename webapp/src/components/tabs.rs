use yew::prelude::*;
use yew::{function_component, html, use_state};

use crate::TabDisplayProps;

// Consider using something like Strum here
// so it can be ensured the tab is always
// a member of the enum
lazy_static! {
    pub(crate) static ref TABS: Vec<String> = vec![
        "Practice".to_string(),
        "History".to_string(),
        "Settings".to_string()
    ];
}

#[function_component(TabDisplay)]
pub fn tab_display(props: &TabDisplayProps) -> Html {
    let active_tab = use_state(|| &TABS[0]);

    let set_active_tab = |i| {
        let active_tab = active_tab.clone();
        let parent_handler = props.on_tab_change.clone();
        Callback::from(move |_| {
            parent_handler.emit(i);
            active_tab.set(&TABS[i]);
        })
    };

    html! {
        <>
        <div class="tabs">
        <ul>
            { for TABS.iter().enumerate().map(|(i,e)| {
                if e == *active_tab {
                    html!{<li class="is-active"><a>{e}</a></li>}
                } else {
                    html!{<li><a onclick={&set_active_tab(i)}>{e}</a></li>}
                }
            })}
        </ul>
        </div>
        </>
    }
}
