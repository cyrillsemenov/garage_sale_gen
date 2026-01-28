use std::collections::{BTreeMap, HashMap};
use tera::{Function, Result, Value, from_value, to_value};

use crate::site_builder::{PageId, models::NavNode};

pub(crate) fn make_url_for(urls: BTreeMap<PageId, String>) -> impl Function {
    Box::new(move |args: &HashMap<String, Value>| -> Result<Value> {
        match args.get("id") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => {
                    let page_id = PageId::from(v);
                    match urls.get(&page_id) {
                        Some(url) => Ok(to_value(url).unwrap()),
                        None => Err(format!("Page not found: {:?}", page_id).into()),
                    }
                }
                Err(_) => Err("Invalid argument type for 'id'".into()),
            },
            None => Err("Missing argument 'id'".into()),
        }
    })
}

pub(crate) fn make_breadcrumbs(breadcrumbs: BTreeMap<PageId, Vec<NavNode>>) -> impl Function {
    Box::new(move |args: &HashMap<String, Value>| -> Result<Value> {
        match args.get("id") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => {
                    let page_id = PageId::from(v);
                    match breadcrumbs.get(&page_id) {
                        Some(crumbs) => Ok(to_value(crumbs).unwrap()),
                        None => Ok(to_value(Vec::<NavNode>::new()).unwrap()),
                    }
                }
                Err(_) => Err("Invalid argument type for 'id'".into()),
            },
            None => Err("Missing argument 'id'".into()),
        }
    })
}
