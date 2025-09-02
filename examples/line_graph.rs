use iced::Length;
use snowline::line_graph::LineGraph;

fn main() {
    iced::application(App::new, App::update, App::view)
        .run()
        .unwrap();
}

#[derive(Debug)]
enum Message {
    Tick,
}

struct App {
    line_graph_cache: iced::widget::canvas::Cache,
    data: Vec<f32>,
}

impl App {
    fn new() -> Self {
        Self {
            line_graph_cache: iced::widget::canvas::Cache::new(),
            data: (0..1000).map(|_| rand::random_range(0.0..=100.0)).collect(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                // Handle tick event
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let text = iced::widget::text("Line Graph - Scroll to zoom, hover for values");

        // Create data iterator - now just pass the values, enumeration happens internally
        let data_iter = self.data.iter().copied();
        let graph = iced::Element::from(
            iced::widget::canvas(
                LineGraph::new(data_iter, &self.line_graph_cache)
                    .show_points(true)
                    .point_radius(4.0)
                    .line_width(2.5)
                    .performance_colors()
                    .show_grid(true)
                    .show_labels(true)
                    .base_points(100.0)
                    .zoom_range(0.1, 5.0),
            )
            .width(Length::Fixed(650.0))
            .height(Length::Fixed(350.0)),
        )
        .map(|_| Message::Tick);

        iced::widget::container(iced::widget::column![text, graph])
            .center(Length::Fill)
            .into()
    }
}
