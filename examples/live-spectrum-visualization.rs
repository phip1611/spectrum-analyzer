/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

use std::io::Stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset};
use tui::{symbols, Terminal};

use spectrum_analyzer::scaling::{combined, divide_by_N, scale_20_times_log10};
use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum};

/// Run in terminal (not IDE!) and it will open an alternate screen where you can see
/// the nice visualization. Unfortunately, this isn't really sexy so far.. :(
///
/// TODO upstream this to "audio-visualizer" crate.
fn main() {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let continue_work = Arc::new(AtomicBool::new(true));

    {
        let continue_work = continue_work.clone();
        ctrlc::set_handler(move || {
            continue_work.store(false, Ordering::SeqCst);
        })
        .unwrap();
    }

    terminal
        .backend_mut()
        .execute(EnterAlternateScreen)
        .unwrap();

    let latest_spectrum_data = Arc::new(Mutex::new(FrequencySpectrum::default()));
    let latest_audio_data = Mutex::new(ConstGenericRingBuffer::<f32, 2048>::new());
    (0..2048).for_each(|_| latest_audio_data.lock().unwrap().push(0.0));

    let stream = setup_audio_input_loop(latest_spectrum_data.clone(), latest_audio_data);
    stream.play().unwrap();
    visualize_loop(&mut terminal, continue_work, latest_spectrum_data);
    stream.pause().unwrap();

    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .unwrap();

    println!("Gracefully shut down.");
}

fn visualize_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    continue_work: Arc<AtomicBool>,
    latest_spectrum_data: Arc<Mutex<FrequencySpectrum>>,
) {
    while continue_work.load(Ordering::SeqCst) {
        // prepare the data for the TUI diagram
        let data = {
            let data = latest_spectrum_data.lock().unwrap();
            let data = data.to_log_spectrum();
            let mut new_data = Vec::with_capacity(data.len());
            data.iter()
                .map(|(fr, fr_val)| (fr.val() as f64, fr_val.val() as f64))
                .for_each(|x| new_data.push(x));
            new_data
        };
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(100), Constraint::Percentage(0)].as_ref())
                    .split(f.size());
                let datasets = vec![
                    Dataset::default()
                        // .name("data2")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Yellow))
                        .data(data.as_slice()),
                    /*.data(&[
                        (0.0, 0.0),
                        (100.0, 100.0),
                        (200.0, 120.0),
                        (300.0, 100.0),
                        (400.0, 50.0),
                    ]),*/
                ];
                let chart = Chart::new(datasets)
                    .block(
                        Block::default()
                            .title(Span::styled(
                                "Frequency Spectrum",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ))
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        Axis::default()
                            .title("Frequency (Hz)")
                            .style(Style::default().fg(Color::Gray))
                            .labels(vec![
                                Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                                //Span::raw("0"),
                                Span::styled(
                                    "22050",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                            ])
                            .bounds([0.0, 22050.0]),
                    )
                    .y_axis(
                        Axis::default()
                            .title("Amplitude")
                            .style(Style::default().fg(Color::Gray))
                            .labels(vec![
                                Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                                //Span::raw("0"),
                                Span::styled("70", Style::default().add_modifier(Modifier::BOLD)),
                            ])
                            .bounds([0.0, 70.0]),
                    );
                f.render_widget(chart, chunks[0]);
            })
            .unwrap();
        sleep(Duration::from_millis(100));
    }
}

/// Sets up `cpal` library to listen for audio on the default input device
/// and connects the data callback with the spectrum visualizer via the
/// helper method [`process_audio_input`].
fn setup_audio_input_loop(
    latest_spectrum_data: Arc<Mutex<FrequencySpectrum>>,
    latest_audio_data: Mutex<ConstGenericRingBuffer<f32, 2048>>,
) -> cpal::Stream {
    const WINDOW_LENGTH: u32 = 2048;
    let host = cpal::default_host();
    let input = host.default_input_device().unwrap_or_else(|| {
        panic!(
            "No default audio input device found for host {}",
            host.id().name()
        )
    });
    let cfg = StreamConfig {
        channels: 1,
        sample_rate: SampleRate(44100),
        // 2048 samples with 1/44100 seconds per sample is ~46ms
        buffer_size: BufferSize::Fixed(WINDOW_LENGTH),
    };
    input
        .build_input_stream(
            &cfg,
            // this is pretty cool by "cpal"; we can use u16, i16 or f32 and
            // the type system does all the magic behind the scenes
            move |data: &[f32], _info| {
                process_audio_input(data, &latest_spectrum_data, &latest_audio_data);
            },
            |_err| {},
        )
        .unwrap()
}

/// Invoked on each audio callback by the audio library. Calculates the spectrum
/// from the latest audio input data.
///
/// On my Linux machine I usually get something 534 samples per callback. For better
/// accuracy and simpler calculation, I use a ringbuffer with a capacity of 2048 elements.
///
/// Each element in `data` is in range `[-1, 1]`.
fn process_audio_input(
    data: &[f32],
    latest_frequency_spectrum: &Arc<Mutex<FrequencySpectrum>>,
    audio_ring_buf: &Mutex<ConstGenericRingBuffer<f32, 2048>>,
) {
    let mut lock = audio_ring_buf.lock().unwrap();
    // scale each value from -1 to 1 ... I'm not sure if this is really required but I think
    // the results will be more accurate in the end..
    data.iter()
        .map(|x| *x * i16::MAX as f32)
        .for_each(|x| lock.push(x));

    // calculate spectrum
    let spectrum = spectrum_analyzer::samples_fft_to_spectrum(
        lock.to_vec().as_slice(),
        44100,
        FrequencyLimit::All,
        // Some(&spectrum_analyzer::scaling::scale_20_times_log10),
        Some(&combined(&[&divide_by_N, &scale_20_times_log10])),
    )
    .unwrap();

    let mut lock = latest_frequency_spectrum.lock().unwrap();
    *lock = spectrum;
}

#[cfg(test)]
mod tests {
    // use super::*;
}
