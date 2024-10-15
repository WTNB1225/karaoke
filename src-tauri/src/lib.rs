use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat, FromSample};
use std::sync::{Arc, Mutex};
use hound;
use rodio::{Decoder, OutputStream, source::Source};
use std::io::BufReader;
use std::fs::File;
fn init_audio() -> Result<(), Box<dyn std::error::Error>> {
    //rodio 設定 
    let (_stream, handle) = OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&handle)?;
    let file = BufReader::new(File::open("D:/workspace/karaoke/src-tauri/BELOVED_Instruments.wav")?);
    let source = Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
    sink.detach();
    //hound 設定
    let spac = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let writer = hound::WavWriter::create("hogehoge.wav", spac)?;
    let writer = Arc::new(Mutex::new(Some(writer)));
    let writer_clone = writer.clone();

    //cpal インプットデバイスの取得、ストリームの作成
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("Failed to get default input device");
    let output_device = host.default_output_device().expect("Failed to get default output device");
    let input_config = input_device.default_input_config().expect("Failed to get default input config");
    let output_config = output_device.default_output_config().expect("Failed to get default output config");
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let input_stream = match input_config.sample_format() {
        SampleFormat::F32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<f32>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::I8 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<i8>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::I16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<i16>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::I32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<i32>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::I64 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<i64>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::U8 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<u8>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::U16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<u16>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::U32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<u32>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::U64 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<u64>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::F64 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input::<f64>(data, &writer_clone),
            err_fn,
            None,
        ),
        _ => panic!("Unsupported sample format"),
    }?; 
    input_stream.play()?;

    let output_stream = output_device.build_output_stream(
        &output_config.into(),
        move |data: &mut [f32], _: &_| {
            println!("Generating {} samples", data.len());
        },
        move |err| {
            eprintln!("An error occurred on the output stream: {}", err);
        },
        None,
    )?;

    std::thread::sleep(std::time::Duration::from_secs(500));
    drop(input_stream);
    writer.lock().unwrap().take().unwrap().finalize()?;
    Ok(())
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>;

fn write_input<T>(data: &[T], writer: &WavWriterHandle)
where
    T: Sample,
{
    let samples: Vec<f32> = data
        .iter()
        .map(|s| s.to_float_sample().to_sample())
        .collect();

    for sample in &samples {
        let amplitude = i16::MAX as f32; //振幅(音量)を調整
        writer.lock().unwrap().as_mut().unwrap().write_sample((sample * amplitude) as i16).unwrap();
    }


}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // AppHandleをクローンしてスレッドに渡す
            std::thread::spawn(move || {
                if let Err(e) = init_audio() {
                    eprintln!("Failed to initialize audio: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}