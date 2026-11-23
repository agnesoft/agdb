use crate::DbValue;

/// Database key-value pair (aka property) attached to
/// database elements. It can be constructed from a
/// tuple of types that are convertible to `DbValue`
/// or from a bare key which makes it a label
/// (value `DbValue::None`).
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct DbKeyValue {
    /// Key of the property
    pub key: DbValue,

    /// Value of the property
    pub value: DbValue,
}

impl<K, T> From<(K, T)> for DbKeyValue
where
    K: Into<DbValue>,
    T: Into<DbValue>,
{
    fn from(value: (K, T)) -> Self {
        DbKeyValue {
            key: value.0.into(),
            value: value.1.into(),
        }
    }
}

impl<K: Into<DbValue>> From<K> for DbKeyValue {
    fn from(key: K) -> Self {
        DbKeyValue {
            key: key.into(),
            value: DbValue::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!(
            "{:?}",
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            },
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_ord() {
        let element = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        let other = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        assert!(element <= other);
    }

    #[test]
    fn derived_from_ord() {
        let element = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        assert_eq!(element.cmp(&element), std::cmp::Ordering::Equal);
    }

    mod db_key_value {
        use super::*;

        mod from {
            use super::*;

            #[allow(clippy::module_inception)]
            mod from {
                use super::*;

                #[test]
                fn from_key_should_make_label() {
                    // WHEN
                    let labels: [DbKeyValue; 3] = ["label".into(), 1.into(), DbValue::None.into()];

                    // THEN
                    assert_eq!(
                        labels,
                        [
                            DbKeyValue {
                                key: "label".into(),
                                value: DbValue::None,
                            },
                            DbKeyValue {
                                key: 1.into(),
                                value: DbValue::None,
                            },
                            DbKeyValue {
                                key: DbValue::None,
                                value: DbValue::None,
                            },
                        ]
                    );
                }

                #[test]
                fn from_key_and_value_should_keep_value() {
                    // WHEN
                    let kv: DbKeyValue = ("key", 1).into();

                    // THEN
                    assert_eq!(
                        kv,
                        DbKeyValue {
                            key: "key".into(),
                            value: 1.into(),
                        }
                    );
                }
            }
        }
    }
}
