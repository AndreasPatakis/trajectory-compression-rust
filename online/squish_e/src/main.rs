use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use std::time::Instant;
use std::error::Error;
use csv;

pub static DBL_MAX: f64 = ::std::f64::MAX;

#[derive(Default, Debug, Copy, Clone)]
struct Point {
    lat: f64,
    lon: f64,
    time: f64,
}

#[derive(Default, Debug, Copy, Clone)]
struct GPSPointWithSED {
    priority: f64,
    pi: f64,
    point: Point,
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

fn write_to_file(buffer: &Vec<GPSPointWithSED>, path: &str) -> Result<(), Box<dyn Error>> {
    // Creates new `Writer` for `stdout`
    let mut writer = csv::Writer::from_path(path)?;

    // Write records
    for buff in buffer.iter(){
        writer.write_record(&[buff.point.lat.to_string(), buff.point.lon.to_string(), buff.point.time.to_string()])?;
    }

    // A CSV writer maintains an internal buffer, so it's important
    // to flush the buffer when you're done.
    writer.flush()?;

    Ok(())
}

fn cacl_sed(s: &Point, m: &Point, e: &Point) -> f64 {
    let numerator = m.time - s.time;
    let denominator = e.time - s.time;
    let time_ratio = if denominator != 0.0 {
        numerator / denominator
    } else {
        1.0
    };
    let lat = s.lat + (e.lat - s.lat)*time_ratio;
    let lon = s.lon + (e.lon - s.lon)*time_ratio;
    let lat_diff = lat - m.lat;
    let lon_diff = lon - m.lon;
    (lat_diff.powi(2) + lon_diff.powi(2)).sqrt()
}

fn adjust_priority(mut queue: Vec<GPSPointWithSED>,pre_index:i32,q_index:i32,succ_index:i32) -> Vec<GPSPointWithSED>{
    if (q_index as usize)==(queue.len() -1) || q_index==0 {
        return queue;
    }
    let p = queue[q_index as usize].pi + cacl_sed(&queue[pre_index as usize].point,&queue[q_index as usize].point,&queue[succ_index as usize].point);
    queue[q_index as usize].priority = p;
    queue
}

fn find_min_priority(queue: &Vec<GPSPointWithSED>) -> usize {
    let mut to_remove_i = queue.len();
    let mut min_index = 1;
    let mut k = 1;
    for (curr_i, _buff) in queue[1..queue.len()-1].iter().enumerate() {
        if to_remove_i == queue.len() || queue[curr_i+1].priority < queue[to_remove_i].priority{
            to_remove_i = curr_i + 1;
            min_index = k;
        }
        k += 1;
    }   
    min_index
}

fn reduce(mut queue: Vec<GPSPointWithSED>, min_index: usize, min_p:f64) -> Vec<GPSPointWithSED>{
    queue[min_index-1].pi = min_p.max(queue[min_index-1].pi);
    queue[min_index+1].pi = min_p.max(queue[min_index+1].pi);
    queue = adjust_priority(queue,(min_index as i32) - 2, (min_index as i32) - 1, (min_index as i32) + 1);
    queue = adjust_priority(queue,(min_index as i32) - 1, (min_index as i32) + 1, (min_index as i32) + 2);
    queue.remove(min_index);
    queue
}

fn squish_e(points: &Vec<Point>, ratio:&f64, sed_error:&f64) -> Vec<GPSPointWithSED> {
    let mut capacity = 4;
    let mut queue = Vec::<GPSPointWithSED>::new();
    let mut i = 0;
    let mut min_index: usize;
    let mut min_p: f64;
    while i < points.len() {
        if ((i as f64) / *ratio) >= (capacity as f64) {
            capacity += 1;
        }
        queue.push(GPSPointWithSED{priority:DBL_MAX,pi:0.0,point:points[i]});
        if i > 0 {
            queue = adjust_priority(queue.clone(),(queue.len() as i32) - 3,(queue.len() as i32) - 2,(queue.len() as i32) - 1);
        }
        if (queue.len() as i32) == capacity {
            min_index = find_min_priority(&queue);
            min_p = queue[min_index].priority;
            queue = reduce(queue,min_index,min_p);
        }
        i += 1;
        if i == 100{
            break;
        }
    }
    min_index = find_min_priority(&queue);
    min_p = queue[min_index].priority;
    while min_p <= *sed_error{
        queue = reduce(queue,min_index,min_p);
        min_index = find_min_priority(&queue);
        min_p = queue[min_index].priority;
    }
    queue
}


fn main() {

    // Read arguments
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let ratio: f64 = args[2].parse().unwrap();
    let sed: f64 = args[3].parse().unwrap();
    let save_filename = &args[4];

    // Paths
    let rel_path = String::from("./data/");
    let filename_path = format!("{}{}",&rel_path,&filename);

    // Read datapoints file
    let points = gpsreader(&filename_path);

    let now = Instant::now();
    // Compress
    let points_compr = squish_e(&points,&ratio,&sed);
    let elapsed = now.elapsed();
    println!("SQUISH-E compression time: {:?}", elapsed);

    // Write to file
    let output_csv = format!("{}{}",&rel_path,&save_filename);
    if let Err(e) = write_to_file(&points_compr, &output_csv) {
        eprintln!("{}", e)
    }
}