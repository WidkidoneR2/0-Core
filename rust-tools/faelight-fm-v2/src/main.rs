// faelight-fm v2 -- INT-293 Phase 1 spike
// Goal: get a libcosmic window rendering on Wayland

use cosmic::app::{Core, Task};
use cosmic::{Application, ApplicationExt, Element, executor};
use cosmic::iced::Length;
use cosmic::widget;

#[derive(Debug, Clone)]
pub enum Message {
    None,
}

struct FaelightFm {
    core: Core,
}

impl Application for FaelightFm {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.faelight.fm";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        (Self { core }, Task::none())
    }

    fn view(&self) -> Element<Self::Message> {
        widget::container(
            widget::text("🌲 faelight-fm v2 -- the forest navigator")
                .size(24)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<FaelightFm>(
        cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(1200.0, 800.0)),
        ()
    )
}
