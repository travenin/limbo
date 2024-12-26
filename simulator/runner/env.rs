use std::rc::Rc;
use std::sync::Arc;

use limbo_core::{Connection, Database};
use rand_chacha::ChaCha8Rng;

use crate::model::table::Table;

use crate::runner::io::SimulatorIO;

pub(crate) struct SimulatorEnv {
    pub(crate) opts: SimulatorOpts,
    pub(crate) tables: Vec<Table>,
    pub(crate) connections: Vec<SimConnection>,
    pub(crate) io: Arc<SimulatorIO>,
    pub(crate) db: Arc<Database>,
    pub(crate) rng: ChaCha8Rng,
}

#[derive(Clone)]
pub(crate) enum SimConnection {
    Connected(Rc<Connection>),
    Disconnected,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct SimulatorOpts {
    pub(crate) ticks: usize,
    pub(crate) max_connections: usize,
    pub(crate) max_tables: usize,
    // this next options are the distribution of workload where read_percent + write_percent +
    // delete_percent == 100%
    pub(crate) read_percent: usize,
    pub(crate) write_percent: usize,
    pub(crate) delete_percent: usize,
    pub(crate) max_interactions: usize,
    pub(crate) page_size: usize,
}
