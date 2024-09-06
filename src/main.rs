use fltk::{
    button::Button,
    draw,
    enums::{Color, Event, FrameType, Shortcut},
    frame::Frame,
    group::{Pack, Scroll},
    input::Input,
    menu::{MenuBar, MenuFlag, MenuItem},
    prelude::*,
    tree::{Tree, TreeItem},
    window::{DoubleWindow, Window},
    {
        app,
        app::{channel, widget_from_id, Receiver, Sender},
    },
};

use rusqlite::*;
use std::env;
use std::env::current_dir;
use std::fs;
use std::{path, path::PathBuf};

const DB_PATH: &str = "cold_storage.db";

struct AppContext {
    fltk_app: fltk::app::App,
    db: Connection,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

#[derive(Clone)]
pub enum Message {
    ClearSubWindow,
    SearchEntities(TreeItem),
    ClearEntities(TreeItem),
    EntityFrameClicked(String),
}

impl AppContext {
    fn new() -> Self {
        let mut db_path = locate_cold_storage();

        db_path.push(DB_PATH);

        let (a, b) = channel::<Message>();

        Self {
            fltk_app: app::App::default(),
            db: Connection::open(db_path).unwrap(),
            sender: a,
            receiver: b,
        }
    }

    fn construct(&mut self) -> () {
        // create main window
        Window::default()
            .with_size(1280, 720)
            .center_screen()
            .with_label("Entity Content Creator")
            .with_id("main_window");

        let mut menu: MenuBar = MenuBar::default().with_size(1280, 35);
        let _ = menu.add("Options", Shortcut::None, MenuFlag::Normal, menu_options);

        // create a tree on the left to allow selecting creation templates
        let mut tree_object: Tree = Tree::default()
            .with_size(
                300,
                widget_from_id::<DoubleWindow>("main_window")
                    .unwrap()
                    .height(),
            )
            .below_of(&menu, 3)
            .with_id("main_window_tree");

        tree_object.set_show_root(false);
        // requires explicit 'end()' call to any 'group' types so we don't nest within this object
        tree_object.end();

        // create 2nd window that houses the dynamic rebuildable widgets - right side of GUI
        Window::default()
            .with_size(
                {
                    let wind: DoubleWindow = widget_from_id::<DoubleWindow>("main_window").unwrap();
                    wind.width() - tree_object.width()
                },
                widget_from_id::<DoubleWindow>("main_window")
                    .unwrap()
                    .height(),
            )
            .with_pos(
                {
                    let x = tree_object.x() + tree_object.width();
                    x
                },
                tree_object.y(),
            )
            .with_label("Entity Content Creator")
            .with_id("sub_window");

        let mut scroll: Scroll = Scroll::default()
            .with_size(
                widget_from_id::<DoubleWindow>("sub_window")
                    .unwrap()
                    .width(),
                widget_from_id::<DoubleWindow>("sub_window")
                    .unwrap()
                    .height()
                    - 30,
            )
            .with_id("main_window_sub_window_scroll");
        scroll.set_frame(fltk::enums::FrameType::DownBox);
        scroll.end();

        Button::default()
            .with_size(80, 20)
            .below_of(&scroll, 3)
            .with_id("create_new_button")
            .with_label("Create New");

        Button::default()
            .with_size(80, 20)
            .right_of(&widget_from_id::<Button>("create_new_button").unwrap(), 3)
            .with_id("setting_button")
            .with_label("Settings");

        // done adding to the right side of the GUI
        widget_from_id::<DoubleWindow>("sub_window").unwrap().end();

        // done adding to the main window
        widget_from_id::<DoubleWindow>("main_window").unwrap().end();

        let _ = self.load_items_into_tree(fetch_entity_categories(&self.db));
        // omit for testing alternative methods of filling scroll GUI
        //        let tree_sender: Sender<Message> = self.sender.clone();

        build_out_creation_categories(self.sender.clone());
    }

    /*
       fn gateway_to_fill_sub_window(
           &mut self,
           mut previous_coordinates: Option<(i32, i32)>,
           selection: String,
       ) -> () {


           self.add_input_to_window(&mut previous_coordinates);
           ()
       }
    */

    fn event_loop(&mut self) -> Result<(), ()> {
        while self.fltk_app.wait() {
            match self.receiver.recv() {
                Some(Message::ClearSubWindow) => {
                    clear_sub_window_scroll();
                }
                Some(Message::SearchEntities(t)) => {
                    fetch_and_fill_entity_search(&self, t);
                }
                Some(Message::ClearEntities(mut t)) => {
                    clear_entities_from_tree(&mut t);
                }
                Some(Message::EntityFrameClicked(s)) => {
                    if !s.is_empty() {
                        self.build_and_fill_scroll_gui(s);
                    }
                }
                None => {}
            }
        }

        Ok(())
    }

    fn load_items_into_tree(&self, items: Vec<String>) -> () {
        let mut t: Option<Tree> = widget_from_id::<Tree>("main_window_tree");

        // load items into root tree item and close them by pathname
        for x in items {
            match t.as_mut() {
                Some(t) => match t.add(&x[..]) {
                    Some(ti) => match t.item_pathname(&ti) {
                        Ok(ti_pn) => match t.close(&ti_pn[..], false) {
                            Ok(_) => (),
                            Err(_) => (),
                        },
                        Err(_) => (),
                    },
                    None => (),
                },
                None => {}
            }
        }
    }

    fn build_and_fill_scroll_gui(&self, s: String) -> () {
        let eid: String = slice_beginning_of_string(s, ":");

        let mut rs: RecordSet = fetch_entity_information(&self.db, eid);
        build_scroll_gui(&rs);
        fill_scroll_gui(&mut rs);
    }
}

#[derive(Default, Clone)]
struct RecordSet {
    records: Vec<Record>,
    headers: Headers,
}

#[derive(Clone)]
struct Record {
    fields: Vec<SqlData>,
}

impl Record {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

#[derive(Default, Clone)]
struct Headers {
    column_names: Vec<String>,
    column_count: usize,
}

#[derive(Clone)]
enum SqlData {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl ToString for SqlData {
    fn to_string(&self) -> String {
        match self {
            SqlData::Null => String::new(),
            SqlData::Integer(z) => z.to_string(),
            SqlData::Real(z) => z.to_string(),
            SqlData::Text(z) => z.to_string(),
            SqlData::Blob(_) => String::new(),
        }
    }
}

fn main() -> Result<(), ()> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    };
    entry_point()?;
    Ok(())
}

/*
fn load_db_into_tree(
    tree: &mut Tree,
) -> Result<()> {
    let rs = query(
        &Connection::open(DB_PATH).unwrap(),
        "SELECT name FROM sqlite_schema WHERE type ='table' AND name NOT LIKE 'sqlite_%';",
        &[]);
    for x in rs.records {
        for y in x.fields {
            tree.add(&y.to_string()[..]);
        }
    }
    Ok(())
}
 */

fn query(
    sqlite_connection: &Connection,
    query_str: &str,
    params: &[(&str, &dyn ToSql)],
) -> RecordSet {
    let mut rs: RecordSet = RecordSet::default();

    let stmt: Result<CachedStatement, Error> = sqlite_connection.prepare_cached(query_str);

    // execute the query if the statement successfully prepared
    match stmt {
        Ok(mut r) => {
            let col_count: usize = r.column_count();
            let col_names: Vec<&str> = r.column_names();

            rs.headers.column_count = col_count;
            for x in col_names {
                rs.headers.column_names.push(String::from(x));
            }

            // when iterating through the rows later, this is the function that each row will be passed into
            let mut rows = r
                .query_map(params, |row| {
                    let mut v: Vec<rusqlite::types::Value> =
                        Vec::with_capacity(rs.headers.column_count);

                    for ind in 0..col_count {
                        v.push(row.get(ind).unwrap());
                    }
                    Ok(v)
                })
                .unwrap();

            while let Some(r) = rows.next() {
                match r {
                    Ok(e) => {
                        let mut new_row: Record = Record::new();
                        for ind in 0..e.len() {
                            // get each field in this row and put it in the gd_recordset
                            // converting each SQLite value to a Godot equivalent
                            match &e[ind] {
                                types::Value::Null => new_row.fields.push(SqlData::Null),
                                types::Value::Integer(v_i64) => {
                                    new_row.fields.push(SqlData::Integer(v_i64.clone()))
                                }
                                types::Value::Real(v_f64) => {
                                    new_row.fields.push(SqlData::Real(v_f64.clone()))
                                }
                                types::Value::Text(v_string) => {
                                    new_row.fields.push(SqlData::Text(v_string.clone()))
                                }
                                types::Value::Blob(v_vec_u8) => {
                                    new_row.fields.push(SqlData::Blob(
                                        Vec::new(), /* Vec::from(v_vec_u8[..])) */
                                    ))
                                }
                            }
                        }
                        rs.records.push(new_row);
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    };

    rs
}

/* This entire function is just stripped prototype code (that worked) from 'fn main()'
// This code won't work on its own, need to provide a reference to Fltk App and resolve the image file loads
// as well as the img struct conversions
fn load_image(
) -> () {

    let mut frame = Frame::default().with_size(360, 260).center_of(&wind);
    let mut loaded_img: image::JpegImage = image::JpegImage::load("..\\test_data\\312-1.JPG").unwrap();
    let mut new_img: image::BmpImage;
    unsafe { new_img =  loaded_img.into_image::<image::BmpImage>(); }

    let mut png_img_file = File::open("C:\\Users\\goomb\\Downloads\\Screenshot 2024-06-15 141905.png").unwrap();
    let bytes: Vec<u8> = png_img_file.bytes().map(|x| x.unwrap()).collect();

    let row_id = db.last_insert_rowid();

    let mut blob = db.blob_open(DatabaseName::Main, "test_table", "content", row_id, false)?;

    blob.seek(SeekFrom::Start(0)).unwrap_or(0);

    let mut buf = [0u8; 242434];

    let bytes_read = blob.read(&mut buf[..]).unwrap_or(0);
    println!("bytes read: {}", bytes_read);


    println!("png img about to load");
    let mut new_png_img = fltk::image::PngImage::from_data(&bytes[..]);
    println!("png img loaded");

    frame.draw(move |f| {
        println!("frame drawing");
        new_img.scale(f.w(), f.h(), true, true);
        println!("frame scaled");
        new_img.draw(f.x() + 40, f.y(), f.w(), f.h());
        println!("frame drewed");
    });


}
*/

fn entry_point() -> Result<(), ()> {
    let mut f: AppContext = AppContext::new();
    f.construct();

    match widget_from_id::<DoubleWindow>("main_window") {
        Some(mut mainwindow) => {
            mainwindow.show();
            f.event_loop()
        }
        None => Ok(()),
    }
}

fn build_scroll_gui(rs: &RecordSet) -> () {
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");

    match scroll {
        Some(mut s) => {
            clear_sub_window_scroll();
            let mut previous_coordinates: Option<(i32, i32)> = None;

            let mut child_text_boxes: Vec<String> = Vec::new();
            for header in &rs.headers.column_names {
                child_text_boxes.push(add_input_to_scroll(
                    &mut s,
                    &mut previous_coordinates,
                    &header[..],
                    &header[..],
                ));
            }

            // make a bunch of test children
            for x in 0..5 {
                let y: &str = &x.to_string()[..];
                child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, y, y));
            }

            autolayout_subwindow_scrollbox_gui(child_text_boxes);

            s.redraw();
        }
        None => (),
    }
    ()
}

fn fill_scroll_gui(rs: &mut RecordSet) -> () {
    let fields: Vec<String> = match rs.records.pop() {
        Some(r) => r.fields.iter().map(|x| x.to_string()).collect(),
        None => return (),
    };

    for index in 0..rs.headers.column_count {
        match rs.headers.column_names.get(index) {
            Some(header) => match widget_from_id::<Input>(&header[..]) {
                Some(mut input_box) => input_box.set_value(&fields.get(index).unwrap()),
                None => (),
            },
            None => (),
        }
    }

    ()
}

fn autolayout_subwindow_scrollbox_gui(child_boxes: Vec<String>) -> () {
    let mut largest: i32 = 0;

    for x in &child_boxes {
        find_largest_label(
            &mut largest,
            widget_from_id::<Input>(&x[..]).unwrap().measure_label(),
        )
    }

    for x in &child_boxes {
        let mut child: Input = widget_from_id::<Input>(&x[..]).unwrap();
        child.set_pos(largest + 3, child.y());
    }

    ()
}

fn add_input_to_scroll(
    widget: &mut Scroll,
    datum: &mut Option<(i32, i32)>,
    label: &str,
    id: &str,
) -> String {
    let coords: (i32, i32) = match datum.as_ref() {
        Some(x) => x.clone(),
        None => (0, 0),
    };

    widget.begin();

    let mut input_one: Input = Input::default()
        .with_size(180, 30)
        .with_pos(coords.0, coords.1)
        .with_label(label)
        .with_id(id);

    // align X to the previous input + the label width + padding from the edge of the scrollbox
    input_one.set_pos(
        coords.0 + input_one.measure_label().0 + 3,
        coords.1 + input_one.height(),
    );

    widget.add(&input_one);
    widget.end();

    *datum = Some((coords.0, coords.1 + input_one.height()));

    String::from(id)
}

fn clear_sub_window_scroll() -> () {
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");
    match scroll {
        Some(mut w) => {
            w.clear();
            w.redraw();
        }
        None => {}
    }

    ()
}

fn find_largest_label(largest: &mut i32, label: (i32, i32)) -> () {
    if *largest < label.0 {
        *largest = label.0;
    }

    ()
}

fn slice_end_of_string(s: String, delim: &str) -> String {
    let mut x: Vec<&str> = s.split(delim).collect();

    match x.pop() {
        Some(y) => y.to_string(),
        None => String::new(),
    }
}

fn slice_beginning_of_string(s: String, delim: &str) -> String {
    let mut x: Vec<&str> = s.split(delim).collect();
    x.reverse();
    match x.pop() {
        Some(y) => y.to_string(),
        None => String::new(),
    }
}

fn locate_cold_storage() -> PathBuf {
    let t: Result<PathBuf, _> = current_dir();

    match t {
        Ok(mut p) => {
            p.push("test_data");
            p
        }
        Err(_) => PathBuf::new(),
    }
}

fn print_tree_items(tree: &mut Tree) -> () {
    match tree.get_items() {
        Some(v) => {
            for i in v {
                match tree.item_pathname(&i) {
                    Ok(s) => println!("item pathname: {}", s),
                    Err(_) => println!("Can't find item pathname"),
                }
            }
        }
        None => {
            println!("No items in tree");
        }
    };
}

fn print_all_tree_items() -> () {
    let tree_object: Tree = widget_from_id::<Tree>("main_window_tree").unwrap();
    match tree_object.find_item("Mob") {
        Some(ti_mob) => {
            for i in 0..=ti_mob.children() {
                match ti_mob.child(i) {
                    Some(ti) => {
                        println!("tree_item: x: {}, y: {}", ti.x(), ti.y());
                        match ti.try_widget() {
                            Some(b) => println!(
                                "input pos: x:{}, y:{} -- input size: w: {}, h: {}",
                                b.x(),
                                b.y(),
                                b.w(),
                                b.h()
                            ),
                            None => {
                                println!("no button inner widget found")
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        None => {}
    }
}

fn _on_search_fill_category_with_items() -> () {
    let mut tree_object: Tree = widget_from_id::<Tree>("main_window_tree").unwrap();
    for x in 0..=20 {
        let mut s: String = String::from("Mob");
        s.push_str("/Mob");
        s.push_str(&x.to_string()[..]);

        let tree_item = TreeItem::new(&tree_object, &x.to_string()[..]);

        tree_object.add_item(&s[..], &tree_item);
        /*
                let mut b: Input = Input::default()
                    .with_size(120, tree_item.label_h())
                    .with_label("test")
                    .with_id(&s[..]);

                b.set_value("WHERE AM I");
        */
    }
}

fn build_out_creation_categories(app_sender: Sender<Message>) -> () {
    match widget_from_id::<Tree>("main_window_tree") {
        Some(mut tree) => {
            tree.begin();
            let tree_root: TreeItem = tree.root().unwrap();
            let count_of_children: i32 = tree_root.children();
            for x in 0..count_of_children {
                let child: TreeItem = tree_root.child(x).unwrap();
                let child_pathname: String = tree.item_pathname(&child).unwrap();
                let mut new_tree_item: TreeItem = TreeItem::new(&tree, "quick_search");

                new_tree_item.draw_item_content(|ti, render| {
                    let dims: (i32, i32, i32, i32) =
                        (ti.label_x(), ti.label_y(), ti.label_w(), ti.label_h());
                    // If the widget is visible 'render'
                    if render {
                        match ti.try_widget() {
                            Some(pack) => {
                                // fetch the nested widget out of TreeItem and cast it to a Pack
                                let mut pack: Pack = unsafe { pack.into_widget::<Pack>() };
                                pack.set_pos(dims.0, dims.1);
                                pack.set_size(dims.2, dims.3);

                                let mut dims: (i32, i32, i32, i32) =
                                    (pack.x(), pack.y(), pack.w(), pack.h());
                                pack.set_color(Color::Gray0);

                                for i in 0..pack.children() {
                                    match pack.child(i) {
                                        Some(mut child) => {
                                            child.set_pos(dims.0, dims.1);
                                            child.set_size(dims.2 - 100, (dims.3 / 4));
                                            dims.1 += dims.3;
                                        }
                                        None => {}
                                    }
                                }
                            }
                            None => {}
                        }
                    };
                    let (label_w, _): (i32, i32) = draw::measure(&ti.label().unwrap()[..], true);
                    return dims.0 + label_w;
                });

                let mut new_path: String = child.label().unwrap();
                new_path.push_str("/");
                new_path.push_str("quick_search");

                match tree.add_item(&new_path, &new_tree_item) {
                    Some(mut ti) => {
                        ti.set_label_size(ti.label_size() * 4);

                        let mut hg: Pack = Pack::new(
                            ti.label_x(),
                            ti.label_h(),
                            ti.label_w(),
                            ti.label_h() * 2,
                            "",
                        );

                        hg.set_frame(FrameType::ThinUpFrame);

                        hg.add(&Frame::new(
                            hg.x(),
                            hg.y(),
                            hg.width(),
                            hg.height(),
                            "Quick Lookup",
                        ));

                        hg.add(&Input::new(hg.x(), hg.y(), hg.width(), hg.height(), ""));
                        let mut button_id: String = String::from(child_pathname.clone());
                        button_id.push_str("_search_button");

                        let mut button: Button =
                            Button::new(hg.x(), hg.y(), hg.width(), hg.height(), "Search")
                                .with_id(&button_id[..]);

                        let tree_item: TreeItem = ti.clone();
                        let app_sender_clone: Sender<Message> = app_sender.clone();

                        button.set_callback(move |_| {
                            let tree_item: TreeItem = tree_item.clone();
                            app_sender_clone.send(Message::SearchEntities(tree_item));
                        });

                        hg.add(&button);

                        button_id = String::from(child_pathname.clone());
                        button_id.push_str("_clear_button");

                        button = Button::new(hg.x(), hg.y(), hg.width(), hg.height(), "Clear");

                        let tree_item: TreeItem = ti.clone();
                        let app_sender_clone: Sender<Message> = app_sender.clone();

                        button.set_callback(move |_| {
                            let tree_item: TreeItem = tree_item.clone();
                            app_sender_clone.send(Message::ClearEntities(tree_item));
                        });

                        hg.add(&button);

                        hg.end();
                        ti.set_widget(&hg);
                    }
                    None => {
                        println!("Failed to add tree item");
                    }
                }
            }
            tree.end();
        }
        None => {}
    }
}

fn fetch_and_fill_entity_search(c: &AppContext, t: TreeItem) -> () {
    let p: Pack = match t.try_widget() {
        Some(w) => {
            let wi: Option<Pack> = fltk::prelude::WidgetBase::from_dyn_widget(&w);
            match wi {
                Some(p) => p,
                None => return,
            }
        }
        None => return,
    };

    let b: Input = match p.child(1) {
        Some(b) => {
            let bi: Option<Input> = fltk::prelude::WidgetBase::from_dyn_widget(&b);
            match bi {
                Some(wi) => wi,
                None => return,
            }
        }
        None => return,
    };

    let input_value: String = b.value();
    let parent: TreeItem = match t.parent() {
        Some(tree_item) => tree_item,
        None => return (),
    };

    let selected_string: String = parent.label().unwrap_or(String::new());

    let x: String = slice_beginning_of_string(selected_string, "/");

    let r: RecordSet = match input_value.len() {
        0 => fetch_all_entity_base_data(&c.db, x),
        _ => fetch_specific_entity_base_data(input_value, &c.db, x),
    };

    fill_tree_with_entity_data(r, t, c);

    ()
}

fn fetch_entity_categories(conn: &Connection) -> Vec<String> {
    let rs = query(conn, "SELECT name FROM entity_core_types;", &[]);

    let v: Vec<String> = rs
        .records
        .iter()
        .map(|x| match x.fields.len() {
            0 => String::new(),
            _ => x.fields[0].to_string(),
        })
        .collect();
    v
}

// create a SQL statement to build out the Scroll widget with entity information
fn fetch_entity_information(conn: &Connection, eid: String) -> RecordSet {
    let x: &[(&str, &dyn ToSql)] = named_params! { ":eid": eid };
    let rs: RecordSet = query(conn, "SELECT 'e'.'entity_base_id', 'e'.'name', 'e'.'entity_core_type_id', 'e'.'entity_sub_type_id' FROM 'entity_base_definitions' as 'e' WHERE 'e'.'entity_base_id' = :eid;", &x);

    rs
}

fn fetch_all_entity_base_data(conn: &Connection, ect: String) -> RecordSet {
    let x: &[(&str, &dyn ToSql)] = named_params! { ":ect": ect };
    let rs = query(
        conn,
        "SELECT 'e'.'entity_base_id', 'e'.'name' FROM 'entity_base_definitions' as 'e' WHERE 'e'.'entity_core_type_id' IN (SELECT 'e'.'entity_core_type_id' FROM 'entity_core_types' as 'e' WHERE 'e'.'name' = :ect);",
        &x,
    );

    rs
}

fn fetch_specific_entity_base_data(v: String, conn: &Connection, ect: String) -> RecordSet {
    let x: &[(&str, &dyn ToSql)] = named_params! { ":ead": v, ":ect": ect };
    let rs = query(
        conn,
        "SELECT 'e'.'entity_base_id', 'e'.'name' FROM 'entity_base_definitions' as 'e' WHERE 'e'.'entity_base_id' = :ead AND 'e'.'entity_core_type_id' IN (SELECT 'e'.'entity_core_type_id' FROM 'entity_core_types' as 'e' WHERE 'e'.'name' = :ect)",
        &x,
    );

    rs
}

fn fill_tree_with_entity_data(rs: RecordSet, mut ti: TreeItem, c: &AppContext) -> () {
    let mut t: Tree = match ti.tree() {
        Some(x) => x,
        None => return (),
    };

    clear_entities_from_tree(&mut ti);
    t.begin();

    for row in rs.records {
        let fields: (String, String) = (row.fields[0].to_string(), row.fields[1].to_string());

        let mut new_ti_path: String = String::new();
        let ti_path: String = t.item_pathname(&ti).unwrap();
        new_ti_path.push_str(&ti_path[..]);
        new_ti_path.push('/');

        let mut new_ti_label: String = String::new();
        new_ti_label.push_str(&fields.0.to_string()[..]);
        new_ti_label.push(':');
        new_ti_label.push_str(&fields.1.to_string()[..]);
        new_ti_path.push_str(&new_ti_label[..]);

        let mut new_tree_item: TreeItem = TreeItem::new(&t, &new_ti_label[..]);

        new_tree_item.draw_item_content(|tree_item, render| {
            let dims: (i32, i32, i32, i32) = (
                tree_item.label_x(),
                tree_item.label_y(),
                tree_item.label_w(),
                tree_item.label_h(),
            );

            if render {
                match tree_item.try_widget() {
                    Some(frame) => {
                        let mut frame: Frame = unsafe { frame.into_widget::<Frame>() };
                        frame.set_pos(dims.0, dims.1);
                        frame.set_size(dims.2, dims.3);
                    }
                    None => (),
                }
            }

            let (label_w, _): (i32, i32) = draw::measure(&tree_item.label().unwrap()[..], render);
            return dims.0 + label_w;
        });

        match t.add_item(&new_ti_path[..], &new_tree_item) {
            Some(mut ti) => {
                let mut f: Frame = Frame::new(
                    ti.label_x(),
                    ti.label_y(),
                    ti.label_w(),
                    ti.label_h(),
                    &ti.label().unwrap()[..],
                );

                ti.set_widget(&f);

                let sender: Sender<_> = c.sender.clone();

                f.handle(move |f_self, event| match event {
                    Event::Released => {
                        sender.send(Message::EntityFrameClicked(f_self.label()));
                        true
                    }
                    _ => false,
                });
            }
            None => (),
        };
    }
    t.end();
}

fn clear_entities_from_tree(ti: &mut TreeItem) -> () {
    let mut t: Tree;

    match ti.tree() {
        Some(x) => t = x,
        None => return (),
    }

    for i in 0..ti.children() {
        match ti.child(i) {
            //            Some(mut child_ti) => child_ti.delete(),
            Some(child_ti) => {
                match child_ti.try_widget() {
                    Some(w) => {
                        let wi: Option<Frame> = fltk::prelude::WidgetBase::from_dyn_widget(&w);
                        match wi {
                            Some(f) => WidgetBase::delete(f),
                            None => (),
                        }
                    }
                    None => (),
                };
            }
            None => (),
        }
    }
    ti.clear_children();

    t.redraw();

    return ();
}

fn menu_options(_: &mut MenuBar) -> () {
    let menu: MenuItem = MenuItem::new(&["Regen Enums"]);
    if app::event_mouse_button() == app::MouseButton::Left {
        let coords = app::event_coords();
        match menu.popup(coords.0, coords.1) {
            None => (),
            Some(v) => match &v.label().unwrap()[..] {
                "Regen Enums" => regen_enums(),
                _ => println!("Failed to match in menu_options"),
            },
        }
    }
}

fn regen_enums() -> () {
    // untested workflow, may need to redo the workflow for capitalizing
    let mut enum_package: String = String::new();
    // println!("entering regen_enums");
    let mut db_path = locate_cold_storage();

    db_path.push(DB_PATH);

    let db: Connection = match Connection::open(db_path) {
        Ok(c) => {
            println!("Successfully opened connection");
            c
        }
        Err(_) => {
            println!("failed to open connection");
            return ();
        }
    };
    let rs_enum_tables: RecordSet = fetch_enum_tables(&db);
    //print_recordset_debug(rs_enum_tables.clone());
    for mut enum_rows in rs_enum_tables.records {
        match enum_rows.fields.pop() {
            Some(f) => {
                // println!("looping through enum table names");
                // "entity_actions" -> Vec["entity","actions"]
                let enum_lowercase: Vec<String> = f
                    .to_string()
                    .split("_")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                let mut enum_capitalized: Vec<String> = Vec::new();

                // "entity" -> "Entity"
                for x in &enum_lowercase {
                    let first: String = x[0..1].to_uppercase();
                    let rest: String = String::from(&x[1..]);
                    enum_capitalized.push(String::from(Vec::from([first, rest]).join("")));
                }
                let rs_enum_data: RecordSet = fetch_enum_values_from_table(&db, f.to_string());
                // begin adding new enum type
                enum_package.push_str("\nenum ");
                //print_recordset_debug(rs_enum_data);
                enum_package.push_str(&String::from(enum_capitalized.join(""))[..]);
                enum_package.push_str("{\n");

                for row in rs_enum_data.records {
                    let mut fields: Vec<SqlData> = row.fields;
                    let type_name: String = fields.pop().unwrap().to_string();
                    let type_value: String = fields.pop().unwrap().to_string();
                    enum_package.push_str("\t");
                    enum_package.push_str(&type_name[..]);
                    enum_package.push_str(" = ");
                    enum_package.push_str(&type_value[..]);
                    enum_package.push_str(",\n");
                }
                enum_package.push_str("}\n");
            }
            None => (),
        }
    }
    let _ = write_enums_to_file(enum_package);
}

fn fetch_enum_tables(db: &Connection) -> RecordSet {
    let sql: String = String::from("SELECT 'e'.'table' FROM 'enums' as 'e';");
    query(&db, &sql[..], &[])
}

fn fetch_enum_values_from_table(db: &Connection, t: String) -> RecordSet {
    let sql: String = Vec::from(&[
        "SELECT 'x'.'_rowid_', 'x'.'name' FROM '",
        &t[..],
        "' as 'x' ORDER BY 'x'.'_rowid_' ASC;",
    ])
    .join("");
    //println!("sql is: {}", &sql[..]);
    query(&db, &sql[..], &[])
}

fn print_recordset_debug(r: RecordSet) -> () {
    //println!("headers: {}", r.headers.column_count);
    for x in r.headers.column_names {
        println!("header name: {}", x);
    }

    //println!("records: {}", r.records.len());
    for x in r.records {
        let x_displayed: Vec<String> = x
            .fields
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>();
        println!(" rec: {:?}", x_displayed);
    }
}

fn write_enums_to_file(s: String) -> std::io::Result<()> {
    fs::write("enums.gd", s.as_bytes())?;
    Ok(())
}
