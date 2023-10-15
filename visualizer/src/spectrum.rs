use std::sync::Mutex;
use std::{cell::RefCell, sync::Arc};
use std::cmp::max;

use audio_visualizer::dynamic::live_input::{setup_audio_input_loop, AudioDevAndCfg};
use audio_visualizer::dynamic::{
    live_input::list_input_devs,
};

use cpal::Stream;
use cpal::traits::StreamTrait;

use eframe::egui::Ui;
use eframe::epaint::{Vec2, Color32};
use egui_plot::{PlotResponse, PlotPoints, Line, Plot, log_grid_spacer, PlotBounds};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use spectrum_analyzer::{windows::hann_window, samples_fft_to_spectrum, FrequencyLimit, scaling::divide_by_N, FrequencyValue};

pub struct Bode {
    stream: Stream,
    sampling_rate: f32,
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    smoothed_spectrum: RefCell<Vec<(f64, f64)>>
}

impl Bode {
    pub fn new() -> Self {
        let audio_device = list_input_devs().remove(0).1;
        let audio_device_and_config = AudioDevAndCfg::new(Some(audio_device), None);

        let sampling_rate = audio_device_and_config.cfg().sample_rate.0 as f32;

        let mut buf = AllocRingBuffer::new((5 * sampling_rate as usize).next_power_of_two());
        buf.fill(0.0);
        let latest_audio_data = Arc::new(Mutex::new(buf));

        let smoothed_spectrum = RefCell::new(vec![(0.0, 0.0); 8192]);

        let stream = setup_audio_input_loop(latest_audio_data.clone(), audio_device_and_config);
        stream.play().unwrap();

        Self {
            stream,
            sampling_rate,
            latest_audio_data,
            smoothed_spectrum
        }
    }

    fn get_spectrum(&self) -> Vec<(f64, f64)> {
        let audio = self.latest_audio_data.clone().lock().unwrap().to_vec();
        let relevant_samples = &audio[audio.len() - 8192..];

        let hann_window = hann_window(relevant_samples);
        let latest_spectrum = samples_fft_to_spectrum(
            &hann_window,
            self.sampling_rate as u32,
            FrequencyLimit::Max(10000.0),
            Some(&divide_by_N)
        ).unwrap();

        latest_spectrum
            .data()
            .iter()
            .zip(self.smoothed_spectrum.borrow_mut().iter_mut())
            .for_each(|((new_freq, new_freq_val), (old_freq, old_freq_val))| {
                *old_freq = new_freq.val() as f64;
                let scaled_old_freq_val = *old_freq_val * 0.84;
                let max = max(
                    *new_freq_val * 5000.0_f32.into(),
                    FrequencyValue::from(scaled_old_freq_val as f32),
                );
                *old_freq_val = max.val() as f64;
            });

        self.smoothed_spectrum.borrow().clone()
    }

    pub fn show(&self, ui: &mut Ui) -> PlotResponse<()> {
        let data = self.get_spectrum();
        let length = data.len();
        let data: Vec<(f64, f64)> = data.into_iter().take((length as f64 / 2.0).floor() as usize).collect();

        let points: PlotPoints = data.iter().map(|(freq, freq_val)| {
            [freq.log10().to_owned(), freq_val.log10().to_owned()]
        }).collect();
        //let points: PlotPoints = data.iter().enumerate().map(|(i, (l, r))| {
        //    let x = i as f64;
        //    [x, (l + r) / 2.0]
        //}).collect();
        let line = Line::new(points)
            .width(5.0);
        Plot::new("spectrum")
            .show_grid([false; 2])
            .show_axes([false; 2])
            .show_x(false)
            .show_y(false)
            .view_aspect(4.0)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([1.5, -3.0], [4.0, 6.0]));
                plot_ui.line(line)
            })
    }
}
