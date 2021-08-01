///! A Suggester is simply a Reader that is used to build elements for autocomplete

use crate::reader::{Reader, ReaderControl};

pub struct Suggester {
    sugg_collector: Rc<RefCell<dyn CommandCollector>>,
    rx_item: Option<SkimItemReceiver>,
}

impl Suggester {
    pub fn with_options(options: &SkimOptions) -> Self {
        Self {
            sugg_collector: options.sugg_collector.clone(),
            rx_item: None,
        }
    }
}
