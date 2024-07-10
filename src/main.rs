use rusqlite::*;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs::File;
use fltk::{
    prelude::*, 
    window::Window, 
    app, 
    group::Scroll,
    tree::{ Tree, TreeReason },
    input::Input,
};

struct AppContext{
    fltk_app: fltk::app::App,
    db: Connection,
//    list_of_controls: Vec<i32>,
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


const DB_PATH: &str = "C:\\Rust_Dev\\EntityCreator\\test_data\\cold_storage.db";


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
) -> Result<(), rusqlite::Error> {
    let mut app: AppContext = AppContext {
        fltk_app: app::App::default(),
        db: Connection::open(DB_PATH).unwrap(),
    };
    let mut wind: fltk::window::DoubleWindow = Window::new(100, 100, 800, 600, "Entity Content Creator").with_id("main_window");
    /*
        self.fltk_windows.push(window::Window::default()
        .with_id("sql_window")
        .with_size(1280, 760)
        .center_screen());
     */
    let mut tree: Tree = Tree::default().with_size(200, wind.height()).with_label("tree").with_id("tree_id");
    
    tree.set_callback_reason(TreeReason::Selected);
    tree.set_callback(|t| {
        println!("Selected an item");
        match t.get_selected_items() {
            Some(v) => {
                    /* // fetch the label that was clicked on
                    for y in v {
                        println!("{}", y.label().unwrap());
                    }
                    */
    
                    // build out the UI to the right with appropriate amount of boxes
                    // based on which label was clicked
                    // ( CreateItem, CreateEntity, etc )
                    
                let mut previous_coordinates: Option<(i32, i32)> = None;
    
                for _ in 0..=3{
    //                    let x: i32 = wind.children(); // this now will accurately get children count
     //                   for y in 0..x {
      //                      println!("child is {}", t.window().unwrap().child(y).unwrap().label());
       //                 }
                    add_input_to_window(&mut previous_coordinates);
                }
            },
            None => {
    /*                t.clear();
                    let _ = load_db_into_tree(t);
    */      },
        }
    });
    tree.end();
    let wind2: fltk::window::DoubleWindow = Window::new(
        tree.x() + tree.width(),
        tree.y(),
        wind.width() - tree.width(),
        wind.height(),
        "Entity Content Creator").with_id("sub_window");

    let scroll: Scroll = Scroll::default_fill() 
        .with_id("scroll_group");

    scroll.end();
    wind2.end();

    wind.end();
    wind.show();
    
    let _ = load_items_into_tree(&mut tree, Vec::from(["Entity", "Item", "Composition"]));
    
    app.fltk_app.run().unwrap();
     
    Ok(())
}


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

fn load_items_into_tree(
    tree: &mut Tree,
    items: Vec<&str>,
) -> Result<()> {
    for x in items {
        tree.add(x);
    }
    Ok(())
}

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

fn add_input_to_window(
    datum: &mut Option<(i32, i32)>,
) -> () {
    //         fltk::app::widget_from_id::<fltk::group::Flex>("record_grid_group").as_ref().unwrap().end();
    let mut widget = fltk::app::widget_from_id::<fltk::group::Scroll>("scroll_group");
    widget.as_mut().unwrap().begin();
    let coords = match datum.as_ref() {
        Some(x) => x.clone(),
        None => (0, 0),
    };
    let input_one: Input = Input::default().with_pos(coords.0, coords.1 + 20).with_size(80, 20).with_label("Test");
    println!("x is {}, y is {}", input_one.x(), input_one.y());
    widget.as_mut().unwrap().add(&input_one);
    println!("scroll size h: {}, w: {} ", widget.as_mut().unwrap().height(), widget.as_mut().unwrap().width());
    *datum = Some((coords.0, coords.1 + input_one.height()));
    widget.as_mut().unwrap().end();
    ()
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

/*
impl AppContext {
    fn construct(&mut self) -> () {
        let mut wind: fltk::window::DoubleWindow = Window::new(100, 100, 800, 600, "Entity Content Creator");
    /*
        self.fltk_windows.push(window::Window::default()
        .with_id("sql_window")
        .with_size(1280, 760)
        .center_screen());
     */
        let mut tree: Tree = Tree::default().with_size(200, wind.height()).with_label("tree").with_id("tree_id");
    
        tree.set_callback_reason(TreeReason::Selected);
        tree.set_callback(|t| {
            println!("Selected an item");
            match t.get_selected_items() {
                Some(v) => {
                    /* // fetch the label that was clicked on
                    for y in v {
                        println!("{}", y.label().unwrap());
                    }
                    */
    
                    // build out the UI to the right with appropriate amount of boxes
                    // based on which label was clicked
                    // ( CreateItem, CreateEntity, etc )
                    
                    let mut previous_coordinates: Option<(i32, i32)> = None;
                    let mut wind = t.window().unwrap();
    
                    for _ in 0..=3{
    //                    let x: i32 = wind.children(); // this now will accurately get children count
     //                   for y in 0..x {
      //                      println!("child is {}", t.window().unwrap().child(y).unwrap().label());
       //                 }
                        add_input_to_window(&mut wind,&mut previous_coordinates);
                    }
                },
                None => {
    /*                t.clear();
                    let _ = load_db_into_tree(t);
    */            },
            }
        });
        tree.end();
    
        let mut scroll: Scroll = Scroll::new(
            tree.x() + tree.width(),
            tree.y(),
            wind.width() - tree.width(),
            wind.height(),
            "scroll")
            .with_label("scroll");
        wind.end();
        wind.show();
    
        let _ = load_items_into_tree(&mut tree, Vec::from(["Entity", "Item", "Composition"]));
    
        app.fltk_app.run().unwrap();
        
    
    }
}
    */