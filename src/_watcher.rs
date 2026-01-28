let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new("."), notify::RecursiveMode::Recursive)?;
    'watch: for res in rx {
        match res {
            Ok(event) => {
                for p in &event.paths {
                    if p.starts_with("/Users/k_semenov/Code/garage_sale/garage_sale_gen/target")
                        || p.starts_with("/Users/k_semenov/Code/garage_sale/garage_sale_gen/.git")
                    {
                        continue 'watch;
                    }
                }
                match event.kind {
                    notify::EventKind::Any => (),
                    notify::EventKind::Access(_) => (),
                    notify::EventKind::Create(create_kind) => match create_kind {
                        notify::event::CreateKind::Any => {
                            println!("event CreateKind::Any   : {:?}", event)
                        }
                        notify::event::CreateKind::File => {
                            println!("event CreateKind::File  : {:?}", event)
                        }
                        notify::event::CreateKind::Folder => {
                            println!("event CreateKind::Folder: {:?}", event)
                        }
                        notify::event::CreateKind::Other => {
                            println!("event CreateKind::Other : {:?}", event)
                        }
                    },
                    notify::EventKind::Modify(modify_kind) => match modify_kind {
                        notify::event::ModifyKind::Data(data_change) => {
                            println!("event ModifyKind::Data : {:?}", event)
                        }
                        notify::event::ModifyKind::Name(rename_mode) => {
                            println!("event ModifyKind::Name : {:?}", event)
                        },
                        _ => (),
                    },
                    notify::EventKind::Remove(remove_kind) => match remove_kind {
                        notify::event::RemoveKind::Any => {
                            println!("event RemoveKind::Any   : {:?}", event)
                        }
                        notify::event::RemoveKind::File => {
                            println!("event RemoveKind::File  : {:?}", event)
                        }
                        notify::event::RemoveKind::Folder => {
                            println!("event RemoveKind::Folder: {:?}", event)
                        }
                        notify::event::RemoveKind::Other => {
                            println!("event RemoveKind::Other : {:?}", event)
                        }
                    },
                    notify::EventKind::Other => (),
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
