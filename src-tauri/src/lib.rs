use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use std::sync::{Arc, Mutex};
use rustfft::{FftPlanner, num_complex::Complex};
use std::collections::HashMap;
use hound;

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>;
// オーディオの初期化
fn init_audio() -> Result<(), Box<dyn std::error::Error>> {
    //音階と周波数の対応表
    let mut freq_map = HashMap::new();
    freq_map.insert("C3", 130.81);
    freq_map.insert("C4", 261.63);
    freq_map.insert("C#4", 277.18);
    freq_map.insert("D4", 293.66);
    freq_map.insert("D#4", 311.13);
    freq_map.insert("E4", 329.63);
    freq_map.insert("F4", 349.23);
    freq_map.insert("F#4", 369.99);
    freq_map.insert("G4", 392.00);
    freq_map.insert("G#4", 415.30);
    freq_map.insert("A4", 440.00);
    freq_map.insert("A#4", 466.16);
    freq_map.insert("B4", 493.88);
    freq_map.insert("C5", 523.25);
    freq_map.insert("C#5", 554.37);
    freq_map.insert("D5", 587.33);
    freq_map.insert("D#5", 622.25);
    freq_map.insert("E5", 659.25);
    freq_map.insert("F5", 698.46);
    freq_map.insert("F#5", 739.99);
    freq_map.insert("G5", 783.99);
    freq_map.insert("G#5", 830.61);
    // hound 設定
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let writer = hound::WavWriter::create("hogehoge.wav", spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));
    let writer_clone = writer.clone();

    // cpal インプットデバイスの取得、ストリームの作成
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("Failed to get default input device");
    let input_config = input_device.default_input_config().expect("Failed to get default input config");

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let input_stream = match input_config.sample_format() {
        SampleFormat::F32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| process_audio::<f32>(data, &writer_clone),
            err_fn,
            None,
        ),
        SampleFormat::I16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| process_audio::<i16>(data, &writer_clone),
            err_fn,
            None,
        ),
        _ => panic!("Unsupported sample format"),
    }?;

    input_stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(500));
    writer.lock().unwrap().take().unwrap().finalize()?;
    Ok(())
}

// ハミング窓関数を定義
fn hamming_window(size: usize) -> Vec<f32> {
    (0..size).map(|i| {
        0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32).cos()
    }).collect()
}


fn process_audio<T>(data: &[T], writer: &WavWriterHandle)
where
    T: Sample,
{
    // サンプルデータを f32 に変換
    let samples: Vec<f32> = data.iter().map(|s| s.to_float_sample().to_sample()).collect();

    // FFTサイズ
    let fft_size = 1024;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    // ハミング窓を適用
    let window = hamming_window(fft_size);
    let mut buffer: Vec<Complex<f32>> = samples.iter()
        .zip(window.iter())
        .map(|(&s, &w)| Complex::new(s * w, 0.0))
        .collect();
    buffer.resize(fft_size, Complex::new(0.0, 0.0));

    // 入力データをComplex型に変換
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();
    buffer.resize(fft_size, Complex::new(0.0, 0.0));

    // FFTの実行
    fft.process(&mut buffer);

    // FFT結果の周波数スペクトルの処理
    let mut magnitude: Vec<f32> = buffer.iter().map(|c| c.norm()).collect();

    // 周波数解像度の計算
    let sample_rate = 44100.0;
    let bin_size = sample_rate / fft_size as f32; // 1 bin当たりの周波数幅

    // 振幅が 3 以下の成分をカット
    let magnitude_threshold = 3.0;
    for i in 0..(fft_size / 2) {
        if magnitude[i] <= magnitude_threshold {
            buffer[i] = Complex::new(0.0, 0.0); // 閾値以下の成分をカット
        }
    }

    magnitude = buffer.iter().map(|c| c.norm()).collect();

    // 最大の振幅を持つ周波数を検出
    let (max_bin, max_value) = magnitude.iter()
        .take(fft_size / 2) // ナイキスト周波数以上は無視
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let dominant_freq = max_bin as f32 * bin_size;
    if *max_value < 1.0 {
        return;
    }
    println!("Dominant frequency: {} Hz, Magnitude: {}", dominant_freq, max_value);

    // 振幅を調整してWAVファイルに保存
    let amplitude = i16::MAX as f32;
    for sample in &samples {
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