use i3ipc::{event, reply::NodeType, I3Connection, I3EventListener, Subscription};
use std::thread;
use std::time::Duration;

fn count_windows(node: &i3ipc::reply::Node) -> i32 {
    let mut count = 0;
    if node.nodetype == i3ipc::reply::NodeType::Con && node.window.is_some() {
        count += 1;
    }
    for child in &node.nodes {
        count += count_windows(child);
    }
    count
}

fn find_workspace_root<'a>(
    node: &'a i3ipc::reply::Node,
    workspace_name: &str,
) -> Option<&'a i3ipc::reply::Node> {
    if node.nodetype == i3ipc::reply::NodeType::Workspace
        && node
            .name
            .as_ref()
            .map_or(false, |name| name == workspace_name)
    {
        Some(node)
    } else {
        for child in &node.nodes {
            if let Some(workspace_root) = find_workspace_root(child, workspace_name) {
                return Some(workspace_root);
            }
        }
        None
    }
}

fn main() {
    let mut event_listener = match I3EventListener::connect() {
        Ok(event_listener) => event_listener,
        Err(error) => {
            println!("Error: {}", error);
            return;
        }
    };

    event_listener.subscribe(&[Subscription::Window]).unwrap();

    let mut iter = event_listener.listen();

    let mut i3 = match I3Connection::connect() {
        Ok(i3) => i3,
        Err(error) => {
            println!("Error: {}", error);
            return;
        }
    };

    i3.run_command("split h").unwrap();

    loop {
        thread::sleep(Duration::from_millis(100));

        // on window close or open event, toggle splitting horizontally and vertically
        let event = iter.next().unwrap().unwrap();

        let mut i3 = match I3Connection::connect() {
            Ok(i3) => i3,
            Err(error) => {
                println!("Error: {}", error);
                return;
            }
        };

        let tree = match i3.get_tree() {
            Ok(tree) => tree,
            Err(error) => {
                println!("Error: {}", error);
                return;
            }
        };

        let workspaces = match i3.get_workspaces() {
            Ok(workspaces) => workspaces,
            Err(error) => {
                println!("Error: {}", error);
                return;
            }
        };

        let current_workspace = workspaces
            .workspaces
            .iter()
            .find(|workspace| workspace.focused)
            .unwrap();

        let workspace_root = find_workspace_root(&tree, &current_workspace.name).unwrap();

        let mut windows = count_windows(&workspace_root);

        for node in tree.nodes {
            if node.nodetype == NodeType::Con {
                windows += 1;
            }
        }

        match event {
            event::Event::WindowEvent(window_event) => match window_event.change {
                event::inner::WindowChange::Close => {
                    let mut i3 = match I3Connection::connect() {
                        Ok(i3) => i3,
                        Err(error) => {
                            println!("Error: {}", error);
                            return;
                        }
                    };

                    if windows % 2 == 0 {
                        i3.run_command("split v").unwrap();
                    } else {
                        i3.run_command("split h").unwrap();
                    }
                }
                event::inner::WindowChange::New => {
                    let mut i3 = match I3Connection::connect() {
                        Ok(i3) => i3,
                        Err(error) => {
                            println!("Error: {}", error);
                            return;
                        }
                    };

                    if windows % 2 == 0 {
                        i3.run_command("split v").unwrap();
                    } else {
                        i3.run_command("split h").unwrap();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
