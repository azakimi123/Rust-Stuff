// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]
// #![allow(dead_code, unused_variables)]

//imports
use seed::{prelude::*, *};


use std::collections::BTreeMap;
use std::mem;


use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use ulid::Ulid;

const ENTER_KEY: &str = "Enter";

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        todos: BTreeMap::new(),
        new_todo_title: String::new(),
        selected_todo: None,
        filter: Filter::All,
        base_url: Url::new(),
    }.add_mock_data()
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.

struct Model {
    //Ulid ids are unique and sortable. Their ordering depends on timestamps so it's suitable for our use-case (ordering from the oldest).
    //A B-Tree instead makes each node contain B-1 to 2B-1 elements in a contiguous array. By doing this, we reduce the number of allocations by a factor of B, and improve cache efficiency in searches.
    todos: BTreeMap<Ulid, Todo>,
    new_todo_title: String,
    selected_todo: Option<SelectedTodo>,
    filter: Filter,
    base_url: Url

}


struct Todo {
    id: Ulid,
    title: String,
    completed: bool,
}

struct SelectedTodo {
    id: Ulid,
    title: String,
    input_element: ElRef<web_sys::HtmlInputElement>,
}

//remember which filter is selected
#[derive(Copy, Clone, Eq, PartialEq, EnumIter)]
enum Filter {
    All,
    Active,
    Completed,
 }

// Data to be displayed at start
impl Model {
    fn add_mock_data(mut self) -> Self {
        let (id_a, id_b, id_c, id_d) = (Ulid::new(), Ulid::new(), Ulid::new(), Ulid::new());
        
        self.todos.insert(id_a, Todo {
            id: id_a,
            title: "Do Right".to_owned(),
            completed: false,
        });

        self.todos.insert(id_b, Todo {
            id: id_b,
            title: "Be The Crew".to_owned(),
            completed: false,
        });
        
        self.todos.insert(id_c, Todo {
            id: id_c,
            title: "Own It".to_owned(),
            completed: false,
        });

        self.todos.insert(id_d, Todo {
            id: id_d,
            title: "Chew The Strap".to_owned(),
            completed: false,
        });

        self
    }
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
//kind of like functions that take in arguements 
// #[derive(Copy, Clone)] - not needed because subs::UrlChanged and String don't implement Copy and standalone Clone for Msg is an anti-pattern.
// `Msg` describes the different events you can modify state with.
enum Msg {
    //update the field filter
    UrlChanged(subs::UrlChanged),
    //to store input element content
    NewTodoTitleChanged(String),


    // ------ Basic Todo operations ------

    //to signal that user wants to push a new todo into the list
    CreateTodo,
    ToggleTodo(Ulid),
    RemoveTodo(Ulid),

    // ------ Bulk operations ------
    
    CheckOrUncheckAll,
    ClearCompleted,

    // ------ Selection ------
    
    //toggles the field completed
    SelectTodo(Option<Ulid>),
    //It stores a new title to SelectedTodo
    SelectedTodoTitleChanged(String),
    //It "moves" title from SelectedTodo into the corresponding Todo in todos
    SaveSelectedTodo
    
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            log!("UrlChanged", url);
        }
        Msg::NewTodoTitleChanged(title) => {
            model.new_todo_title = title;
            // log!("NewTodoTitleChanged", title);
        }
    
        // ------ Basic Todo operations ------

        Msg::CreateTodo => {
            let title = model.new_todo_title.trim();
            if not(title.is_empty()){
                let id = Ulid::new();
                model.todos.insert(id, Todo {
                    id,
                    title: title.to_owned(),
                    completed: false,
                });
                model.new_todo_title.clear();
            }
            log!("CreateTodo");
        }
        Msg::ToggleTodo(id) => {
            if let Some(todo) = model.todos.get_mut(&id) {
                todo.completed = not(todo.completed);
            }
            log!("ToggleTodo");
        }
        Msg::RemoveTodo(id) => {
            model.todos.remove(&id);
            log!("RemoveTodo");
        }
        
        // ------ Bulk operations ------

        Msg::CheckOrUncheckAll => {
            log!("CheckOrUncheckAll");
        }
        Msg::ClearCompleted => {
            //Todo: Refractor with 'BTreeMap::drain_filter' once stable.
            model.todos = mem::take(&mut model.todos)
                .into_iter()
                .filter(|(_, todo) | not(todo.completed))
                .collect();
            log!("ClearCompleted");
        }
        
        // ------ Selection ------

        Msg::SelectTodo(opt_id) => {
            log!("SelectTodo", opt_id);
        },
        Msg::SelectedTodoTitleChanged(title) => {
            log!("SelectedTodoTitleChanged", title);
        },
        Msg::SaveSelectedTodo => {
            log!("SaveSelectedTodo");
        }
    }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        view_header(&model.new_todo_title),
        IF!(not(model.todos.is_empty()) => vec![
            // This section should be hidden by default and shown when there are todos
            view_main(&model.todos, model.selected_todo.as_ref()),
            // This footer should be hidden by default and shown when there are todos
            view_footer(&model.todos, model.filter),
        ]),
    ]
}

// ------ header ------

fn view_header(new_todo_title: &str) -> Node<Msg> {
    header![C!["header"],
        input![C!["new-todo"],
            attrs!{
                At::Placeholder => "What needs to be done?", 
                At::AutoFocus => AtValue::None,
                At::Value => new_todo_title,
            },
            input_ev(Ev::Input, Msg::NewTodoTitleChanged),
            keyboard_ev(Ev::KeyDown, |keyboard_event| {
                IF!(keyboard_event.key() == ENTER_KEY => Msg::CreateTodo)
            })
        ]
    ]
}

// ------ main ------

fn view_main(todos: &BTreeMap<Ulid, Todo>, selected_todo: Option<&SelectedTodo>) -> Node<Msg> {
    section![C!["main"],
        view_toggle_all(todos),
        view_todo_list(todos, selected_todo),
    ]
}

fn view_toggle_all(todos: &BTreeMap<Ulid, Todo>) -> Vec<Node<Msg>> {
    let all_completed = todos.values().all(|todo| todo.completed);
    vec![
        input![C!["toggle-all"], 
            attrs!{
                At::Id => "toggle-all", 
                At::Type => "checkbox", 
                At::Checked => all_completed.as_at_value()
            }
        ],
        label![attrs!{At::For => "toggle-all"}, "Mark all as complete"],
    ]
}

fn view_todo_list(todos: &BTreeMap<Ulid, Todo>, selected_todo: Option<&SelectedTodo>) -> Node<Msg> {
    ul![C!["todo-list"],
        todos.values().map(|todo| {
            let id = todo.id;
            let is_selected = Some(todo.id) == selected_todo.map(|selected_todo| selected_todo.id);

            // These are here just to show the structure of the list items
            // List items should get the class `editing` when editing and `completed` when marked as completed
            li![C![IF!(todo.completed => "completed"), IF!(is_selected => "editing")],
                el_key(&todo.id),
                div![C!["view"],
                input![C!["toggle"], 
                    attrs!{At::Type => "checkbox", At::Checked => todo.completed.as_at_value()},
                    ev(Ev::Change, move |_| Msg::ToggleTodo(id)),
                ],
                    label![&todo.title],
                    button![C!["destroy"],
                        ev(Ev::Click, move |_| Msg::RemoveTodo(id))
                        ],
                ],
                IF!(is_selected => {
                    let selected_todo = selected_todo.unwrap();
                    input![C!["edit"],
                    el_ref(&selected_todo.input_element), 
                    attrs!{At::Value => selected_todo.title},
                    ]
                }),
            ]
        })
    ]
}

// ------ footer ------

fn view_footer(todos: &BTreeMap<Ulid, Todo>, selected_filter: Filter) -> Node<Msg> {
    let completed_count = todos.values().filter(|todo| todo.completed).count();
    let active_count = todos.len() - completed_count;

    footer![C!["footer"],
    // This should be `0 items left` by default
    span![C!["todo-count"],
    strong![active_count],
    format!(" item{} left", if active_count == 1 { "" } else { "s" }),
    ]
    ]

}


// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    
    let root_element = document()
        .get_elements_by_class_name("todoapp")
        .item(0)
        .expect("element with the class `todoapp`");

    App::start(root_element, init, update, view);
}
