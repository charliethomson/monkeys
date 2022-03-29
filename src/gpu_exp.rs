#[macro_use]
extern crate glium;

use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::Path,
    time::{Instant, UNIX_EPOCH},
};

#[allow(unused_imports)]
use glium::glutin;
use glium::{glutin::dpi::PhysicalSize, program::ComputeShader, uniforms::UniformBuffer};

fn remap(x: usize, in_min: usize, in_max: usize, out_min: usize, out_max: usize) -> usize {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

fn load_words<P: AsRef<Path>>(path: P) -> HashSet<String> {
    let mut buffer = String::new();
    let mut file_handle = File::open(path).unwrap();
    file_handle.read_to_string(&mut buffer).unwrap();

    return buffer
        .split(',')
        .map(<_ as ToString>::to_string)
        .collect::<HashSet<String>>();
}

const MAX_WORD_COUNT: usize = 5;

fn main() {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let cb = glutin::ContextBuilder::new();
    let size = PhysicalSize {
        width: 800,
        height: 600,
    };
    let context = cb.build_headless(&event_loop, size).unwrap();
    let context = unsafe { context.treat_as_current() };
    let hamlet_words = load_words("./resources/hamletWords.csv");

    let display = glium::backend::glutin::headless::Headless::new(context).unwrap();
    let program = glium::program::ComputeShader::from_source(
        &display,
        r#"\
            #version 430
            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
            layout(std140) buffer MyBlock {
                float time;
                vec4 values[4096/4];
            };

            
            // A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
            uint hash( uint x ) {
                x += ( x << 10u );
                x ^= ( x >>  6u );
                x += ( x <<  3u );
                x ^= ( x >> 11u );
                x += ( x << 15u );
                return x;
            }



            // Compound versions of the hashing algorithm I whipped together.
            uint hash( uvec2 v ) { return hash( v.x ^ hash(v.y)                         ); }
            uint hash( uvec3 v ) { return hash( v.x ^ hash(v.y) ^ hash(v.z)             ); }
            uint hash( uvec4 v ) { return hash( v.x ^ hash(v.y) ^ hash(v.z) ^ hash(v.w) ); }



            // Construct a float with half-open range [0:1] using low 23 bits.
            // All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
            float floatConstruct( uint m ) {
                const uint ieeeMantissa = 0x007FFFFFu; // binary32 mantissa bitmask
                const uint ieeeOne      = 0x3F800000u; // 1.0 in IEEE binary32

                m &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
                m |= ieeeOne;                          // Add fractional part to 1.0

                float  f = uintBitsToFloat( m );       // Range [1:2]
                return f - 1.0;                        // Range [0:1]
            }



            // Pseudo-random value in half-open range [0:1].
            float random( float x ) { return floatConstruct(hash(floatBitsToUint(x))); }
            float random( vec2  v ) { return floatConstruct(hash(floatBitsToUint(v))); }
            float random( vec3  v ) { return floatConstruct(hash(floatBitsToUint(v))); }
            float random( vec4  v ) { return floatConstruct(hash(floatBitsToUint(v))); }


            void main() {
                vec3 inputs1 = vec3(gl_GlobalInvocationID.xy, time);
                vec3 inputs2 = vec3(gl_GlobalInvocationID.xy, time * time);
                vec3 inputs3 = vec3(gl_GlobalInvocationID.xy, time * time * time);
                vec3 inputs4 = vec3(gl_GlobalInvocationID.xy, time * time * time * time);
                values[gl_GlobalInvocationID.x] = vec4(random(inputs1), random(inputs2), random(inputs3), random(inputs4));
            }
        "#,
    )
    .unwrap();

    let mut last_word = String::new();

    let mut last_flush = Instant::now();
    let mut found_words = vec![];

    let mut file_handle = File::create(format!("logs/gpu-{}", 1234)).unwrap();

    let mut time = 0.;

    loop {
        let mut buffer: glium::uniforms::UniformBuffer<Data> =
            glium::uniforms::UniformBuffer::empty(&display).unwrap();

        time += 0.01;

        let result = spin(buffer, &program, time);

        last_word.push_str(&result);
        let mut words = last_word.split_ascii_whitespace().peekable();
        println!(
            "{:#?}",
            words.next().unwrap().chars().take(5).collect::<String>(),
        );
        while words.peek().is_some() {
            if let Some(word) = words.next().map(<_ as ToString>::to_string) {
                if hamlet_words.contains(&word) {
                    found_words.push(word);
                }
            } else {
                unreachable!()
            }
        }

        if found_words.len() > MAX_WORD_COUNT {
            println!(
                "Took {}ms to get {} words",
                Instant::now().duration_since(last_flush).as_millis(),
                hamlet_words.len()
            );

            let out_bytes: Vec<u8> = found_words.join("\n").bytes().collect();
            file_handle.write_all(&out_bytes).unwrap();

            found_words.clear();

            last_flush = Instant::now();
        } else {
        }
    }
}
const NUM_VALUES: usize = 1024 * 4;

#[repr(C)]
#[derive(Clone, Copy)]
struct Data {
    time: f32,
    _padding: [f32; 3],
    values: [[f32; 4]; NUM_VALUES / 4],
}

fn spin(mut buffer: UniformBuffer<Data>, program: &ComputeShader, time: f32) -> String {
    let value = [
        fastrand::f32(),
        fastrand::f32(),
        fastrand::f32(),
        fastrand::f32(),
    ];

    println!("{:#?}: {:?}", time, value);
    {
        let mut mapping = buffer.map();
        mapping.time = time;
        for val in mapping.values.iter_mut() {
            *val = value;
        }
    }

    implement_uniform_block!(Data, time, values);

    program.execute(uniform! { MyBlock: &*buffer }, NUM_VALUES as u32 / 4, 1, 1);
    let allowed_chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ ";

    let mut output = String::new();

    {
        let mapping = buffer.map();

        for val in mapping.values.iter() {
            for i in val {
                let b = (i * 255.).floor() as u8;
                let r = remap(b as usize, 0, 255, 0, 52);
                let c = allowed_chars[r] as char;
                output.push(c);
            }
        }
    }
    output
}
