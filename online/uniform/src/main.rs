use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use std::time::Instant;
use std::error::Error;
use csv;

#[derive(Default, Debug, Copy, Clone)]
struct Point {
    lat: f64,
    lon: f64,
    time: f64,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn gpsreader(filename :&String) -> Vec<Point> {
    // Define Point vector
    let mut points: Vec<Point> = Vec::new();

    // Read lines in file
    if let Ok(lines) = read_lines(&filename) {
        for line in lines {
            if let Ok(datapoint) = line {
                let mut data = datapoint.split_whitespace();
                let mut point = Point::default();
                point.lat = data.next().unwrap().parse().unwrap();
                point.lon = data.next().unwrap().parse().unwrap();
                point.time = data.next().unwrap().parse().unwrap();
                points.push(point);
            }
        }
    // Exit if reading error occurs
    }else {
        println!("open file error !");
        process::exit(0);
    }
    points
}


fn uniform(points: &Vec<Point>, eps: usize) -> Vec<Point>{
    let mut simplified_index = Vec::<usize>::new();
    let mut original_index = 0;
    loop {
        if original_index > points.len() {
            break;
        }
        else{
            simplified_index.push(original_index);
        }
        original_index += eps;
    }
    simplified_index.push(points.len() - 1);

    let mut simplified_points = Vec::<Point>::new();
    for i in simplified_index{
        simplified_points.push(points[i]);
    }
    simplified_points
}

fn write_to_file(points: &Vec<Point>, path: &str) -> Result<(), Box<dyn Error>> {
    // Creates new `Writer` for `stdout`
    let mut writer = csv::Writer::from_path(path)?;

    // Write records
    for point in points.iter(){
        writer.write_record(&[point.lat.to_string(), point.lon.to_string(), point.time.to_string()])?;
    }

    // A CSV writer maintains an internal buffer, so it's important
    // to flush the buffer when you're done.
    writer.flush()?;

    Ok(())
}


fn main() {

    // Read arguments
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let epsilon: usize = args[2].parse().unwrap();
    let save_filename = &args[3];

    // Paths
    let rel_path = String::from("./data/");
    let filename_path = format!("{}{}",&rel_path,&filename);

    // Read datapoints file
    let points = gpsreader(&filename_path);

    let now = Instant::now();
    // Compress
    let points_compr = uniform(&points,epsilon);
    let elapsed = now.elapsed();
    println!("Uniform compression time: {:?}", elapsed);

    // Write to file
    let output_csv = format!("{}{}",&rel_path,&save_filename);
    if let Err(e) = write_to_file(&points_compr, &output_csv) {
        eprintln!("{}", e)
    }
}