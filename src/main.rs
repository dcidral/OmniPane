mod core;
mod services;
mod video_display;
mod wrappers;

use crate::core::{CliArgs, OmniPane, OverlayType};
use crate::services::{
    CurrentTimeService, OverlayService, TemperatureService,
};
use crate::wrappers::ImageBuffer;
use clap::Parser;
use opencv::core::{Mat, UMat};
use std::sync::atomic::Ordering;

fn main() {
    println!("Starting video streaming...");

    match CliArgs::try_parse() {
        Ok(args) => {
            let overlay_providers: Vec<Box<dyn OverlayService>> = args
                .overlays
                .into_iter()
                .map(|config| match config {
                    OverlayType::Time => {
                        Box::new(CurrentTimeService::new()) as Box<dyn OverlayService>
                    }
                    OverlayType::Temperature { sensor_id } => {
                        Box::new(TemperatureService::new(&sensor_id)) as Box<dyn OverlayService>
                    }
                })
                .collect();

            if args.gpu {
                opencv::core::set_use_opencl(true).unwrap();

                if opencv::core::have_opencl().unwrap() && opencv::core::use_opencl().unwrap() {
                    println!("GPU acceleration with opencl enabled.");
                } else {
                    panic!("Unable to use GPU acceleration. OpenCL not available.")
                }
                run_omni_pane::<UMat>(args.channel_urls, overlay_providers);
            } else {
                run_omni_pane::<Mat>(args.channel_urls, overlay_providers);
            }
        }
        Err(e) => {
            panic!("Arguments error: {}", e);
        }
    }
}

fn run_omni_pane<T: ImageBuffer>(channels: Vec<String>, overlays: Vec<Box<dyn OverlayService>>) {
    let mut omni_pane = OmniPane::<T>::new(channels, overlays);

    omni_pane.start();

    // TODO: improve services exit sync
    omni_pane.running.store(false, Ordering::Relaxed);
}
