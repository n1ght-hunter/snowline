use iced::{Color, Length};
use snowline::bar_graph::{self, BarGraph};

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
    bar_graph_cache: iced::widget::canvas::Cache,
    data: Vec<f32>,
}

impl App {
    fn new() -> Self {
        Self {
            bar_graph_cache: iced::widget::canvas::Cache::new(),
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
        let text = iced::widget::text("Bar Graph - Aggregated bins, hover for values");

        // Create data iterator - now just pass the values, enumeration happens internally
        let data_iter = self.data.iter().copied();
        let graph = iced::Element::from(
            iced::widget::canvas(
                BarGraph::new(data_iter, &self.bar_graph_cache)
                    .bar_width(8.0)
                    .bar_color_scheme(bar_graph::color_scheme::BarColorScheme::palette(vec![
                        Color::from_rgb(0.2, 0.6, 1.0),
                        Color::from_rgb(1.0, 0.6, 0.2),
                        Color::from_rgb(0.3, 0.9, 0.4),
                    ]))
                    .show_grid(true)
                    .show_labels(true)
                    .bins(10)
                    .bin_aggregator(bar_graph::BinAggregator::Average),
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
