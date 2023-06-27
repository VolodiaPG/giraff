use model::dto::function::{Finished, FunctionRecord, Proposed, Provisioned};
use model::BidId;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub enum States {
    Proposed(Arc<FunctionRecord<Proposed>>),
    Provisioned(Arc<FunctionRecord<Provisioned>>),
    Finished(Arc<FunctionRecord<Finished>>),
}

impl From<Arc<FunctionRecord<Proposed>>> for States {
    fn from(value: Arc<FunctionRecord<Proposed>>) -> Self {
        States::Proposed(value)
    }
}
impl From<Arc<FunctionRecord<Provisioned>>> for States {
    fn from(value: Arc<FunctionRecord<Provisioned>>) -> Self {
        States::Provisioned(value)
    }
}
impl From<Arc<FunctionRecord<Finished>>> for States {
    fn from(value: Arc<FunctionRecord<Finished>>) -> Self {
        States::Finished(value)
    }
}

#[derive(Debug, Default)]
pub struct FunctionTracking {
    database: dashmap::DashMap<BidId, States>,
}

impl FunctionTracking {
    pub fn insert(&self, record: FunctionRecord<Proposed>) -> BidId {
        let id = BidId::from(uuid::Uuid::new_v4());
        self.database.insert(id.clone(), Arc::new(record).into());
        id
    }

    pub fn get_proposed(
        &self,
        id: &BidId,
    ) -> Option<Arc<FunctionRecord<Proposed>>> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Proposed(x) => Some(x.clone()),
            _ => None,
        })
    }

    pub fn save_provisioned(
        &self,
        id: &BidId,
        record: FunctionRecord<Provisioned>,
    ) {
        let Some(mut previous_record) = self.database
            .get_mut(id) else {
                return ;
            };
        let value = previous_record.value_mut();
        if let States::Proposed(_) = value {
            *value = Arc::new(record).into();
        };
    }

    pub fn save_finished(&self, id: &BidId, record: FunctionRecord<Finished>) {
        let Some(mut previous_record) = self.database
            .get_mut(id) else {
                return ;
            };
        let value = previous_record.value_mut();
        if let States::Provisioned(_) = value {
            *value = Arc::new(record).into();
        };
    }

    pub fn get_provisioned(
        &self,
        id: &BidId,
    ) -> Option<Arc<FunctionRecord<Provisioned>>> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Provisioned(x) => Some(x.clone()),
            _ => None,
        })
    }
}
