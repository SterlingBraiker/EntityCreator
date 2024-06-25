use rusqlite::*;


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


fn main() {
    println!("Hello, world!");
}


fn query(
    sqlite_connection: Connection,
    query_str: &str,
    params: &[(&str, &dyn ToSql)],
) -> RecordSet {
    
    // build out an empty recordset
/*    let rs: RecordSet = create_empty_rs();
    let mut headers: Array<GString> = rs.gd_recordset.get("headers").as_ref().unwrap().to();
    let mut gd_rs: Array<Array<Variant>> = rs.gd_recordset.get("records").as_ref().unwrap().to();
 */
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