use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use renderer::Configuration;
use renderer::profiling::*;

fn get_config() -> Configuration {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let configuration_path = resource_dir.join(Configuration::DEFAULT_PATH);
    Configuration::read(&configuration_path)
}

fn read(path: impl AsRef<Path>) {
    let mut file = BufReader::new(File::open(path).unwrap());

    while let Ok(event) = bincode::deserialize_from::<_, MeasurementEvent>(&mut file) {
        println!("{:?}", event);
    }
}

// fn print_sample_names(sample_names: &[String]) {
//     for (sample_index, sample_name) in sample_names.iter().enumerate() {
//         println!("({:04}) {}", sample_index, sample_name);
//     }
// }

// fn print_sample(frames: &Frames, sample_index: usize) {
//     println!("{:>6}, {:>12}, {:>12}", "frame", "cpu", "gpu");
//     for (frame_index, sample) in frames
//         .iter()
//         .enumerate()
//         .filter_map(|(index, samples)| match samples.get(sample_index) {
//             Some(sample) => sample.as_ref().map(|sample| (index, sample)),
//             None => None,
//         })
//     {
//         println!("{:>6}, {:>12}, {:>12}", frame_index, sample.cpu.delta(), sample.gpu.delta());
//     }
// }

fn main() {
    let matches = App::new("Profiling Reader")
        .version("1.0")
        .author("Mick van Gelderen")
        .about("Reads profiling information from logs generated by renderer.")
        .arg(
            Arg::with_name("sample")
                .help("Print data for a sample index")
                .required(false)
                .index(1),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)

    let cfg = get_config();
    read(cfg.profiling.path.as_ref().unwrap());
    // let (frames, sample_names) = 

    // match matches.value_of("sample") {
    //     Some(sample_index_string) => {
    //         let sample_index = sample_index_string.parse::<usize>().unwrap();
    //         print_sample(&frames, sample_index);
    //     }
    //     None => {
    //         println!("Frames: {}", frames.len());
    //         println!("Samples: {}", sample_names.len());
    //         print_sample_names(&sample_names);
    //     }
    // }
}
