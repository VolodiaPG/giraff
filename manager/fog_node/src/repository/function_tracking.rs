use model::domain::sla::Sla;
use model::dto::function::{
    Finishable, Finished, Live, Paid, Proposed, Provisioned,
};
use model::SlaId;
use std::fmt::Debug;

use super::faas::RemovableFunctionRecord;

#[derive(Debug)]
pub enum States {
    Proposed(Proposed),
    Paid(Paid),
    Provisioned(Provisioned),
    Live(Live),
    #[allow(dead_code)]
    Finished(Finished),
}

impl From<Proposed> for States {
    fn from(value: Proposed) -> Self { States::Proposed(value) }
}
impl From<Paid> for States {
    fn from(value: Paid) -> Self { States::Paid(value) }
}
impl From<Provisioned> for States {
    fn from(value: Provisioned) -> Self { States::Provisioned(value) }
}
impl From<Live> for States {
    fn from(value: Live) -> Self { States::Live(value) }
}
impl From<Finished> for States {
    fn from(value: Finished) -> Self { States::Finished(value) }
}

#[derive(Debug, Default)]
pub struct FunctionTracking {
    database: dashmap::DashMap<SlaId, States>,
}

impl FunctionTracking {
    pub fn insert(&self, record: Proposed) {
        let id = record.sla.id.clone();
        self.database.insert(id.clone(), record.into());
        #[cfg(test)]
        assert!(matches!(
            self.database.get(&id).unwrap().value(),
            States::Proposed(_)
        ));
        #[cfg(test)]
        assert_eq!(self.get_proposed(&id).unwrap().sla.id, id);
    }

    pub fn get_proposed(&self, id: &SlaId) -> Option<Proposed> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Proposed(x) => Some(x.clone()),
            _ => None,
        })
    }

    pub fn get_paid(&self, id: &SlaId) -> Option<Paid> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Paid(x) => Some(x.clone()),
            _ => None,
        })
    }

    pub fn save_paid(&self, id: &SlaId, record: Paid) {
        let Some(mut previous_record) = self.database.get_mut(id) else {
            return;
        };
        let value = previous_record.value_mut();
        if let States::Proposed(_) = value {
            *value = record.into();
        };
    }

    pub fn save_provisioned(&self, id: &SlaId, record: Provisioned) {
        let Some(mut previous_record) = self.database.get_mut(id) else {
            return;
        };
        let value = previous_record.value_mut();
        if let States::Paid(_) = value {
            *value = record.into();
        };
    }

    pub fn save_live(&self, id: &SlaId, record: Live) {
        let Some(mut previous_record) = self.database.get_mut(id) else {
            return;
        };
        let value = previous_record.value_mut();
        if let States::Provisioned(_) = value {
            *value = record.into();
        };
    }

    pub fn save_finished(&self, id: &SlaId, record: Finished) {
        let Some(mut previous_record) = self.database.get_mut(id) else {
            return;
        };
        let value = previous_record.value_mut();
        *value = record.into();
    }

    pub fn get_provisioned(&self, id: &SlaId) -> Option<Provisioned> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Provisioned(x) => Some(x.clone()),
            _ => None,
        })
    }

    pub fn get_removable(
        &self,
        id: &SlaId,
    ) -> Option<RemovableFunctionRecord> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Live(x) => Some(x.clone().into()),
            States::Provisioned(x) => Some(x.clone().into()),
            _ => None,
        })
    }

    pub fn get_finishable_sla(&self, id: &SlaId) -> Option<Sla> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Live(Live { sla, .. })
            | States::Paid(Paid { sla, .. })
            | States::Provisioned(Provisioned { sla, .. }) => {
                Some(sla.clone())
            }
            _ => None,
        })
    }

    pub fn get_finishable(
        &self,
        id: &SlaId,
    ) -> Option<Box<dyn Finishable + Send + Sync>> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Paid(x) => {
                Some(Box::new(x.clone()) as Box<dyn Finishable + Send + Sync>)
            }
            States::Provisioned(x) => {
                Some(Box::new(x.clone()) as Box<dyn Finishable + Send + Sync>)
            }
            States::Live(x) => {
                Some(Box::new(x.clone()) as Box<dyn Finishable + Send + Sync>)
            }
            _ => None,
        })
    }

    pub fn get_finished(&self, id: &SlaId) -> Option<Finished> {
        self.database.get(id).and_then(|x| match x.value() {
            States::Finished(x) => Some(x.clone()),
            _ => None,
        })
    }
}
