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

fn cacl_distance(points: &Vec<Point>) -> Vec<f64>{
    let mut distance = Vec::<f64>::new();
    for i in 1..points.len() {
        distance.push(((points[i].lat - points[i-1].lat).powi(2) + (points[i].lon - points[i-1].lon).powi(2)).sqrt());
    }
    distance
}

fn cacl_angle(points: &Vec<Point>) -> Vec<f64>{
    let mut angles = Vec::<f64>::new();
    for i in 1..points.len() {
        let lat_diff = points[i].lat - points[i-1].lat;
        let lon_diff = points[i].lon - points[i-1].lon;
        angles.push(lon_diff.atan2(lat_diff));
    }
    angles
}

fn dead_reckoning(points: &Vec<Point>, eps: &f64) -> Vec<Point>{
    let n = points.len();
    let mut max_d: f64 = 0.0;
    let mut start_idx = 0;
    let d = cacl_distance(&points);
    let angles = cacl_angle(&points);

    let mut simplified_index = Vec::<usize>::new();
    simplified_index.push(0);
    for i in 2..n {
        max_d += (d[i-1]*(angles[i-1] - angles[start_idx]).sin()).abs();
        if max_d.abs() > *eps {
            max_d = 0.0;
            simplified_index.push(i-1);
            start_idx = i-1;
        }
    }
    if simplified_index[simplified_index.len()-1] != n-1 {
        simplified_index.push(n-1);
    }
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
    let epsilon: f64 = args[2].parse().unwrap();
    let save_filename = &args[3];

    // Paths
    let rel_path = String::from("./data/");
    let filename_path = format!("{}{}",&rel_path,&filename);

    // Read datapoints file
    let points = gpsreader(&filename_path);

    let now = Instant::now();
    // Compress
    let points_compr = dead_reckoning(&points,&epsilon);
    let elapsed = now.elapsed();
    println!("Dead Reckoning compression time: {:?}", elapsed);

    // Write to file
    let output_csv = format!("{}{}",&rel_path,&save_filename);
    if let Err(e) = write_to_file(&points_compr, &output_csv) {
        eprintln!("{}", e)
    }
    

}