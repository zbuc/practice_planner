use super::event_bus::EventBus;
use yew::prelude::*;
use yew::{html, Component, Context, Html};
use yew_agent::{Bridge, Bridged};

pub enum Msg {
    PlaySound(String),
}

pub struct AudioPlayer {
    _producer: Box<dyn Bridge<EventBus>>,
    player: NodeRef,
}

impl Component for AudioPlayer {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _producer: EventBus::bridge(ctx.link().callback(Msg::PlaySound)),
            player: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PlaySound(_s) => {
                let player = &self.player.clone();
                // let is_playing = is_playing.clone();
                if let Some(audio) = player.cast::<web_sys::HtmlAudioElement>() {
                    // let is_playing = is_playing.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        // toggle music
                        if audio.paused() {
                            let toggle_play = audio.play().expect("Failed to play audio");
                            if let Err(err) =
                                wasm_bindgen_futures::JsFuture::from(toggle_play).await
                            {
                                log::error!("{:?}", err);
                            } else {
                                // is_playing.set(true);
                            }
                        } else {
                            audio.pause().expect("Failed to pause audio");
                            // is_playing.set(false);
                        }
                    });
                } else {
                    unreachable!()
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let audio_url = "ding.wav".to_string();

        html! {
            <>
            <audio class="hidden" src={audio_url.to_string()} ref={&self.player} />
            </>
        }
    }
}
