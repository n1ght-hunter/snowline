use iced::Length;
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
            data: (0..100)
                .flat_map(|n| {
                    (0..10).map(move |_| {
                        rand::random_range(
                            (1.0 * (n as f32).min(0.1))..=(10.0 * (n as f32).min(0.1)),
                        )
                    })
                })
                .collect(),
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
                    .bar_color_scheme(bar_graph::color_scheme::BarColorScheme::performance())
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
