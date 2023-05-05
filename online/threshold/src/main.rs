use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use std::time::Instant;
use std::error::Error;
use csv;

const EARTH_RADIUS:i32 = 6371229;
const M_PI:f64 = std::f64::consts::PI;

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

fn cacl_haversine(a: &Point, b: &Point) -> f64{
    let lat1 = a.lat * M_PI / 180.0;
    let lat2 = b.lat * M_PI / 180.0;
    let lon1 = a.lon * M_PI / 180.0;
    let lon2 = b.lon * M_PI / 180.0;
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a_a = ((dlat/2.0).sin()).powi(2) + ((dlon/2.0).sin()).powi(2) * lat1.cos() * lat2.cos();
    let c = 2.0 * (((a_a).sqrt()).atan2((1.0 - a_a).sqrt()));
    (EARTH_RADIUS as f64)*c
}

fn cacl_angle(a: &Point, b: &Point) -> f64 {
    let lat_diff = b.lat - a.lat;
    let lon_diff = b.lon - a.lon;
    lon_diff.atan2(lat_diff)
}

fn cacl_distance(a: &Point, b: &Point, dataset:&i32) -> f64 {
    if *dataset == 0 {
        return ((a.lat-b.lat).powi(2) + (a.lon-b.lon).powi(2)).sqrt();
    } else if *dataset == 1 {
        return ((a.lat/1000.0 - b.lat/1000.0).powi(2) + (a.lon/1000.0 - b.lon/1000.0).powi(2)).sqrt();
    }else{
        return cacl_haversine(a, b);
    }
}

fn cacl_speed(a: &Point, b: &Point, dataset:&i32) -> f64 {
    cacl_distance(a,b,dataset)/(b.time-a.time)
}

fn safe_speed(sample_b:&Point,sample_c:&Point,point_c:&Point,point_d:&Point,point_e:&Point,dataset:&i32,speed_threshold:&f64) -> bool {
    let sample_speed = cacl_speed(sample_b,sample_c,dataset);
    let trajectory_speed = cacl_speed(point_c,point_d,dataset);
    let de_speed = cacl_speed(point_d,point_e,dataset);
    if (sample_speed-de_speed).abs() > *speed_threshold || (trajectory_speed-de_speed).abs() > *speed_threshold {
        return false;
    }else{
        return true;
    }
}

fn safe_orientation(sample_b:&Point,sample_c:&Point,point_c:&Point,point_d:&Point,point_e:&Point, ori_threshold:&f64) -> bool {
    let angle_sample_bc = cacl_angle(sample_b, sample_c);
    let angle_de = cacl_angle(point_d, point_e);
    let angle_sample_bc_de = angle_de - angle_sample_bc;
    let angle_trajectory_cd = cacl_angle(point_c, point_d);
    let angle_trajectory_cd_de = angle_de - angle_trajectory_cd;
    if angle_sample_bc_de.abs() > *ori_threshold || angle_trajectory_cd_de.abs() > *ori_threshold {
        return false;
    }else{
        return true;
    }
}


fn threshold(points: &Vec<Point>, speed_threshold: &f64, dataset:&i32, ori_threshold:&f64) -> Vec<Point>{
    let mut sample = Vec::<Point>::new();
    let mut simplified_index = Vec::<i32>::new();
    sample.push(points[0]);
    sample.push(points[1]);
    simplified_index.push(0);
    simplified_index.push(1);
    for i in 2..(points.len()-1) {
        let has_safe_speed = safe_speed(&sample[sample.len()-2], &sample[sample.len()-1], &points[i-2], &points[i-1], &points[i],dataset,speed_threshold);
        let has_safe_orientation = safe_orientation(&sample[sample.len()-2], &sample[sample.len()-1], &points[i-2], &points[i-1], &points[i],ori_threshold);
        if has_safe_speed && has_safe_orientation {
            continue;
        }else{
            sample.push(points[i]);
        }
    }
    sample.push(points[points.len()-1]);
    sample
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
    let speed_threshold: f64 = args[2].parse().unwrap();
    let ori_threshold: f64 = args[3].parse().unwrap();
    let dataset:i32 = args[4].parse().unwrap();
    let save_filename = &args[5];

    // Paths
    let rel_path = String::from("./data/");
    let filename_path = format!("{}{}",&rel_path,&filename);

    // Read datapoints file
    let points = gpsreader(&filename_path);

    let now = Instant::now();
    // Compress
    let points_compr = threshold(&points,&speed_threshold,&dataset,&ori_threshold);
    let elapsed = now.elapsed();
    println!("Threshold compression time: {:?}", elapsed);

    // Write to file
    let output_csv = format!("{}{}",&rel_path,&save_filename);
    if let Err(e) = write_to_file(&points_compr, &output_csv) {
        eprintln!("{}", e)
    }
}