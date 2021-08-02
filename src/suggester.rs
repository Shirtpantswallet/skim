use crate::global::mark_new_run;
///! A Suggester is simply a Reader that is used to build elements for autocomplete

use crate::options::SkimOptions;
use crate::spinlock::SpinLock;
use crate::reader::{collect_item, CommandCollector,};
use crate::{SkimItem, SkimItemReceiver};
use crossbeam::channel::Sender;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize,Ordering};
use std::sync::Arc;

pub struct SuggesterControl {
    tx_interrupt: Sender<i32>,
    tx_interrupt_cmd: Option<Sender<i32>>,
    components_to_stop: Arc<AtomicUsize>,
    items: Arc<SpinLock<Vec<Arc<dyn SkimItem>>>>,
}

impl SuggesterControl {
    pub fn kill(self) {
        debug!(
            "kill suggerter, compontents before {}",
            self.components_to_stop.load(Ordering::SeqCst)
        );

        let _ = self.tx_interrupt_cmd.map(|tx| tx.send(1));
        let _ = self.tx_interrupt.send(1);
        while self.components_to_stop.load(Ordering::SeqCst) != 0 {}
    }

    pub fn take(&self) -> Vec<Arc<dyn SkimItem>> {
        let mut items = self.items.lock();
        let mut ret = Vec::with_capacity(items.len());
        ret.append(&mut items);
        ret
    }

    pub fn is_done(&self) -> bool {
        let items = self.items.lock();
        self.components_to_stop.load(Ordering::SeqCst) == 0 && items.is_empty()
    }
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
