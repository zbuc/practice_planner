use yew::prelude::*;
use yew::{function_component, html};

#[derive(Default, Properties, PartialEq, Clone)]
pub struct ModalDisplayProps {
    pub modal_type: String,
    pub modal_title: String,
    pub content: Html,
    pub active: bool,
    // There's no Callback without an argument so this is just unused
    pub on_close_modal: Callback<usize>,
}

#[function_component(ModalDisplay)]
pub fn modal_display(props: &ModalDisplayProps) -> Html {
    let mut modal_class = Classes::from("modal");
    if props.active {
        modal_class.push("is-active");
    }

    let mut article_class = Classes::from("message");
    article_class.push(format!("is-{}", props.modal_type));

    let close_modal = || {
        let parent_handler = props.on_close_modal.clone();
        Callback::from(move |_| {
            log::debug!("close_modal");
            parent_handler.emit(1);
        })
    };

    let content = props.content.clone();
    html! {
            <div class={modal_class}>
                <div class="modal-background"></div>
                <div class="modal-content">
                    <article class={article_class}>
                        <div class="message-header">
                            <p>{props.modal_title.clone()}</p>
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
