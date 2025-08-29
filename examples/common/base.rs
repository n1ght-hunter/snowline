use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use iced::{
    Application, Program,
    application::{Boot, Update, View},
    window,
};

use super::mcp_server::{AppMessage, server_task};

pub fn application<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>(
    boot: BootFn,
    update: UpdateFn,
    view: ViewFn,
) -> Application<
    impl Program<
        State = Base<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>,
        Message = BaseMessage<Message>,
        Theme = Theme,
    >,
>
where
    BootFn: Boot<State, Message> + 'static,
    UpdateFn: Update<State, Message> + 'static,
    ViewFn: for<'a> View<'a, State, Message, Theme, Renderer> + 'static,
    State: Send + 'static,
    Message: Send + Debug + 'static,
    Theme: Default + iced::theme::Base + 'static,
    Renderer: iced::advanced::Renderer
        + iced::advanced::text::Renderer
        + iced::advanced::graphics::compositor::Default
        + 'static,
{
    let wrapper = Arc::new(Wrap::new(boot, update, view));
    iced::application(move || Base::new(wrapper.clone()), Base::update, Base::view)
}

pub struct Wrap<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>
where
    BootFn: Boot<State, Message>,
    UpdateFn: Update<State, Message>,
    ViewFn: for<'a> View<'a, State, Message, Theme, Renderer>,
{
    boot: BootFn,
    update_fn: UpdateFn,
    view_fn: ViewFn,
    _state: std::marker::PhantomData<State>,
    _message: std::marker::PhantomData<Message>,
    _theme: std::marker::PhantomData<Theme>,
    _renderer: std::marker::PhantomData<Renderer>,
}

impl<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>
    Wrap<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>
where
    BootFn: Boot<State, Message>,
    UpdateFn: Update<State, Message>,
    ViewFn: for<'a> View<'a, State, Message, Theme, Renderer>,
{
    pub fn new(boot: BootFn, update_fn: UpdateFn, view_fn: ViewFn) -> Self {
        Self {
            boot,
            update_fn,
            view_fn,
            _state: std::marker::PhantomData,
            _message: std::marker::PhantomData,
            _theme: std::marker::PhantomData,
            _renderer: std::marker::PhantomData,
        }
    }
}

pub struct Base<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>
where
    BootFn: Boot<State, Message>,
    UpdateFn: Update<State, Message>,
    ViewFn: for<'a> View<'a, State, Message, Theme, Renderer>,
{
    state: State,
    wrapper: Arc<Wrap<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>>,
}

#[derive(Debug)]
pub enum BaseMessage<Message> {
    UpdateState(Message),
    McpServerMessage(AppMessage),
}

impl<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>
    Base<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>
where
    BootFn: Boot<State, Message>,
    UpdateFn: Update<State, Message>,
    ViewFn: for<'a> View<'a, State, Message, Theme, Renderer>,
    Renderer: iced::advanced::Renderer,
    State: Send + 'static,
    Message: Send + 'static,
{
    fn new(
        wrapper: Arc<Wrap<State, BootFn, UpdateFn, ViewFn, Message, Theme, Renderer>>,
    ) -> (Self, iced::Task<BaseMessage<Message>>) {
        let (state, tasks) = wrapper.boot.boot();
        let server_task = server_task();
        (
            Self { wrapper, state },
            iced::Task::batch([tasks.map(BaseMessage::UpdateState), server_task]),
        )
    }

    pub fn update(&mut self, message: BaseMessage<Message>) -> iced::Task<BaseMessage<Message>> {
        match message {
            BaseMessage::UpdateState(msg) => {
                return self
                    .wrapper
                    .update_fn
                    .update(&mut self.state, msg)
                    .map(BaseMessage::UpdateState);
            }
            BaseMessage::McpServerMessage(msg) => match msg {
                AppMessage::Screenshot(message_wrap) => {
                    let sender = Arc::new(Mutex::new(Some(message_wrap.responder)));
                    return window::get_latest()
                        .and_then(window::screenshot)
                        .then(move |screen| {
                            let output = message_wrap.message.clone();
                            let sender = sender.clone();
                            iced::Task::future(async move {
                                let res = tokio::task::spawn_blocking({
                                    let output = output.clone();
                                    move || {
                                        let size = screen.size;
                                        image::save_buffer_with_format(
                                            &output,
                                            &screen.bytes,
                                            size.width,
                                            size.height,
                                            image::ColorType::Rgba8,
                                            image::ImageFormat::Png,
                                        )?;

                                        Ok(())
                                            as Result<(), Box<dyn std::error::Error + Send + Sync>>
                                    }
                                })
                                .await;

                                match res {
                                    Ok(Ok(())) => {
                                        if let Some(sender) = sender.lock().unwrap().take() {
                                            let _ = sender.send(Ok(()));
                                        }
                                    }
                                    Ok(Err(e)) => {
                                        if let Some(sender) = sender.lock().unwrap().take() {
                                            let _ = sender.send(Err(e));
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(sender) = sender.lock().unwrap().take() {
                                            let _ = sender.send(Err(Box::new(e)));
                                        }
                                    }
                                }
                            })
                        })
                        .discard();
                }
                AppMessage::Exit => {
                    // Exit the application - this will terminate the app
                    return iced::exit::<BaseMessage<Message>>();
                }
            },
        };
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, BaseMessage<Message>, Theme, Renderer> {
        self.wrapper
            .view_fn
            .view(&self.state)
            .map(BaseMessage::UpdateState)
    }
}
