use rusqlite::*;
use fltk::{
    prelude::*, 
    window::{ Window, DoubleWindow },
    { app, 
        app::{ widget_from_id,
            Sender,
            Receiver,
            channel,
        }
    }, 
    group::Scroll,
    tree::{ Tree, TreeItem, TreeReason },
    input::Input,
    button::Button,
};


const CREATION_CATEGORIES: [&str; 3] = [ "Mob", "Item", "Static" ];
const DB_PATH: &str = ".\\test_data\\cold_storage.db";


struct AppContext{
    fltk_app: fltk::app::App,
    db: Connection,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

#[derive(Clone)]
pub enum Message {
    TreeSelection(String),
    ClearSubWindow,
}


impl AppContext {
    fn new() -> Self {
        let (a, b) = channel::<Message>();
        Self {
            fltk_app: app::App::default(),
            db: Connection::open(DB_PATH).unwrap(),
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
        Tree::default()
            .with_size(200, widget_from_id::<DoubleWindow>("main_window")
                .unwrap()
                .height()
            )
            .with_label("tree")
            .with_id("main_window_tree");

        println!("{:?}", widget_from_id::<Tree>("main_window_tree").unwrap().select_mode());

        // requires explicit 'end()' call to any 'group' types so we don't nest within this object
        widget_from_id::<Tree>("main_window_tree")
            .unwrap()
            .end();

        // create 2nd window that houses the dynamic rebuildable widgets - right side of GUI
        Window::default()
            .with_size( 
                {
                    let wind: DoubleWindow = widget_from_id::<DoubleWindow>("main_window")
                        .unwrap();
                    let tree: Tree = widget_from_id::<Tree>("main_window_tree")
                        .unwrap();
                    wind.width() - tree.width()
                },
                widget_from_id::<DoubleWindow>("main_window")
                    .unwrap()
                    .height()
            )
            .with_pos(
                {
                    let tree: Tree = widget_from_id::<Tree>("main_window_tree").unwrap();
                    let x = tree.x() + tree.width();
                    x
                },
                widget_from_id::<Tree>("main_window_tree").unwrap().y()
            )
            .with_label("Entity Content Creator")
            .with_id("sub_window");
    
        let mut scroll: Scroll = Scroll::default()
            .with_size(
                widget_from_id::<DoubleWindow>("sub_window").unwrap().width(),
                widget_from_id::<DoubleWindow>("sub_window").unwrap().height() - 30)
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
        
        let mut tree: Option<Tree> = widget_from_id::<Tree>("main_window_tree");
        match t.as_mut() {
            Some(t) => {
                for x in 0..=20 {
            
                    let t: TreeItem = TreeItem::new(&t, &x.to_string()[..]);
        
                    add_child_treeitem(t, p, &tree);
                }        

            },
            None => { },
        }
    

        let tree_sender: Sender<Message> = self.sender.clone();

        widget_from_id::<Tree>("main_window_tree")
            .unwrap()
            .set_callback({move |t| {
                match t.callback_reason() {
                    TreeReason::None => {
                    },
                    TreeReason::Selected => { 
                        tree_selected_callback(&tree_sender, &t);
                    },
                    TreeReason::Deselected => { 
                    },
                    TreeReason::Reselected => {
                    },
                    TreeReason::Opened => {
                    },
                    TreeReason::Closed => {
                    },
                    TreeReason::Dragged => {
                    },
                }
            }
        });

        ()
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
                },
                Some(Message::ClearSubWindow) => {
                    clear_sub_window_scroll();
                }
                None => { },
            }
        }

        Ok(())
    }

    fn load_items_into_tree(
        &self,
        items: Vec<&str>,
    ) -> () {
        let mut t: Option<Tree> = widget_from_id::<Tree>("main_window_tree");

        for x in items {

            match t.as_mut() {
                Some(t) => {
                    t.add(x);
                },
                None => { },
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
        Self {
            fields: Vec::new(),
        }
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
    fn to_string(
        &self,
    ) -> String {
        match self {
            SqlData::Null => String::new(),
            SqlData::Integer(z) => z.to_string(),
            SqlData::Real(z) => z.to_string(),
            SqlData::Text(z) => z.to_string(),
            SqlData::Blob(_) => String::new(),
        }
    }
}


fn main(
) -> Result<(), ()> {

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
            let mut rows = r.query_map(params, |row| {
                let mut v : Vec<rusqlite::types::Value> = Vec::with_capacity(rs.headers.column_count);

                for ind in 0..col_count  {
                    v.push(row.get(ind).unwrap());
                }
                Ok(v)
            }).unwrap();

            while let Some(r) = rows.next() {
                match r {
                    Ok(e) => {
                        let mut new_row: Record = Record::new();
                        for ind in 0..e.len() {

                            // get each field in this row and put it in the gd_recordset
                            // converting each SQLite value to a Godot equivalent
                            match &e[ind] {

                                types::Value::Null =>                     { new_row.fields.push( SqlData::Null ) },
                                types::Value::Integer(v_i64) =>     { new_row.fields.push( SqlData::Integer(v_i64.clone()) ) },
                                types::Value::Real(v_f64) =>        { new_row.fields.push( SqlData::Real(v_f64.clone()) ) },
                                types::Value::Text(v_string) =>  { new_row.fields.push( SqlData::Text(v_string.clone()) ) },
                                types::Value::Blob(v_vec_u8) => { new_row.fields.push( SqlData::Blob( Vec::new() /* Vec::from(v_vec_u8[..])) */ ) ) },
                            }
                        }
                        rs.records.push(new_row);
                    },
                    Err(_) => {
                    },
                }
            }
        },
        Err(_) => { 
        },
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

    let mut mainwindow: Window = widget_from_id::<DoubleWindow>("main_window").unwrap();
    mainwindow.show();

    f.event_loop()
}


fn match_tree_selection(
    s: String,
) -> () {
    println!("match tree selection");
    match &s[..] {
        "Mob" => build_mob_gui(),
        "Item" => build_item_gui(),
        "Static" => build_static_gui(),
        _ => {},
    };

    ()
}

fn build_mob_gui(
) -> () {
    
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");
    
    match scroll {
        Some(mut s) => {
            let mut previous_coordinates: Option<(i32, i32)> = None;

            let mut child_text_boxes: Vec<String> = Vec::new();

            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Entity ID", "entity_id"));
            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Entity Name", "entity_name"));

            
            // make a bunch of test children
            for x in 0..100 {
                let y: &str = &x.to_string()[..];
                child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, y, y ));
            }

            autolayout_subwindow_scrollbox_gui(child_text_boxes);

            s.redraw();
        },
        None => (),
    }
    ()
}

fn build_item_gui(
) -> () {
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");
    
    match scroll {
        Some(mut s) => {
            let mut previous_coordinates: Option<(i32, i32)> = None;
            
            let mut child_text_boxes: Vec<String> = Vec::new();

            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Entity ID", "entity_id"));
            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Entity Name", "entity_name"));
            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Item Type", "item_type"));

            autolayout_subwindow_scrollbox_gui(child_text_boxes);

            s.redraw();
        },
        None => (),
    }
    
    ()
}

fn build_static_gui(
) -> () {
        
    match widget_from_id::<Scroll>("main_window_sub_window_scroll") {
        Some(mut s) => {
            let mut previous_coordinates: Option<(i32, i32)> = None;

            let mut child_text_boxes: Vec<String> = Vec::new();

            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Entity ID", "entity_id"));
            child_text_boxes.push(add_input_to_scroll(&mut s, &mut previous_coordinates, "Entity Name", "entity_name"));

            autolayout_subwindow_scrollbox_gui(child_text_boxes);

            s.redraw();
        },
        None => (),
    }
    
    ()
}

fn autolayout_subwindow_scrollbox_gui(
    child_boxes: Vec<String>,
) -> () {

    let mut largest: i32 = 0;
    
    for x in &child_boxes {
        find_largest_label(&mut largest, widget_from_id::<Input>(&x[..]).unwrap().measure_label())
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
    input_one.set_pos(coords.0 + input_one.measure_label().0 + 3, coords.1 + input_one.height());

    widget.add(&input_one);    
    widget.end();

    *datum = Some((coords.0, coords.1 + input_one.height()));

    String::from(id)
}


fn clear_sub_window_scroll(
) -> () {
    let scroll: Option<Scroll> = widget_from_id::<Scroll>("main_window_sub_window_scroll");
    match scroll {
        Some(mut w) => {
            w.clear();
            w.redraw();
        },
        None => {},
    }

    ()
}


fn find_largest_label(
    largest: &mut i32,
    label: (i32, i32),
) -> () {
    if *largest < label.0 {
        *largest = label.0;
    }

    ()
}


fn slice_end_of_string(
    s: String,
) -> String {
    let mut x: Vec<&str> = s.split("/").collect();
    
    match x.pop() {
        Some(y) => y.to_string(),
        None => String::new(),
    }
}


fn tree_selected_callback(
    tree_sender: &Sender<Message>,
    t: &Tree,
) -> () {

    clear_sub_window_scroll();

    match t.callback_item() {
        Some(tree_item) => {
            match t.item_pathname( &tree_item ) {
                Ok( t ) => {
                    tree_sender.send( Message::TreeSelection( slice_end_of_string( t ) ) );
                },
                Err( _ ) => { },
            }
        },
        None => { },
    };

    ()
}


fn add_child_treeitem(
    item: TreeItem,
    parent: TreeItem,
    tree: Tree,
) -> Result<(), ()> {

    tree.add(item.path);
    Ok(())
}