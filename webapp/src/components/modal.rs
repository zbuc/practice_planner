use std::rc::Rc;

use yew::prelude::*;
use yew::{function_component, html, use_state};

#[derive(Default, Properties, PartialEq, Clone)]
pub struct ModalDisplayProps {
    pub modal_type: String,
    pub content: Html,
    pub active: bool,
    // There's no Callback without an argument so this is just unused
    pub on_close_modal: Callback<usize>,
}

#[function_component(ModalDisplay)]
pub fn modal_display(props: &ModalDisplayProps) -> Html {
    let mut class = Classes::from("modal");
    if props.active {
        class.push("is-active");
    }

    let close_modal = || {
        let parent_handler = props.on_close_modal.clone();
        Callback::from(move |_| {
            log::info!("close_modal");
            parent_handler.emit(1);
        })
    };

    let content = props.content.clone();
    html! {
            <div {class}>
                <div class="modal-background"></div>
                <div class="modal-content">
                    <article class="message is-danger">
                        <div class="message-header">
                            <p>{"Danger"}</p>
                            <button class="delete" aria-label="close" onclick={close_modal()}></button>
                        </div>
                        <div class="message-body">
                        {content}
                        </div>
                    </article>
                </div>
                <button class="modal-close is-large" aria-label="close" onclick={close_modal()}></button>
            </div>
    }
}
