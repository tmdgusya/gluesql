use {
    super::{
        err_into, key,
        lock::{get_txdata_key, Lock, TxData},
        SledStorage, Snapshot,
    },
    crate::{
        data::{Row, Schema},
        result::Result,
    },
};

impl SledStorage {
    pub fn gc(&self) -> Result<()> {
        let txids = self
            .tree
            .scan_prefix("tx_data/")
            .map(|item| -> Result<TxData> {
                item.map(|(_, v)| bincode::deserialize(&v))
                    .map_err(err_into)?
                    .map_err(err_into)
            })
            .take_while(|tx_data| match tx_data {
                Ok(TxData { alive, .. }) => !alive,
                Err(_) => false,
            })
            .map(|tx_data| tx_data.map(|TxData { txid, .. }| txid))
            .collect::<Result<Vec<u64>>>()?;

        let max_txid = match txids.iter().last() {
            Some(txid) => txid,
            None => {
                return Ok(());
            }
        };

        let Lock { lock_txid, .. } = self
            .tree
            .get("lock/")
            .map_err(err_into)?
            .map(|l| bincode::deserialize(&l))
            .transpose()
            .map_err(err_into)?
            .unwrap_or_default();

        let lock = Lock {
            lock_txid,
            gc_txid: Some(*max_txid),
        };

        bincode::serialize(&lock)
            .map(|lock| self.tree.insert("lock/", lock))
            .map_err(err_into)?
            .map_err(err_into)?;

        let fetch_keys = |prefix| {
            self.tree
                .scan_prefix(prefix)
                .map(|item| item.map_err(err_into))
                .collect::<Result<Vec<_>>>()
        };

        macro_rules! gc_txid {
            ($txid: expr, $prefix: expr, $T: ty) => {
                for (temp_key, data_key) in fetch_keys($prefix)? {
                    let snapshot: Option<Snapshot<$T>> = self
                        .tree
                        .get(&data_key)
                        .map_err(err_into)?
                        .map(|v| bincode::deserialize(&v))
                        .transpose()
                        .map_err(err_into)?;

                    let snapshot = match snapshot {
                        None => {
                            continue;
                        }
                        Some(snapshot) => snapshot.gc($txid),
                    };

                    match snapshot {
                        Some(snapshot) => {
                            bincode::serialize(&snapshot)
                                .map_err(err_into)
                                .map(|v| self.tree.insert(data_key, v))?
                                .map_err(err_into)?;
                        }
                        None => {
                            self.tree.remove(data_key).map_err(err_into)?;
                        }
                    }

                    self.tree.remove(temp_key).map_err(err_into)?;
                }
            };
        }

        for txid in txids {
            gc_txid!(txid, key::temp_data_prefix(txid), Row);
            gc_txid!(txid, key::temp_schema_prefix(txid), Schema);

            for (temp_key, data_key) in fetch_keys(key::temp_index_prefix(txid))? {
                let snapshots: Option<Vec<Snapshot<Vec<u8>>>> = self
                    .tree
                    .get(&data_key)
                    .map_err(err_into)?
                    .map(|v| bincode::deserialize(&v))
                    .transpose()
                    .map_err(err_into)?;

                let snapshots = match snapshots {
                    Some(snapshots) => snapshots,
                    None => {
                        continue;
                    }
                };

                let snapshots = snapshots
                    .into_iter()
                    .filter_map(|snapshot| snapshot.gc(txid))
                    .collect::<Vec<_>>();

                if snapshots.is_empty() {
                    self.tree.remove(data_key).map_err(err_into)?;
                } else {
                    bincode::serialize(&snapshots)
                        .map_err(err_into)
                        .map(|v| self.tree.insert(data_key, v))?
                        .map_err(err_into)?;
                }

                self.tree.remove(temp_key).map_err(err_into)?;
            }

            self.tree.remove(&get_txdata_key(txid)).map_err(err_into)?;
        }

        Ok(())
    }
}
