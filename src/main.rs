use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::Path,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

fn load_words<P: AsRef<Path>>(path: P) -> HashSet<String> {
    let mut buffer = String::new();
    let mut file_handle = File::open(path).unwrap();
    file_handle.read_to_string(&mut buffer).unwrap();

    return buffer
        .split(',')
        .map(<_ as ToString>::to_string)
        .collect::<HashSet<String>>();
}

const MAX_BUFFER_SIZE: usize = 8192;

fn spin(words: HashSet<String>, index: usize) {
    let mut buffer = String::new();

    let now: u64 = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let seed = now % (index + 3 * index + 5 * index + 7 * index + 9 * index + 11) as u64;

    let mut file_handle = File::options()
        .create_new(true)
        .write(true)
        .open(format!(
            "./logs/found-words-{}-{}-thread{}",
            now, seed, index,
        ))
        .unwrap();

    let rng = fastrand::Rng::with_seed(now);

    let allowed_chars = b"abcdefghijklmnopqrstuvwxyz ";

    let mut word_buffer: Vec<String> = Vec::new();

    let mut last_flush = SystemTime::now();

    loop {
        if word_buffer.len() == MAX_BUFFER_SIZE {
            println!(
                "Thread #{:03}: Took {}ms to get {} words",
                index,
                SystemTime::now()
                    .duration_since(last_flush)
                    .unwrap()
                    .as_millis(),
                MAX_BUFFER_SIZE
            );

            let out_bytes: Vec<u8> = word_buffer.join("\n").bytes().collect();
            file_handle.write_all(&out_bytes).unwrap();

            word_buffer.clear();

            last_flush = SystemTime::now();
        }

        if buffer.ends_with(' ') {
            if words.contains(buffer.trim()) {
                word_buffer.push(buffer.trim().to_string());
            }

            buffer.clear();
        }
        buffer.push(allowed_chars[rng.usize(0..27)] as char);
    }
}

const NUM_THREADS: usize = 64;

fn main() {
    let words = load_words("./resources/hamletWords.csv");

    for thread_index in 0..NUM_THREADS {
        let thread_words = words.clone();

        thread::spawn(move || spin(thread_words, thread_index));
    }

    loop {}
}
