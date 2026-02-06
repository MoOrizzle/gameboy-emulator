use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use ringbuf::{HeapRb, traits::*};

pub struct AudioQueue {
    producer_left: Arc<Mutex<ringbuf::HeapProd<f32>>>,
    producer_right: Arc<Mutex<ringbuf::HeapProd<f32>>>,
    _stream: Stream,
}

impl AudioQueue {
    pub fn new() -> Self {
        let ring_left = HeapRb::<f32>::new(8192);
        let ring_right = HeapRb::<f32>::new(8192);
        
        let (prod_left, cons_left) = ring_left.split();
        let (prod_right, cons_right) = ring_right.split();

        let producer_left = Arc::new(Mutex::new(prod_left));
        let producer_right = Arc::new(Mutex::new(prod_right));

        let mut cons_left = cons_left;
        let mut cons_right = cons_right;

        let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("No output device available");
        
        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(44100),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(2) {
                    let l = cons_left.try_pop().unwrap_or(0.0);
                    let r = cons_right.try_pop().unwrap_or(0.0);
                    
                    frame[0] = l;
                    frame[1] = r;
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        ).expect("Failed to build audio stream");

        stream.play().expect("Failed to play stream");

        Self {
            producer_left,
            producer_right,
            _stream: stream,
        }
    }

    pub fn push_samples(&self, left: &[f32], right: &[f32]) {
        if left.is_empty() { return; }
        
        let mut prod_left = self.producer_left.lock().unwrap();
        let mut prod_right = self.producer_right.lock().unwrap();

        let available = prod_left.vacant_len();
        if available < 2048 { return; }

        let occupied = 8192 - available;
        if occupied < 2048 {
            for _ in 0..100 {
                let _ = prod_left.try_push(0.0);
                let _ = prod_right.try_push(0.0);
            }
        }
        
        for (&l, &r) in left.iter().zip(right.iter()) {
            let _ = prod_left.try_push(l);
            let _ = prod_right.try_push(r);
        }
    }
}