/*
MIT License

Copyright (c) 2026 Philipp Schuster

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
//! Helpers shared examples.

use audio_visualizer::live::AudioInput;
use std::io::{BufRead, stdin};

/// Lets the user select an audio input device on stdin if there are multiple.
pub fn select_input() -> AudioInput {
    let mut devs = AudioInput::devices().unwrap();
    assert!(!devs.is_empty(), "no audio input devices found!");
    if devs.len() == 1 {
        return AudioInput::from_device(devs.remove(0).1).unwrap();
    }
    println!();
    devs.iter()
        .enumerate()
        // skip devices without a usable input config, and show the config
        // that would actually be recorded with
        .filter_map(|(i, (name, dev))| {
            AudioInput::from_device(dev.clone())
                .ok()
                .map(|input| (i, name, input))
        })
        .for_each(|(i, name, input)| {
            println!("  [{i}] {name} {:?}", input.config());
        });
    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();
    let index = input.trim().parse::<usize>().unwrap();
    AudioInput::from_device(devs.remove(index).1).unwrap()
}
