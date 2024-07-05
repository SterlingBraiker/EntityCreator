use rusqlite::*;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs::File;
use fltk::{prelude::*, window::Window, app, image, frame::Frame};

#[derive(Default)]
struct RecordSet {
    records: Vec<Records>,
    headers: Headers,
}


struct Records {
    fields: Vec<SqlData>,
}

impl Records {
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


fn main(
) -> Result<(), rusqlite::Error> {

    let fltk_app: app::App = app::App::default();
    let mut wind: fltk::window::DoubleWindow = Window::new(100, 100, 800, 600, "Entity Content Creator");
    let mut frame = Frame::default().center_of(&wind);


        
    wind.end();
    wind.show();


    let db = Connection::open_in_memory()?;

    db.execute_batch("CREATE TABLE test_table (content BLOB);")?;

    db.execute("INSERT INTO test_table (content) VALUES (ZEROBLOB(242434))", [])?;


    fltk_app.run().unwrap();

    Ok(())
}


fn query(
    sqlite_connection: Connection,
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
                        let mut new_row: Records = Records::new();
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
                        // godot_print!("no more records in the recordset!");
                    },
                }
            }

        },
        Err(_) => { 
            // godot_print!("statement failed to prepare");
        },
    };
 
    // rs.gd_recordset
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