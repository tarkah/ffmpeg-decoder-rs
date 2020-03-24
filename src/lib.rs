#![allow(non_upper_case_globals)]

use crate::error::Error;

use libav_sys::avformat::{
    av_frame_alloc, av_frame_free, av_frame_unref, av_get_alt_sample_fmt, av_get_bytes_per_sample,
    av_get_sample_fmt_name, av_init_packet, av_packet_unref, av_read_frame, av_register_all,
    av_sample_fmt_is_planar, avcodec_alloc_context3, avcodec_close, avcodec_find_decoder,
    avcodec_free_context, avcodec_open2, avcodec_parameters_to_context, avcodec_receive_frame,
    avcodec_send_packet, avformat_close_input, avformat_find_stream_info, avformat_open_input,
    AVCodec, AVCodecContext, AVFormatContext, AVFrame, AVMediaType_AVMEDIA_TYPE_AUDIO, AVPacket,
    AVSampleFormat_AV_SAMPLE_FMT_S16, AVStream,
};
use libav_sys::swresample::{
    av_get_channel_layout_nb_channels, av_samples_alloc, av_samples_get_buffer_size,
    swr_alloc_set_opts, swr_convert, swr_get_out_samples, swr_init, SwrContext,
};
use std::ffi::{CStr, CString};

const AVERROR_EOF: i32 = -0x20464F45;
const AVERROR_EAGAIN: i32 = -11;

mod error;

pub fn run() -> Result<(), Error> {
    let in_file = CString::new("assets/BGM_AI.at3").unwrap();

    unsafe {
        av_register_all();

        let mut format_ctx = std::ptr::null_mut::<AVFormatContext>();

        let status = avformat_open_input(
            &mut format_ctx,
            in_file.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if status != 0 {
            panic!("Could not open file");
        }

        let status = avformat_find_stream_info(format_ctx, std::ptr::null_mut());
        if status < 0 {
            panic!("Could not find stream info");
        }

        //  Get the first audio stream index
        let num_streams = format_ctx.as_ref().unwrap().nb_streams;
        let streams =
            std::slice::from_raw_parts(format_ctx.as_ref().unwrap().streams, num_streams as usize);
        let stream_idx = find_audio_stream(streams, num_streams)?;

        // Get streams codec
        let codec_params = streams[stream_idx].as_ref().unwrap().codecpar;
        let codec_id = codec_params.as_ref().unwrap().codec_id;
        let codec: *mut AVCodec = avcodec_find_decoder(codec_id);
        if codec.is_null() {
            return Err(Error::NullCodec);
        }

        // Initialize codec context
        let mut codec_ctx: *mut AVCodecContext = avcodec_alloc_context3(codec);
        if codec_ctx.is_null() {
            return Err(Error::NullCodecContext);
        }

        // Copy codec params from source file to codec context
        let status = avcodec_parameters_to_context(codec_ctx, codec_params);
        if status != 0 {
            return Err(Error::CodecParamsToContext);
        }

        // Request non planar data
        let alt_format = av_get_alt_sample_fmt(codec_ctx.as_ref().unwrap().sample_fmt, 0);
        codec_ctx.as_mut().unwrap().request_sample_fmt = alt_format;

        // Initialize the decoder
        let status = avcodec_open2(codec_ctx, codec, &mut std::ptr::null_mut());
        if status != 0 {
            return Err(Error::InitializeDecoder);
        }

        print_stream_info(codec, codec_ctx, 1);

        // Allocate frame
        let mut frame: *mut AVFrame = av_frame_alloc();
        if frame.is_null() {
            return Err(Error::NullFrame);
        }

        // Initialize packet
        let packet: *mut AVPacket = {
            let mut packet_uninit = std::mem::MaybeUninit::uninit();
            av_init_packet(packet_uninit.as_mut_ptr());
            packet_uninit.as_mut_ptr()
        };

        let swr_ctx: *mut SwrContext = swr_alloc_set_opts(
            std::ptr::null_mut(),
            codec_params.as_ref().unwrap().channel_layout as i64,
            AVSampleFormat_AV_SAMPLE_FMT_S16,
            codec_params.as_ref().unwrap().sample_rate,
            codec_params.as_ref().unwrap().channel_layout as i64,
            codec_params.as_ref().unwrap().format,
            codec_params.as_ref().unwrap().sample_rate,
            0,
            std::ptr::null_mut(),
        );
        let status = swr_init(swr_ctx);
        if status != 0 {
            return Err(Error::InitializeSwr);
        }

        // Data buffer to store decoded samples
        let mut data: Vec<u8> = vec![];

        loop {
            let status = av_read_frame(format_ctx, packet);
            if status == AVERROR_EOF as i32 {
                break;
            } else if status != 0 {
                return Err(Error::ReadFrame(status));
            }

            // Reset packet if frame doesn't belong to this stream
            if packet.as_ref().unwrap().stream_index as usize != stream_idx {
                av_packet_unref(packet);
                continue;
            }

            let status = avcodec_send_packet(codec_ctx, packet);
            if status == 0 {
                av_packet_unref(packet);
            } else {
                return Err(Error::SendPacket(status));
            }

            let status = receive_and_handle(&mut data, swr_ctx, codec_ctx, frame);
            if status != AVERROR_EAGAIN {
                break;
            }
        }

        // Drain the decoder.
        drain_decoder(codec_ctx)?;

        // Free all data used by the frame.
        av_frame_free(&mut frame);

        // Close the context and free all data associated to it, but not the context itself.
        avcodec_close(codec_ctx);

        // Free the context itself.
        avcodec_free_context(&mut codec_ctx);

        // Close the input.
        avformat_close_input(&mut format_ctx);

        let _ = std::fs::write("assets/BGM_AI.raw", data);
    }

    Ok(())
}

fn print_stream_info(codec: *mut AVCodec, codec_ctx: *mut AVCodecContext, stream_idx: u32) {
    let codec_name = unsafe { CStr::from_ptr(codec.as_ref().unwrap().long_name) };
    let sample_fmt = unsafe {
        CStr::from_ptr(av_get_sample_fmt_name(
            codec_ctx.as_ref().unwrap().sample_fmt,
        ))
    };
    let sample_rate = unsafe { codec_ctx.as_ref().unwrap().sample_rate };
    let sample_size = unsafe { av_get_bytes_per_sample(codec_ctx.as_ref().unwrap().sample_fmt) };
    let channels = unsafe { codec_ctx.as_ref().unwrap().channels };
    let is_planar = unsafe { av_sample_fmt_is_planar(codec_ctx.as_ref().unwrap().sample_fmt) };

    println!("Codec:         {}", codec_name.to_str().unwrap());
    println!("Stream:        {}", stream_idx);
    println!("Sample Format: {}", sample_fmt.to_str().unwrap());
    println!("Sample Rate:   {}", sample_rate);
    println!("Sample Size:   {}", sample_size);
    println!("Channels:      {}", channels);
    println!("Planar:        {}", is_planar);
}

fn find_audio_stream(streams: &[*mut AVStream], num_streams: u32) -> Result<usize, Error> {
    for n in 0..num_streams as usize {
        let codec_type = unsafe {
            streams[n]
                .as_ref()
                .unwrap()
                .codecpar
                .as_ref()
                .unwrap()
                .codec_type
        };

        if codec_type == AVMediaType_AVMEDIA_TYPE_AUDIO {
            return Ok(n);
        }
    }

    Err(Error::NoAudioStream)
}

fn drain_decoder(codec_ctx: *mut AVCodecContext) -> Result<(), Error> {
    let status = unsafe { avcodec_send_packet(codec_ctx, std::ptr::null()) };
    if status == 0 {
    } else {
        return Err(Error::DraidDecoder(status));
    }

    Ok(())
}

fn receive_and_handle(
    data: &mut Vec<u8>,
    swr_ctx: *mut SwrContext,
    codec_ctx: *mut AVCodecContext,
    frame: *mut AVFrame,
) -> i32 {
    loop {
        let status = unsafe { avcodec_receive_frame(codec_ctx, frame) };
        if status == 0 {
            handle_frame(data, swr_ctx, frame);
            unsafe { av_frame_unref(frame) };
        } else {
            return status;
        }
    }
}

fn handle_frame(data: &mut Vec<u8>, swr_ctx: *mut SwrContext, frame: *mut AVFrame) {
    let num_samples = unsafe { frame.as_ref().unwrap().nb_samples };

    let mut out_buf = std::ptr::null_mut::<u8>();
    let out_channels =
        unsafe { av_get_channel_layout_nb_channels(frame.as_ref().unwrap().channel_layout) };
    let out_samples = unsafe { swr_get_out_samples(swr_ctx, num_samples) };

    unsafe {
        av_samples_alloc(
            &mut out_buf,
            std::ptr::null_mut(),
            out_channels,
            out_samples,
            AVSampleFormat_AV_SAMPLE_FMT_S16,
            0,
        )
    };

    let extended_data = unsafe { frame.as_ref().unwrap().extended_data as *mut *const u8 };

    unsafe {
        swr_convert(
            swr_ctx,
            &mut out_buf,
            out_samples,
            extended_data,
            num_samples,
        )
    };

    let out_size = unsafe {
        av_samples_get_buffer_size(
            std::ptr::null_mut(),
            out_channels,
            out_samples,
            AVSampleFormat_AV_SAMPLE_FMT_S16,
            0,
        )
    };
    let out_data = unsafe { std::slice::from_raw_parts(out_buf, out_size as usize) };

    data.extend_from_slice(out_data);
}
