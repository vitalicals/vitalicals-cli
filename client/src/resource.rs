use bdk::LocalUtxo;
use vital_script_primitives::resources::Resource;

pub struct LocalResource {
    pub utxo: LocalUtxo,
    pub resource: Resource,
    pub pending: bool,
}
