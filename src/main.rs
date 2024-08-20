use fltk::{
    button::Button,
    draw,
    enums::{Color, FrameType},
    frame::Frame,
    group::{Pack, Scroll},
    input::Input,
    prelude::*,
    tree::{Tree, TreeItem, TreeReason},
    window::{DoubleWindow, Window},
    {
        app,
        app::{channel, widget_from_id, Receiver, Sender},
    },
};
use rusqlite::*;
use std::env::current_dir;
use std::path::PathBuf;

const CREATION_CATEGORIES: [&str; 3] = ["Mob", "Item", "Static"];
const DB_PATH: &str = "cold_storage.db";

struct AppContext {
    fltk_app: fltk::app::App,
    db: Connection,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

#[derive(Clone)]
pub enum Message {
    TreeSelection(String),
    ClearSubWindow,
    SearchEntities(TreeItem),
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
            .with_size(800, 600)
            .center_screen()
            .with_label("Entity Content Creator")
            .with_id("main_window");

        // create a tree on the left to allow selecting creation templates
        let mut tree_object: Tree = Tree::default()
            .with_size(
                300,
                widget_from_id::<DoubleWindow>("main_window")
                    .unwrap()
                    .height(),
            )
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

        let _ = self.load_items_into_tree(Vec::from(CREATION_CATEGORIES));

        let tree_sender: Sender<Message> = self.sender.clone();

        tree_object.set_callback({
            move |t| match t.callback_reason() {
                TreeReason::None => {}
                TreeReason::Selected => {
                    tree_selected_callback(&tree_sender, &t);
                }
                TreeReason::Deselected => {}
                TreeReason::Reselected => {}
                TreeReason::Opened => {}
                TreeReason::Closed => {}
                TreeReason::Dragged => {}
            }
        });

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
                Some(Message::TreeSelection(s)) => {
                    match_tree_selection(s);
                }
                Some(Message::ClearSubWindow) => {
                    clear_sub_window_scroll();
                }
                Some(Message::SearchEntities(t)) => {
                    fetch_and_fill_entity_search(&self.db, t);
                }
                None => {}
            }
        }

        Ok(())
    }

    fn load_items_into_tree(&self, items: Vec<&str>) -> () {
        let mut t: Option<Tree> = widget_from_id::<Tree>("main_window_tree");

        for x in items {
            match t.as_mut() {
                Some(t) => {
                    t.add(x);
                }
                None => {}
            }
        }
    }
}

#[derive(Default)]
struct RecordSet {
    records: Vec<Record>,
    headers: Headers,
}

struct Record {
    fields: Vec<SqlData>,
}

impl Record {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

#[derive(Default)]
struct Headers {
    column_names: Vec<String>,
    column_count: usize,
}

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

fn match_tree_selection(s: String) -> () {
    match &s[..] {
        //        "Mob" => build_mob_gui(),
        "Mob" => {
            println!("entering 'mob' of match_tree_selection");
            _on_search_fill_category_with_items();
        }
        "Item" => build_item_gui(),
        "Static" => build_static_gui(),
        _ => {
            println!("entering _ of match_tree_selection");
        }
    };
}

fn build_mob_gui() -> () {
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");

    match scroll {
        Some(mut s) => {
            let mut previous_coordinates: Option<(i32, i32)> = None;

            let mut child_text_boxes: Vec<String> = Vec::new();

            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Entity ID",
                "entity_id",
            ));
            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Entity Name",
                "entity_name",
            ));

            // make a bunch of test children
            for x in 0..100 {
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

fn build_item_gui() -> () {
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");

    match scroll {
        Some(mut s) => {
            let mut previous_coordinates: Option<(i32, i32)> = None;

            let mut child_text_boxes: Vec<String> = Vec::new();

            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Entity ID",
                "entity_id",
            ));
            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Entity Name",
                "entity_name",
            ));
            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Item Type",
                "item_type",
            ));

            autolayout_subwindow_scrollbox_gui(child_text_boxes);

            s.redraw();
        }
        None => (),
    }

    ()
}

fn build_static_gui() -> () {
    match widget_from_id::<Scroll>("main_window_sub_window_scroll") {
        Some(mut s) => {
            let mut previous_coordinates: Option<(i32, i32)> = None;

            let mut child_text_boxes: Vec<String> = Vec::new();

            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Entity ID",
                "entity_id",
            ));
            child_text_boxes.push(add_input_to_scroll(
                &mut s,
                &mut previous_coordinates,
                "Entity Name",
                "entity_name",
            ));

            autolayout_subwindow_scrollbox_gui(child_text_boxes);

            s.redraw();
        }
        None => (),
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

fn slice_end_of_string(s: String) -> String {
    let mut x: Vec<&str> = s.split("/").collect();

    match x.pop() {
        Some(y) => y.to_string(),
        None => String::new(),
    }
}

fn tree_selected_callback(tree_sender: &Sender<Message>, t: &Tree) -> () {
    match t.callback_item() {
        Some(tree_item) => match t.item_pathname(&tree_item) {
            Ok(t) => {
                tree_sender.send(Message::TreeSelection(slice_end_of_string(t)));
            }
            Err(_) => {}
        },
        None => {
            clear_sub_window_scroll();
        }
    }
}

fn locate_cold_storage() -> PathBuf {
    // need to understand where current_dir() is pointing to at runtime
    //    let t: Result<PathBuf, _> = current_dir();
    let t: Result<PathBuf, Error> = Ok(PathBuf::new());

    match t {
        Ok(mut p) => {
            p.push("C:\\Rust_Dev\\EntityCreator");
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

fn wipe_tree_items() -> () {
    match widget_from_id::<Tree>("main_window_tree") {
        Some(tree) => match tree.get_items() {
            Some(v_ti) => {}
            None => {}
        },
        None => {
            println!("unable to find tree object");
        }
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
                    println!("drawing custom item content");
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
                        let mut button_id: String = String::from(child_pathname);
                        button_id.push_str("_search_button");

                        let mut button: Button =
                            Button::new(hg.x(), hg.y(), hg.width(), hg.height(), "Search")
                                .with_id(&button_id[..]);

                        let tree_item: TreeItem = ti.clone();
                        let app_sender_clone: Sender<Message> = app_sender.clone();

                        button.set_callback(move |_| {
                            let tree_item: TreeItem = tree_item.clone();
                            app_sender_clone.send(Message::SearchEntities(tree_item))
                        });

                        hg.add(&button);

                        button_id = String::from(child_pathname);
                        button_id.push_str("_cancel_button");

                        button = Button::new(hg.x(), hg.y(), hg.width(), hg.height(), "Clear");

                        let tree_item: TreeItem = ti.clone();
                        let app_sender_clone: Sender<Message> = app_sender.clone();

                        button.set_callback(move |_| {
                            let tree_item: TreeItem = tree_item.clone();
                            app_sender_clone.send(Message::SearchEntities(tree_item))
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

fn fetch_and_fill_entity_search(c: &Connection, t: TreeItem) -> () {
    let r: RecordSet = fetch_entity_base_data(c);
    fill_tree_with_entity_data(r, t);

    ()
}

fn fetch_entity_base_data(conn: &Connection) -> RecordSet {
    let rs = query(
        conn,
        "SELECT 'e'.'entity_base_id', 'e'.'name' FROM 'entity_base_definitions' as 'e';",
        &[],
    );

    rs
}

fn fill_tree_with_entity_data(rs: RecordSet, ti: TreeItem) -> () {
    let mut t: Tree = ti.tree().unwrap();
    println!("entering fill_tree_with_entity_data");
    t.clear_children(&ti);

    t.redraw();
    for row in rs.records {
        println!("looping thru rows");
        let fields: (String, String) = (row.fields[0].to_string(), row.fields[1].to_string());

        let mut new_ti_path: String = String::new();
        let ti_path: String = t.item_pathname(&ti).unwrap();
        new_ti_path.push_str(&ti_path[..]);
        new_ti_path.push('/');
        new_ti_path.push_str(&fields.0.to_string()[..]);
        new_ti_path.push(':');
        new_ti_path.push_str(&fields.1.to_string()[..]);
        t.add(&new_ti_path[..]);

        ()
    }
}
