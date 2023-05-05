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

#[derive(Default, Debug, Copy, Clone)]
struct GPSPointWithSED {
    point: Point,
    sed: f64,
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


fn sttrace(points: &Vec<Point>, cmp_ratio:&f64) -> Vec<GPSPointWithSED> {
    let max_buffer_size = usize::try_from((cmp_ratio*(points.len() as f64)) as i32).unwrap();
    let mut buffer: Vec<GPSPointWithSED> = Vec::with_capacity(max_buffer_size+1);
    buffer.push(GPSPointWithSED{point:points[0], sed:0.0});
    if max_buffer_size > 2 {
        buffer.push(GPSPointWithSED{point:points[1], sed:0.0});
        for i in 2..points.len() {
            buffer.push(GPSPointWithSED{point:points[i], sed:0.0});
            // Compute SED for previous point
            let segment_start: Point = buffer[buffer.len() - 3].point;
            let segment_end: Point = buffer[buffer.len() - 1].point;
            let buff_index = buffer.len() - 2;
            buffer[buff_index].sed += cacl_sed(&segment_start, &buffer[buff_index].point, &segment_end);
            // Buffer full, remove a point
            if buffer.len() > max_buffer_size {
                let mut to_remove_i = buffer.len();
                let mut min_index = 1;
                let mut k = 1;
                for (curr_i, _buff) in buffer[1..buffer.len()-1].iter().enumerate() {
                    if to_remove_i == buffer.len() || buffer[curr_i+1].sed < buffer[to_remove_i].sed{
                        // index + 1 because we start from [1..]
                        to_remove_i = curr_i + 1;
                        min_index = k;
                    }
                    k += 1;
                }
                if min_index - 1 > 0 {
                    buffer[min_index-1].sed = cacl_sed(&buffer[min_index-2].point, &buffer[min_index - 1].point, &buffer[min_index + 1].point)
                }
                if min_index + 1 < buffer.len() - 1{
                    buffer[min_index + 1].sed = cacl_sed(&buffer[min_index - 1].point, &buffer[min_index + 1].point, &buffer[min_index + 2].point);

                }
                buffer.remove(to_remove_i); 
            }
        }
    }
    else {
        buffer.push(GPSPointWithSED{point:points[points.len()-1], sed:0.0});
    }
    buffer
}


fn main() {

    // Read arguments
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let cmp_ratio: f64 = args[2].parse().unwrap();
    let save_filename = &args[3];

    // Paths
    let rel_path = String::from("./data/");
    let filename_path = format!("{}{}",&rel_path,&filename);

    // Read datapoints file
    let points = gpsreader(&filename_path);

    let now = Instant::now();
    // Compress
    let points_compr = sttrace(&points,&cmp_ratio);
    let elapsed = now.elapsed();
    println!("STTrace compression time: {:?}", elapsed);

    // Write to file
    let output_csv = format!("{}{}",&rel_path,&save_filename);
    if let Err(e) = write_to_file(&points_compr, &output_csv) {
        eprintln!("{}", e)
    }
    

}