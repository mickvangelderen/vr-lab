use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use renderer::configuration;
use renderer::profiling::*;

type Frames = Vec<Vec<Option<GpuCpuTimeSpan>>>;
type SampleNames = Vec<String>;

fn get_config() -> configuration::Root {
    let current_dir = std::env::current_dir().unwrap();
    let resource_dir: PathBuf = [current_dir.as_ref(), Path::new("resources")].into_iter().collect();
    let configuration_path = resource_dir.join(configuration::FILE_PATH);
    configuration::read(&configuration_path)
}

fn read(path: impl AsRef<Path>) -> (Frames, SampleNames) {
    let mut file = BufReader::new(File::open(path).unwrap());

    let mut frames: Frames = Vec::new();
    let mut samples = None;
    while let Ok(entry) = bincode::deserialize_from::<_, FileEntry>(&mut file) {
        match entry {
            FileEntry::Frame(frame) => frames.push(frame),
            FileEntry::Samples(s) => assert!(samples.replace(s).is_none()),
        }
    }
    (frames, samples.unwrap())
}

fn print_sample_names(sample_names: &[String]) {
    for (sample_index, sample_name) in sample_names.iter().enumerate() {
        println!("({:04}) {}", sample_index, sample_name);
    }
}

fn print_sample(frames: &Frames, sample_index: usize) {
    println!("frame, cpu_begin, cpu_end, gpu_begin, gpu_end");
    for (frame_index, sample) in frames
        .iter()
        .enumerate()
        .filter_map(|(index, samples)| match samples.get(sample_index) {
            Some(sample) => sample.as_ref().map(|sample| (index, sample)),
            None => None,
        })
    {
        println!("{:05}, {:08}, {:08}, {:08}, {:08}", frame_index, sample.cpu.begin, sample.cpu.end, sample.gpu.begin, sample.gpu.end);
    }
}

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
    let (frames, sample_names) = read(cfg.profiling.path.as_ref().unwrap());

    match matches.value_of("sample") {
        Some(sample_index_string) => {
            let sample_index = sample_index_string.parse::<usize>().unwrap();
            print_sample(&frames, sample_index);
        }
        None => {
            println!("Frames: {}", frames.len());
            println!("Samples: {}", sample_names.len());
            print_sample_names(&sample_names);
        }
    }
}
