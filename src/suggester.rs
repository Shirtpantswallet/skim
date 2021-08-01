use crate::global::mark_new_run;
///! A Suggester is simply a Reader that is used to build elements for autocomplete

use crate::options::SkimOptions;
use crate::spinlock::SpinLock;
use crate::reader::{collect_item, CommandCollector, Reader, ReaderControl};
use crate::{SkimItem, SkimItemReceiver};
use crossbeam::channel::{bounded, select, Sender};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct SuggesterControl {
    tx_interrupt: Sender<i32>,
    tx_interrupt_cmd: Option<Sender<i32>>,
    components_to_stop: Arc<AtomicUsize>,
    items: Arc<SpinLock<Vec<Arc<dyn SkimItem>>>>,
}

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

    pub fn source(mut self, rx_item: Option<SkimItemReceiver>) -> Self {
        self.rx_item = rx_item;
        self
    }

    pub fn run(&mut self, cmd: &str) -> SuggesterControl {
        mark_new_run(cmd);

        let components_to_stop: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let items = Arc::new(SpinLock::new(Vec::new()));
        let items_clone = items.clone();

        let (rx_item, tx_interrupt_cmd) = self.rx_item.take().map(|rx| (rx, None)).unwrap_or_else(|| {
            let components_to_stop_clone = components_to_stop.clone();
            let (rx_item, tx_interrupt_cmd) = self.sugg_collector.borrow_mut().invoke(cmd, components_to_stop_clone);
            (rx_item, Some(tx_interrupt_cmd))
        });

        let components_to_stop_clone = components_to_stop.clone();
        let tx_interrupt = collect_item(components_to_stop_clone, rx_item, items_clone);

        SuggesterControl {
            tx_interrupt,
            tx_interrupt_cmd,
            components_to_stop,
            items,
        }
    }
}
